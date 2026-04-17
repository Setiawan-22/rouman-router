use rhai::{Engine, EvalAltResult, Scope, AST};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use rouman_api::config::RoumanConfig;
use std::process::Command;

pub struct AutomationManager {
    engine: Engine,
}

impl AutomationManager {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register custom functions for the script
        engine.register_fn("log", |msg: &str| {
            println!("[SCRIPT] {}", msg);
        });

        engine.register_fn("exec", |cmd: &str| -> String {
            println!("[SCRIPT] Executing: {}", cmd);
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output();
            
            match output {
                Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                Err(e) => format!("Error: {}", e),
            }
        });

        engine.register_fn("ping", |host: &str| -> bool {
            let status = Command::new("ping")
                .arg("-c")
                .arg("1")
                .arg("-W")
                .arg("1")
                .arg(host)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            
            match status {
                Ok(s) => s.success(),
                Err(_) => false,
            }
        });

        Self { engine }
    }

    pub async fn run(&self, config_arc: Arc<RwLock<RoumanConfig>>) {
        let mut last_netwatch_states: std::collections::HashMap<String, bool> = std::collections::HashMap::new();

        loop {
            let config = {
                let guard = config_arc.read().await;
                guard.automation.clone()
            };

            // 1. Process Netwatch Rules
            for rule in config.netwatch {
                if !rule.enabled { continue; }

                let is_up = self.check_ping(&rule.target).await;
                let last_state = last_netwatch_states.get(&rule.name).cloned();

                if let Some(prev) = last_state {
                    if is_up && !prev {
                        // Changed from Down to Up
                        println!("[NETWATCH] {} is UP. Running UP script.", rule.name);
                        let _ = self.execute_script(&rule.script_up).await;
                    } else if !is_up && prev {
                        // Changed from Up to Down
                        println!("[NETWATCH] {} is DOWN. Running DOWN script.", rule.name);
                        let _ = self.execute_script(&rule.script_down).await;
                    }
                }
                last_netwatch_states.insert(rule.name.clone(), is_up);
            }

            // 2. Process Scheduler Rules (Simplistic for now: every loop check)
            // Note: In a real implementation, we'd use timestamps.
            
            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn check_ping(&self, target: &str) -> bool {
        let target = target.to_string();
        tokio::task::spawn_blocking(move || {
            let status = Command::new("ping")
                .arg("-c")
                .arg("1")
                .arg("-W")
                .arg("1")
                .arg(&target)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            
            match status {
                Ok(s) => s.success(),
                Err(_) => false,
            }
        }).await.unwrap_or(false)
    }

    pub async fn execute_script(&self, script: &str) -> Result<(), Box<rhai::EvalAltResult>> {
        if script.trim().is_empty() { return Ok(()); }
        
        let mut scope = Scope::new();
        let engine = &self.engine;
        
        engine.run_with_scope(&mut scope, script)?;
        Ok(())
    }
}
