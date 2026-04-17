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
struct DnsDohState {
    dns: Arc<DnsState>,
    config: Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>>,
}

impl FromRef<DnsDohState> for Arc<DnsState> {
    fn from_ref(state: &DnsDohState) -> Self {
        state.dns.clone()
    }
}

impl FromRef<DnsDohState> for Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>> {
    fn from_ref(state: &DnsDohState) -> Self {
        state.config.clone()
    }
}

async fn process_dns_query(request_data: &[u8], state: Arc<DnsState>, config_arc: Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>>) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut decoder = BinDecoder::new(request_data);
    let request = match Message::read(&mut decoder) {
        Ok(msg) => msg,
        Err(e) => {
            return Err(e.into());
        }
    };

    if let Some(query) = request.queries().first() {
        let name = query.name().to_string();
        let clean_name = name.trim_end_matches('.').to_string();
        
        let is_blocked = {
            let blocked = state.blocked_domains.lock().await;
            blocked.contains(&clean_name)
        };

        if is_blocked {
            println!("Blocked DNS query: {}", clean_name);
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

    // Ambil konfigurasi DNS
    let (doh_enabled, doh_url) = {
        let conf = config_arc.read().await;
        (conf.dns.doh_enabled, conf.dns.doh_url.clone())
    };

    if doh_enabled && !doh_url.is_empty() {
        let client = reqwest::Client::new();
        let res = client.post(&doh_url)
            .header("Content-Type", "application/dns-message")
            .body(request_data.to_vec())
            .send()
            .await?;

        if res.status().is_success() {
            let bytes = res.bytes().await?;
            return Ok(bytes.to_vec());
        }
    }

    // Fallback to upstream UDP (Legacy)
    let upstream_addr = SocketAddr::from(([8, 8, 8, 8], 53));
    let upstream_socket = UdpSocket::bind("0.0.0.0:0").await?;
    upstream_socket.send_to(request_data, upstream_addr).await?;

    let mut upstream_buf = [0u8; 1500];
    match tokio::time::timeout(std::time::Duration::from_secs(2), upstream_socket.recv_from(&mut upstream_buf)).await {
        Ok(Ok((resp_size, _))) => {
            Ok(upstream_buf[..resp_size].to_vec())
        }
        Ok(Err(e)) => Err(e.into()),
        Err(e) => Err(e.into()),
    }
}

// ========================
// DoH HANDLERS
// ========================
async fn doh_post(
    State(state): State<Arc<DnsState>>,
    State(config): State<Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>>>,
    body: Bytes,
) -> impl IntoResponse {
    match process_dns_query(&body, state, config).await {
        Ok(resp) => {
            (
                [(header::CONTENT_TYPE, "application/dns-message")],
                resp
            ).into_response()
        }
        Err(_) => StatusCode::BAD_REQUEST.into_response()
    }
}

async fn doh_get(
    State(state): State<Arc<DnsState>>,
    State(config): State<Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let process = |b64: &str| -> Result<Vec<u8>, ()> {
        let mut b64_clean = b64.to_string();
        // Coba decoding fallback untuk standard / param-driven curl
        if let Ok(decoded) = URL_SAFE_NO_PAD.decode(&b64_clean) {
            return Ok(decoded);
        }
        b64_clean = b64_clean.replace("=", "");
        URL_SAFE_NO_PAD.decode(&b64_clean).map_err(|_| ())
    };

    if let Some(dns) = params.get("dns") {
        if let Ok(query_bytes) = process(dns) {
            if let Ok(resp) = process_dns_query(&query_bytes, state, config).await {
                return (
                    [(header::CONTENT_TYPE, "application/dns-message")],
                    resp
                ).into_response();
            }
        }
    }
    StatusCode::BAD_REQUEST.into_response()
}

// ========================
// DAEMON RUNNER
// ========================
pub async fn run_dns_server(state: Arc<DnsState>, config_arc: Arc<tokio::sync::RwLock<rouman_api::config::RoumanConfig>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Generating self-signed TLS certificates for DoT and DoH...");
    let alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string(), "rouman.router".to_string()];
    let cert = generate_simple_self_signed(alt_names).unwrap();
    
    let cert_der_bytes = cert.cert.der().to_vec();
    let key_der_bytes = cert.signing_key.serialize_der();
    
    let cert_der = CertificateDer::from(cert_der_bytes.clone());
    let key_der = PrivateKeyDer::try_from(key_der_bytes.clone()).unwrap();

    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der.clone()], key_der)
        .unwrap();
    server_config.alpn_protocols = vec![b"dot".to_vec()]; // For DoT
    let tls_config_dot = Arc::new(server_config);

    // 1. Spawning UDP DNS Task (Port 53)
    let state_udp = state.clone();
    let config_udp = config_arc.clone();
    tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], 53));
        if let Ok(socket) = UdpSocket::bind(addr).await {
            println!("[DNS] UDP Sinhkole listening on {}", addr);
            let mut buf = [0u8; 1500];
            loop {
                if let Ok((size, src)) = socket.recv_from(&mut buf).await {
                    let req_data = buf[..size].to_vec();
                    let st = state_udp.clone();
                    let cf = config_udp.clone();
                    if let Ok(resp) = process_dns_query(&req_data, st, cf).await {
                        let _ = socket.send_to(&resp, src).await;
                    }
                }
            }
        }
    });

    // 2. Spawning TCP DoT Task (Port 853)
    let state_dot = state.clone();
    let config_dot = config_arc.clone();
    tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], 853));
        let listener = TcpListener::bind(addr).await.unwrap();
        println!("[DNS] TLS (DoT) Sinkhole listening on {}", addr);
        let acceptor = TlsAcceptor::from(tls_config_dot);

        loop {
            if let Ok((stream, _peer_addr)) = listener.accept().await {
                let acceptor = acceptor.clone();
                let st = state_dot.clone();
                let cf = config_dot.clone();
                tokio::spawn(async move {
                    if let Ok(mut tls_stream) = acceptor.accept(stream).await {
                        let mut len_buf = [0u8; 2];
                        loop {
                            if tls_stream.read_exact(&mut len_buf).await.is_err() { break; }
                            let length = u16::from_be_bytes(len_buf) as usize;
                            if length == 0 || length > 4096 { break; }
                            
                            let mut req_buf = vec![0u8; length];
                            if tls_stream.read_exact(&mut req_buf).await.is_err() { break; }
                            
                            if let Ok(resp) = process_dns_query(&req_buf, st.clone(), cf.clone()).await {
                                let resp_len = (resp.len() as u16).to_be_bytes();
                                if tls_stream.write_all(&resp_len).await.is_err() { break; }
                                if tls_stream.write_all(&resp).await.is_err() { break; }
                            }
                        }
                    }
                });
            }
        }
    });

    // 3. Spawning DoH Server Task (Port 443 with Axum)
    let state_doh = state.clone();
    let config_doh = config_arc.clone();
    tokio::spawn(async move {
        let doh_state = DnsDohState { dns: state_doh, config: config_doh };

        let app = Router::new()
            .route("/dns-query", get(doh_get).post(doh_post))
            .with_state(doh_state);

        let rustls_config = RustlsConfig::from_der(
            vec![cert_der_bytes],
            key_der_bytes
        ).await.unwrap();

        let addr = SocketAddr::from(([0, 0, 0, 0], 443));
        println!("[DNS] HTTPS (DoH) Sinkhole listening on {}", addr);
        axum_server::bind_rustls(addr, rustls_config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(())
}
