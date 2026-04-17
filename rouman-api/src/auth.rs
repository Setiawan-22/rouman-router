use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

const JWT_SECRET: &[u8] = b"super_secret_mandala_key_change_me_later";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub username: String,
}

pub fn auth_routes() -> Router<crate::AppState> {
    Router::<crate::AppState>::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/me", get(me_handler).route_layer(axum::middleware::from_fn(auth_middleware)))
}

async fn login_handler(jar: CookieJar, Json(payload): Json<AuthPayload>) -> Result<(CookieJar, Json<AuthResponse>), StatusCode> {
    // Hardcoded Dummy Authenticate
    if payload.username == "admin" && payload.password == "admin" {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: payload.username.to_owned(),
            exp: expiration,
        };

        // Create token
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Set HttpOnly Cookie
        let cookie = Cookie::build(("auth_token", token))
            .path("/")
            .http_only(true)
            .secure(false) // Set to true if using HTTPS
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .build();

        Ok((
            jar.add(cookie),
            Json(AuthResponse { message: "Login successful".to_owned() }),
        ))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn logout_handler(jar: CookieJar) -> (CookieJar, Json<AuthResponse>) {
    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .http_only(true)
        .build();

    (
        jar.remove(cookie),
        Json(AuthResponse { message: "Logged out".to_owned() })
    )
}

async fn me_handler(claims: Claims) -> Json<MeResponse> {
    Json(MeResponse {
        username: claims.sub,
    })
}

// Extractor to parse JWT Claims directly in handler arguments 
// (Used implicitly by the auth_middleware underneath)
impl<S> axum::extract::FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;

        // In a real app we might grab the CookieJar from Request parts,
        // but it's simpler to parse the exact cookie header manually or rely on the Jar extractor.
        // For here, since we check cookie directly:
        let cookie_header = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()).unwrap_or("");
        
        let token = cookie_header
            .split(';')
            .find(|pair| pair.trim().starts_with("auth_token="))
            .unwrap_or("")
            .trim()
            .strip_prefix("auth_token=")
            .unwrap_or("");

        if token.is_empty() {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        ).map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(token_data.claims)
    }
}

pub async fn auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    // If request contains valid token (extracted by Claims via from_request_parts simulation here)
    // Actually we can simply use the header parser logic from above.
    let headers = req.headers().clone();
    let cookie_header = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()).unwrap_or("");
    let token = cookie_header
        .split(';')
        .find(|pair| pair.trim().starts_with("auth_token="))
        .unwrap_or("")
        .trim()
        .strip_prefix("auth_token=")
        .unwrap_or("");

    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    ).is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}
