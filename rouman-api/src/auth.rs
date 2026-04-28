use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use sqlx::Row;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use crate::error::{AppError, AppResult};

fn get_jwt_secret() -> Vec<u8> {
    let machine_id = std::fs::read_to_string("/etc/machine-id").unwrap_or_else(|_| "rouman-default-api-secret".to_string());
    let mut hasher = sha2::Sha256::new();
    hasher.update(machine_id.trim().as_bytes());
    hasher.update(b"ROUMAN_JWT_SALT_V1");
    hasher.finalize().to_vec()
}

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

async fn login_handler(
    State(state): State<crate::AppState>,
    jar: CookieJar, 
    Json(payload): Json<AuthPayload>
) -> AppResult<(CookieJar, Json<AuthResponse>)> {
    // Query database for user
    let user = sqlx::query("SELECT password_hash, salt FROM system_admins WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if let Some(row) = user {
        let stored_hash: String = row.get("password_hash");
        let salt: String = row.get("salt");

        // Verify password
        let mut hasher = sha2::Sha256::new();
        sha2::Digest::update(&mut hasher, payload.password.as_bytes());
        sha2::Digest::update(&mut hasher, salt.as_bytes());
        let calculated_hash = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();

        if calculated_hash == stored_hash {
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
                &EncodingKey::from_secret(&get_jwt_secret()),
            ).map_err(|e| AppError::Internal(format!("Token creation failed: {}", e)))?;

            // Set HttpOnly Cookie
            let cookie = Cookie::build(("auth_token", token))
                .path("/")
                .http_only(true)
                .secure(false) // Set to true if using HTTPS
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .build();

            return Ok((
                jar.add(cookie),
                Json(AuthResponse { message: "Login successful".to_owned() }),
            ));
        }
    }

    Err(AppError::Unauthorized("Invalid username or password".to_owned()))
}

async fn logout_handler(jar: CookieJar) -> (CookieJar, Json<AuthResponse>) {
    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(time::Duration::ZERO)
        .build();

    (
        jar.add(cookie),
        Json(AuthResponse { message: "Logged out".to_owned() })
    )
}

async fn me_handler(claims: Claims) -> Json<MeResponse> {
    Json(MeResponse {
        username: claims.sub,
    })
}

// Extractor to parse JWT Claims directly in handler arguments 
impl<S> axum::extract::FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let cookie_header = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()).unwrap_or("");
        
        let token = cookie_header
            .split(';')
            .find(|pair| pair.trim().starts_with("auth_token="))
            .unwrap_or("")
            .trim()
            .strip_prefix("auth_token=")
            .unwrap_or("");

        if token.is_empty() {
            return Err(AppError::Unauthorized("Session expired or missing".to_owned()));
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&get_jwt_secret()),
            &Validation::default(),
        ).map_err(|_| AppError::Unauthorized("Invalid session token".to_owned()))?;

        Ok(token_data.claims)
    }
}

pub async fn auth_middleware(req: Request, next: Next) -> AppResult<Response> {
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
        return Err(AppError::Unauthorized("Authentication required".to_owned()));
    }

    if decode::<Claims>(
        token,
        &DecodingKey::from_secret(&get_jwt_secret()),
        &Validation::default(),
    ).is_err() {
        return Err(AppError::Unauthorized("Invalid session".to_owned()));
    }

    Ok(next.run(req).await)
}
