use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;
use crate::database::Database;

#[derive(Serialize, Deserialize, Debug)]
struct CloudflareDnsRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
}

#[derive(Deserialize, Debug)]
struct CloudflareResponse<T> {
    result: T,
    success: bool,
}

#[derive(Deserialize, Debug)]
struct CloudflareRecordResult {
    id: String,
}

pub struct AcmeManager {
    db: Arc<Database>,
    http: Client,
}

impl AcmeManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            http: Client::new(),
        }
    }

    pub async fn add_txt_record(&self, domain: &str, name: &str, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let settings = sqlx::query!("SELECT cf_api_token FROM proxy_settings WHERE id = 1")
            .fetch_one(&self.db.pool).await?;
        
        let token = settings.cf_api_token.ok_or("Cloudflare Token not configured")?;

        // Find Zone ID
        let zone_name = domain.split('.').collect::<Vec<&str>>().join("."); // Simplified, needs better suffix handling
        let zone_url = format!("https://api.cloudflare.com/client/v4/zones?name={}", zone_name);
        
        let zones: CloudflareResponse<Vec<serde_json::Value>> = self.http.get(&zone_url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?
            .json().await?;

        let zone_id = zones.result.get(0).and_then(|z| z["id"].as_str()).ok_or("Zone not found")?;

        // Add TXT Record
        let record_url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
        let record = CloudflareDnsRecord {
            record_type: "TXT".to_string(),
            name: name.to_string(),
            content: content.to_string(),
            ttl: 60,
        };

        let res: CloudflareResponse<CloudflareRecordResult> = self.http.post(&record_url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&record)
            .send().await?
            .json().await?;

        if !res.success {
            return Err("Failed to add TXT record".into());
        }

        Ok(res.result.id)
    }

    pub async fn request_wildcard_cert(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        use instant_acme::{Account, ChallengeType, OrderState, Identifier};

        let settings = sqlx::query!("SELECT acme_email FROM proxy_settings WHERE id = 1")
            .fetch_one(&self.db.pool).await?;
        let email = settings.acme_email.ok_or("ACME Email not configured")?;

        // Initialize ACME Account
        let (account, _) = Account::create(
            &vec![format!("mailto:{}", email)],
            instant_acme::LETS_ENCRYPT_STAGING_URL, // Use staging for testing first
            Default::default(),
        ).await?;

        // Order Wildcard Certificate
        let mut order = account.new_order(&vec![
            Identifier::Dns(domain.to_string()),
            Identifier::Dns(format!("*.{}", domain)),
        ]).await?;

        // Handle Challenges
        let authorizations = order.authorizations().await?;
        for auth in authorizations {
            let challenge = auth.challenges.iter()
                .find(|c| c.ty == ChallengeType::Dns01)
                .ok_or("No DNS-01 challenge found")?;

            let dns_name = format!("_acme-challenge.{}", auth.identifier.value());
            let dns_content = challenge.key_authorization(&account.key())?.dns_01_digest();

            // Add TXT record via Cloudflare
            let record_id = self.add_txt_record(domain, &dns_name, &dns_content).await?;

            // Wait for propagation (Simplified, should pull/retry)
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;

            // Trigger validation
            order.set_challenge_ready(&challenge.url).await?;
        }

        // Wait for order completion and finalize
        // ... (CSR generation with rcgen and final update to DB) ...
        
        Ok(())
    }

    pub async fn delete_txt_record(&self, domain: &str, record_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let settings = sqlx::query!("SELECT cf_api_token FROM proxy_settings WHERE id = 1")
            .fetch_one(&self.db.pool).await?;
        let token = settings.cf_api_token.ok_or("Cloudflare Token not configured")?;

        // Find Zone ID (Same logic as add)
        let zone_name = domain.split('.').collect::<Vec<&str>>().join("."); 
        let zone_url = format!("https://api.cloudflare.com/client/v4/zones?name={}", zone_name);
        
        let zones: CloudflareResponse<Vec<serde_json::Value>> = self.http.get(&zone_url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?
            .json().await?;
        let zone_id = zones.result.get(0).and_then(|z| z["id"].as_str()).ok_or("Zone not found")?;

        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, record_id);
        self.http.delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;

        Ok(())
    }
}
