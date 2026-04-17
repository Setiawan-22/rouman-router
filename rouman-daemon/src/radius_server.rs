use std::sync::Arc;
use radius_server::dictionary::Dictionary;
use radius_server::packet::{RadiusAttribute};
use crate::database::Database;
use md5::{Md5, Digest};

pub async fn start_radius_server(db: Arc<Database>) {
    let dict = Arc::new(Dictionary::load_embedded().unwrap());
    // TODO: Ambil dari config/database di masa depan
    let secret = "rouman-secret";
    let addr = "0.0.0.0:1812";

    println!("Starting RADIUS Server on {}", addr);

    let _ = radius_server::serve_async(addr, dict, secret, move |packet| {
        let db = db.clone();
        let shared_secret = secret.to_string();
        async move {
            let username = packet.username().unwrap_or_default();
            
            // 1. Ambil User-Password attribute (Type 2)
            let obfuscated_password = packet.attributes.iter()
                .find(|a| a.typ == 2)
                .map(|a| a.value.clone())
                .unwrap_or_default();

            if obfuscated_password.is_empty() {
                return Ok(packet.reply_reject("Missing User-Password attribute"));
            }

            // 2. Dekripsi PAP (RFC 2865)
            // P1 = C1 ^ MD5(S + RA)
            // P2 = C2 ^ MD5(S + C1) ...
            let mut decrypted_password = Vec::new();
            let mut last_c = packet.authenticator.to_vec();

            for chunk in obfuscated_password.chunks(16) {
                let mut hasher = Md5::new();
                hasher.update(shared_secret.as_bytes());
                hasher.update(&last_c);
                let b = hasher.finalize();
                
                for (i, &byte) in chunk.iter().enumerate() {
                    decrypted_password.push(byte ^ b[i]);
                }
                last_c = chunk.to_vec();
            }

            // Hapus padding null bytes jika ada
            let password = String::from_utf8_lossy(&decrypted_password)
                .trim_matches(char::from(0))
                .to_string();

            // 3. Verifikasi Basis Data
            let user = sqlx::query(
                "SELECT password, profile_id FROM radius_users WHERE username = ?"
            ).bind(&username)
             .fetch_optional(&db.pool).await.unwrap_or(None);

            if let Some(user) = user {
                use sqlx::Row;
                let user_password = user.get::<String, _>("password");
                if user_password == password {
                    println!("RADIUS Auth Success: {}", username);
                    return Ok(packet.reply_accept(vec![
                        RadiusAttribute::reply_message("Welcome to Rouman OS"),
                    ]));
                    
                }
            }

            println!("RADIUS Auth Failed: {}", username);
            Ok(packet.reply_reject("Invalid username or password"))
        }
    }).await;
}
