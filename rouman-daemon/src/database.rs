use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::Arc;

#[derive(Debug)]
pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        // Buat folder jika belum ada (asumsi /root/.rouman/ atau /opt/rouman/data/)
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .after_connect(|conn, _meta| Box::pin(async move {
                sqlx::query("PRAGMA journal_mode=WAL;").execute(&mut *conn).await?;
                sqlx::query("PRAGMA synchronous=NORMAL;").execute(&mut *conn).await?;
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
}
