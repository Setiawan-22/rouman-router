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

#[derive(Debug)]
pub struct ProxyCertResolver {
    pub db: Arc<Database>,
}

impl ResolvesServerCert for ProxyCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        let _domain = client_hello.server_name()?;
        
        // TODO: Implementasi caching sertifikat dari database.
        // Karena trait ini adalah sync, kita perlu menggunakan cache internal
        // yang diupdate oleh background worker.
        None 
    }
}

pub struct ProxyState {
    pub db: Arc<Database>,
    pub client: Client<HttpConnector, Incoming>,
}

pub async fn run_proxy_server(db: Arc<Database>) {
    let state = Arc::new(ProxyState {
        db: db.clone(),
        client: Client::builder(TokioExecutor::new()).build(HttpConnector::new()),
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 443));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind proxy port 443: {}", e);
            return;
        }
    };
    
    println!("Reverse Proxy HTTPS listening on {}", addr);

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };

        let state_clone = state.clone();
        tokio::spawn(async move {
            let service = hyper::service::service_fn(move |req| {
                handle_request(req, state_clone.clone(), peer_addr)
            });

            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                .await
            {
                eprintln!("Error serving connection for {}: {:?}", peer_addr, err);
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
        req.headers_mut().insert("X-Forwarded-For", peer_addr.ip().to_string().parse().unwrap());
        req.headers_mut().insert("X-Real-IP", peer_addr.ip().to_string().parse().unwrap());
        req.headers_mut().insert("X-Forwarded-Proto", "https".parse().unwrap());

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
