use std::sync::Arc;
use std::net::SocketAddr;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::TokioExecutor;
use hyper_util::server::conn::auto::Builder;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use rustls::server::{ResolvesServerCert, ClientHello};
use rustls::sign::CertifiedKey;
use crate::database::Database;
use sqlx;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tokio_rustls::TlsAcceptor;
use rustls_pemfile;

#[derive(Debug)]
pub struct ProxyCertResolver {
    pub cache: Arc<RwLock<HashMap<String, Arc<CertifiedKey>>>>,
}

impl ResolvesServerCert for ProxyCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        let domain = client_hello.server_name()?;
        let cache = self.cache.blocking_read();
        cache.get(domain).cloned()
    }
}

pub struct ProxyState {
    pub db: Arc<Database>,
    pub client: Client<HttpConnector, Incoming>,
    pub cert_cache: Arc<RwLock<HashMap<String, Arc<CertifiedKey>>>>,
}

pub async fn run_proxy_server(db: Arc<Database>) {
    let cert_cache = Arc::new(RwLock::new(HashMap::new()));
    let state = Arc::new(ProxyState {
        db: db.clone(),
        client: Client::builder(TokioExecutor::new()).build(HttpConnector::new()),
        cert_cache: cert_cache.clone(),
    });

    // Background task to sync certs from DB
    let db_sync = db.clone();
    let cache_sync = cert_cache.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(rows) = sqlx::query("SELECT domain, fullchain, privkey FROM proxy_certificates")
                .fetch_all(&db_sync.pool).await 
            {
                let mut new_cache = HashMap::new();
                for row in rows {
                    use sqlx::Row;
                    let domain: String = row.get("domain");
                    let fullchain: String = row.get("fullchain");
                    let privkey: String = row.get("privkey");

                    // Parse certs
                    let certs: Vec<_> = rustls_pemfile::certs(&mut fullchain.as_bytes())
                        .filter_map(|c| c.ok())
                        .collect();
                    let key = rustls_pemfile::private_key(&mut privkey.as_bytes())
                        .ok().flatten();

                    if let (Some(k), false) = (key, certs.is_empty()) {
                        if let Ok(signing_key) = rustls::crypto::ring::sign::any_supported_type(&k) {
                            let certified_key = CertifiedKey::new(certs, signing_key);
                            new_cache.insert(domain, Arc::new(certified_key));
                        }
                    }
                }
                let mut cache = cache_sync.write().await;
                *cache = new_cache;
            }
            tokio::time::sleep(std::time::Duration::from_secs(300)).await;
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 443));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind proxy port 443: {}", e);
            return;
        }
    };
    
    // SSL Config
    let cert_resolver = Arc::new(ProxyCertResolver { cache: cert_cache.clone() });
    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(cert_resolver);
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

    println!("Reverse Proxy HTTPS (SSL Terminated) listening on {}", addr);

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };

        let state_clone = state.clone();
        let acceptor = tls_acceptor.clone();
        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    let service = hyper::service::service_fn(move |req| {
                        handle_request(req, state_clone.clone(), peer_addr)
                    });

                    if let Err(err) = Builder::new(TokioExecutor::new())
                        .serve_connection(hyper_util::rt::TokioIo::new(tls_stream), service)
                        .await
                    {
                        eprintln!("Error serving TLS connection for {}: {:?}", peer_addr, err);
                    }
                }
                Err(e) => {
                    eprintln!("TLS Handshake error from {}: {:?}", peer_addr, e);
                }
            }
        });
    }
}

async fn handle_request(
    mut req: Request<Incoming>,
    state: Arc<ProxyState>,
    peer_addr: SocketAddr,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let host = req.headers().get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();

    if host.is_empty() {
        return Ok(Response::builder()
            .status(400)
            .body(Full::new(Bytes::from("Missing Host header")))
            .unwrap());
    }

    // 1. Cari upstream di database
    let upstream = sqlx::query(
        "SELECT upstream_url FROM proxy_hosts WHERE domain = ? AND ssl_enabled = 1"
    ).bind(&host)
     .fetch_optional(&state.db.pool).await.ok().flatten();

    if let Some(row) = upstream {
        use sqlx::Row;
        let upstream_url: String = row.get("upstream_url");
        
        // 2. Siapkan URI untuk forward
        let path_query = req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("/");
        let uri_string = format!("{}{}", upstream_url.trim_end_matches('/'), path_query);
        
        let uri = match uri_string.parse::<hyper::Uri>() {
            Ok(u) => u,
            Err(_) => return Ok(Response::builder()
                .status(500)
                .body(Full::new(Bytes::from("Invalid upstream URI")))
                .unwrap()),
        };
        
        *req.uri_mut() = uri;

        // 3. Tambahkan proxy headers
        if let Ok(ip) = peer_addr.ip().to_string().parse() {
            req.headers_mut().insert("X-Forwarded-For", ip);
        }
        if let Ok(ip) = peer_addr.ip().to_string().parse() {
            req.headers_mut().insert("X-Real-IP", ip);
        }
        if let Ok(proto) = "https".parse() {
            req.headers_mut().insert("X-Forwarded-Proto", proto);
        }

        // 4. Lakukan request ke upstream
        match state.client.request(req).await {
            Ok(res) => {
                let (parts, body) = res.into_parts();
                match body.collect().await {
                    Ok(collected) => Ok(Response::from_parts(parts, Full::new(collected.to_bytes()))),
                    Err(_) => Ok(Response::builder()
                        .status(502)
                        .body(Full::new(Bytes::from("Error reading upstream body")))
                        .unwrap()),
                }
            },
            Err(e) => {
                eprintln!("Proxy upstream error for {}: {}", host, e);
                Ok(Response::builder()
                    .status(502)
                    .body(Full::new(Bytes::from("Bad Gateway")))
                    .unwrap())
            }
        }
    } else {
        Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("Domain not configured or SSL disabled")))
            .unwrap())
    }
}
