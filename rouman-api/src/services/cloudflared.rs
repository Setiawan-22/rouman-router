use tokio::sync::Mutex;
use tokio::process::{Child, Command};
use std::process::Stdio;

// Global Supervisor State
static CLOUDFLARED_PROCESS: Mutex<Option<Child>> = Mutex::const_new(None);

#[derive(Debug)]
pub enum SupervisorError {
    AlreadyRunning,
    NotRunning,
    FailedToSpawn(String),
    FailedToKill,
}

pub async fn start(token: &str) -> Result<(), SupervisorError> {
    let mut process = CLOUDFLARED_PROCESS.lock().await;

    // Handle stale exited process before attempting spawn
    if let Some(ref mut child) = *process {
        if let Ok(Some(_)) = child.try_wait() {
            *process = None; // Process exited, clean slate
        } else {
            return Err(SupervisorError::AlreadyRunning); // Still active
        }
    }

    let child = Command::new("cloudflared")
        .arg("tunnel")
        .arg("--no-autoupdate")
        .arg("run")
        .arg("--token")
        .arg(token)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| SupervisorError::FailedToSpawn(e.to_string()))?;

    *process = Some(child);
    Ok(())
}

pub async fn stop() -> Result<(), SupervisorError> {
    let mut process = CLOUDFLARED_PROCESS.lock().await;
    
    if let Some(mut child) = process.take() {
        if child.kill().await.is_err() {
            return Err(SupervisorError::FailedToKill);
        }
        Ok(())
    } else {
        Err(SupervisorError::NotRunning)
    }
}

pub async fn status() -> bool {
    let mut process = CLOUDFLARED_PROCESS.lock().await;
    
    if let Some(ref mut child) = *process {
         if let Ok(Some(_)) = child.try_wait() {
             *process = None;
             false
         } else {
             true
         }
    } else {
        false
    }
}
