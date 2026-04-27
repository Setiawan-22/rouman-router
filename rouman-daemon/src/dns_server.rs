use hickory_proto::op::{Message, MessageType, OpCode, ResponseCode};
use hickory_proto::rr::{RData, Record};
use hickory_proto::serialize::binary::{BinDecodable, BinDecoder, BinEncodable, BinEncoder};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{UdpSocket, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use rcgen::generate_simple_self_signed;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::TlsAcceptor;

use axum::{
    routing::{get, post},
    extract::{State, Query, FromRef},
    http::{StatusCode, header},
    response::IntoResponse,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use axum::body::Bytes;
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use rouman_api::dns::DnsState;

#[derive(Clone)]
struct DnsResolver {
    http_client: reqwest::Client,
    upstream_udp: Arc<UdpSocket>,
    state: Arc<DnsState>,
    config: Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>>,
}

impl DnsResolver {
    async fn new(
        state: Arc<DnsState>,
        config: Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let upstream_udp = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        Ok(Self {
            http_client: reqwest::Client::builder()
                .pool_idle_timeout(std::time::Duration::from_secs(90))
                .build()?,
            upstream_udp,
            state,
            config,
        })
    }

    async fn resolve(&self, request_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut decoder = BinDecoder::new(request_data);
        let request = Message::read(&mut decoder).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if let Some(query) = request.queries().first() {
            let name = query.name().to_string();
            let clean_name = name.trim_end_matches('.').to_string();
            
            let is_blocked = {
                let blocked = self.state.blocked_domains.lock().await;
                blocked.contains(&clean_name)
            };

            if is_blocked {
                let mut response = Message::new();
                response.set_id(request.id());
                response.set_message_type(MessageType::Response);
                response.set_op_code(OpCode::Query);
                response.set_response_code(ResponseCode::NoError);
                response.add_query(query.clone());
                
                let name = query.name().clone();
                let rdata = RData::A(hickory_proto::rr::rdata::A(Ipv4Addr::new(0, 0, 0, 0)));
                let record = Record::from_rdata(name, 60, rdata);
                response.add_answer(record);

                let mut out_buf = Vec::new();
                let mut encoder = BinEncoder::new(&mut out_buf);
                response.emit(&mut encoder)?;
                return Ok(out_buf);
            }
        }

        // DNS Config
        let (doh_enabled, doh_url) = {
            let conf = self.config.read().await;
            (conf.dns.doh_enabled, conf.dns.doh_url.clone())
        };

        if doh_enabled && !doh_url.is_empty() {
            let res = self.http_client.post(&doh_url)
                .header("Content-Type", "application/dns-message")
                .body(request_data.to_vec())
                .send()
                .await?;

            if res.status().is_success() {
                return Ok(res.bytes().await?.to_vec());
            }
        }

        // Fallback UDP
        let upstream_addr = SocketAddr::from(([8, 8, 8, 8], 53));
        self.upstream_udp.send_to(request_data, upstream_addr).await?;

        let mut upstream_buf = [0u8; 1500];
        match tokio::time::timeout(std::time::Duration::from_secs(2), self.upstream_udp.recv_from(&mut upstream_buf)).await {
            Ok(Ok((resp_size, _))) => Ok(upstream_buf[..resp_size].to_vec()),
            Ok(Err(e)) => Err(e.into()),
            Err(e) => Err(e.into()),
        }
    }
}

// ========================
// DoH HANDLERS
// ========================
async fn doh_post(
    State(resolver): State<Arc<DnsResolver>>,
    body: Bytes,
) -> impl IntoResponse {
    match resolver.resolve(&body).await {
        Ok(resp) => ([(header::CONTENT_TYPE, "application/dns-message")], resp).into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response()
    }
}

async fn doh_get(
    State(resolver): State<Arc<DnsResolver>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let process = |b64: &str| -> Result<Vec<u8>, ()> {
        let b64_clean = b64.replace("=", "");
        URL_SAFE_NO_PAD.decode(&b64_clean).map_err(|_| ())
    };

    if let Some(dns) = params.get("dns") {
        if let Ok(query_bytes) = process(dns) {
            if let Ok(resp) = resolver.resolve(&query_bytes).await {
                return ([(header::CONTENT_TYPE, "application/dns-message")], resp).into_response();
            }
        }
    }
    StatusCode::BAD_REQUEST.into_response()
}

// ========================
// DAEMON RUNNER
// ========================
pub async fn run_dns_server(state: Arc<DnsState>, config_arc: Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = Arc::new(DnsResolver::new(state.clone(), config_arc.clone()).await?);
    
    println!("Generating self-signed TLS certificates for DoT and DoH...");
    let alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string(), "rouman.router".to_string()];
    let cert = generate_simple_self_signed(alt_names).unwrap();
    
    let cert_der_bytes = cert.cert.der().to_vec();
    let key_der_bytes = cert.signing_key.serialize_der();
    
    let (sinkhole_enabled, udp_enabled, dot_enabled, doh_enabled) = {
        let conf = config_arc.read().await;
        (
            conf.dns.sinkhole_enabled,
            conf.dns.udp_enabled,
            conf.dns.dot_enabled,
            conf.dns.doh_enabled,
        )
    };

    if !sinkhole_enabled {
        println!("DNS Sinkhole is globally disabled.");
        return Ok(());
    }

    // 1. Spawning UDP DNS Task (Port 53)
    if udp_enabled {
        let resolver_udp = resolver.clone();
        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], 53));
            if let Ok(socket) = UdpSocket::bind(addr).await {
                let socket = Arc::new(socket);
                println!("[DNS] UDP Sinkhole listening on {}", addr);
                let mut buf = [0u8; 1500];
                loop {
                    match socket.recv_from(&mut buf).await {
                        Ok((size, src)) => {
                            let req_data = buf[..size].to_vec();
                            let r = resolver_udp.clone();
                            let s_clone = socket.clone();
                            tokio::spawn(async move {
                                if let Ok(resp) = r.resolve(&req_data).await {
                                    let _ = s_clone.send_to(&resp, src).await;
                                }
                            });
                        }
                        Err(_) => continue,
                    }
                }
            }
        });
    }

    // 2. Spawning TCP DoT Task (Port 853)
    if dot_enabled {
        let resolver_dot = resolver.clone();
        
        // Load certs
        let cert_der = CertificateDer::from(cert_der_bytes.clone());
        let key_der = PrivateKeyDer::try_from(key_der_bytes.clone()).unwrap();

        let mut server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)
            .unwrap();
        server_config.alpn_protocols = vec![b"dot".to_vec()]; 
        let tls_config_dot = Arc::new(server_config);

        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], 853));
            if let Ok(listener) = TcpListener::bind(addr).await {
                println!("[DNS] TLS (DoT) Sinkhole listening on {}", addr);
                let acceptor = TlsAcceptor::from(tls_config_dot);

                loop {
                    if let Ok((stream, _peer_addr)) = listener.accept().await {
                        let acceptor = acceptor.clone();
                        let r = resolver_dot.clone();
                        tokio::spawn(async move {
                            if let Ok(mut tls_stream) = acceptor.accept(stream).await {
                                let mut len_buf = [0u8; 2];
                                loop {
                                    if tls_stream.read_exact(&mut len_buf).await.is_err() { break; }
                                    let length = u16::from_be_bytes(len_buf) as usize;
                                    if length == 0 || length > 4096 { break; }
                                    
                                    let mut req_buf = vec![0u8; length];
                                    if tls_stream.read_exact(&mut req_buf).await.is_err() { break; }
                                    
                                    if let Ok(resp) = r.resolve(&req_buf).await {
                                        let resp_len = (resp.len() as u16).to_be_bytes();
                                        if tls_stream.write_all(&resp_len).await.is_err() { break; }
                                        if tls_stream.write_all(&resp).await.is_err() { break; }
                                    }
                                }
                            }
                        });
                    }
                }
            }
        });
    }

    // 3. Spawning DoH Server Task (Port 444 with Axum)
    if doh_enabled {
        let resolver_doh = resolver.clone();
        tokio::spawn(async move {
            let app = Router::new()
                .route("/dns-query", get(doh_get).post(doh_post))
                .with_state(resolver_doh);

            let rustls_config = RustlsConfig::from_der(
                vec![cert_der_bytes],
                key_der_bytes
            ).await.unwrap();

            let addr = SocketAddr::from(([0, 0, 0, 0], 444));
            println!("[DNS] HTTPS (DoH) Sinkhole listening on {}", addr);
            let _ = axum_server::bind_rustls(addr, rustls_config)
                .serve(app.into_make_service())
                .await;
        });
    }

    Ok(())
}
