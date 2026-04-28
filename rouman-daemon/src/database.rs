use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use sha2::Digest;

#[derive(Debug)]
pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        // Ensure directory exists for standard FHS
        std::fs::create_dir_all("/var/lib/rouman").ok();
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .after_connect(|conn, _meta| Box::pin(async move {
                sqlx::query("PRAGMA journal_mode=WAL;").execute(&mut *conn).await?;
                sqlx::query("PRAGMA synchronous=NORMAL;").execute(&mut *conn).await?;
                sqlx::query("PRAGMA busy_timeout=5000;").execute(&mut *conn).await?;
                Ok(())
            }))
            .connect(db_url)
            .await?;

        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // 1. Profil (Kumpulan limitasi)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS radius_profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                upload_mbps INTEGER DEFAULT 0,
                download_mbps INTEGER DEFAULT 0,
                uptime_limit_sec INTEGER DEFAULT 0,
                data_limit_mb INTEGER DEFAULT 0
            )"
        ).execute(&self.pool).await?;

        // 2. User Data / Voucher
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS radius_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                profile_id INTEGER,
                is_voucher BOOLEAN DEFAULT FALSE,
                status TEXT DEFAULT 'active',
                FOREIGN KEY(profile_id) REFERENCES radius_profiles(id)
            )"
        ).execute(&self.pool).await?;

        // 3. Aktivitas Sesi (Accounting)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS radius_sessions (
                session_id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                mac_address TEXT NOT NULL,
                ip_address TEXT,
                start_time DATETIME DEFAULT CURRENT_TIMESTAMP,
                bytes_in INTEGER DEFAULT 0,
                bytes_out INTEGER DEFAULT 0,
                active BOOLEAN DEFAULT TRUE
            )"
        ).execute(&self.pool).await?;

        // 4. Proxy Global Settings
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS proxy_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                acme_email TEXT,
                cf_api_token TEXT
            )"
        ).execute(&self.pool).await?;

        // 5. Proxy Hosts (Virtual Hosts)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS proxy_hosts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT UNIQUE NOT NULL,
                upstream_url TEXT NOT NULL,
                ssl_enabled BOOLEAN DEFAULT TRUE,
                node_id TEXT DEFAULT 'local',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(&self.pool).await?;

        // Migration: Add node_id if it doesn't exist
        let _ = sqlx::query("ALTER TABLE proxy_hosts ADD COLUMN node_id TEXT DEFAULT 'local'").execute(&self.pool).await;

        // 6. SSL Certificates Storage
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS proxy_certificates (
                domain TEXT PRIMARY KEY,
                fullchain TEXT NOT NULL,
                privkey TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(&self.pool).await?;

        // 7. Cloudflare Tunnels (Zero Trust)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS cloudflare_tunnels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                token TEXT NOT NULL,
                enabled BOOLEAN DEFAULT FALSE,
                status TEXT DEFAULT 'offline',
                last_error TEXT,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(&self.pool).await?;

        // 8. System Notifications
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                type TEXT NOT NULL,
                is_read BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(&self.pool).await?;

        // 9. System Administrators
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS system_admins (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(&self.pool).await?;

        // 10. DHCP Leases Persistence
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS dhcp_leases (
                mac TEXT PRIMARY KEY,
                ip TEXT NOT NULL,
                hostname TEXT,
                expires_at INTEGER NOT NULL
            )"
        ).execute(&self.pool).await?;

        // Seed default admin if empty
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM system_admins")
            .fetch_one(&self.pool)
            .await?;

        if count == 0 {
            // Default: admin:admin
            // Kita gunakan salt konstan untuk admin pertama agar mudah didefinisikan di sini,
            // namun admin harus segera mengganti passwordnya setelah login pertama.
            let salt = "ROUMAN_INIT_SALT"; 
            let mut hasher = sha2::Sha256::new();
            sha2::Digest::update(&mut hasher, "admin".as_bytes());
            sha2::Digest::update(&mut hasher, salt.as_bytes());
            let hash = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();

            sqlx::query("INSERT INTO system_admins (username, password_hash, salt) VALUES ('admin', ?, ?)")
                .bind(hash)
                .bind(salt)
                .execute(&self.pool)
                .await?;
        }

        // Tambahkan profil default jika kosong
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO radius_profiles (name, upload_mbps, download_mbps) VALUES ('default', 2, 5)"
        ).execute(&self.pool).await?;

        Ok(())
    }
    pub async fn add_notification(&self, title: &str, message: &str, n_type: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO notifications (title, message, type) VALUES (?, ?, ?)")
            .bind(title)
            .bind(message)
            .bind(n_type)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_dhcp_lease(&self, mac: &str, ip: &str, hostname: Option<&str>, expires_at: i64) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO dhcp_leases (mac, ip, hostname, expires_at) VALUES (?, ?, ?, ?)")
            .bind(mac)
            .bind(ip)
            .bind(hostname)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_active_dhcp_leases(&self) -> Result<Vec<(String, String, Option<String>, i64)>, sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
            
        let rows = sqlx::query("SELECT mac, ip, hostname, expires_at FROM dhcp_leases WHERE expires_at > ?")
            .bind(now)
            .fetch_all(&self.pool)
            .await?;
            
        let mut result = Vec::new();
        for row in rows {
            use sqlx::Row;
            result.push((
                row.get::<String, _>("mac"),
                row.get::<String, _>("ip"),
                row.get::<Option<String>, _>("hostname"),
                row.get::<i64, _>("expires_at"),
            ));
        }
        Ok(result)
    }
}
