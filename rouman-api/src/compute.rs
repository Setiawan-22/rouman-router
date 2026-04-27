use axum::{routing::{get, post}, Router, Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
pub struct VmInstance {
    pub id: String,
    pub vcpu: u32,
    pub ram: u32,
    pub image: String,
    pub status: String,
    pub uptime: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerInstance {
    pub id: String,
    pub image: String,
    pub status: String,
    pub ip: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StackInstance {
    pub name: String,
    pub config: String,
    pub service_count: u32,
    pub status: String,
    pub uptime: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StorageMount {
    pub protocol: String,
    pub remote: String,
    pub local: String,
    pub options: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub id: String,
    pub hostname: String,
    pub ip: String,
    pub status: String, // "Online", "Offline"
    pub is_local: bool,
}

pub struct ComputeState {
    pub nodes: Mutex<Vec<ComputeNode>>,
    pub vms: Mutex<Vec<VmInstance>>,
    pub containers: Mutex<Vec<ContainerInstance>>,
    pub stacks: Mutex<Vec<StackInstance>>,
    pub storage: Mutex<Vec<StorageMount>>,
}

impl Default for ComputeState {
    fn default() -> Self {
        Self {
            nodes: Mutex::new(vec![
                ComputeNode {
                    id: "local".to_string(),
                    hostname: "Rouman-Primary".to_string(),
                    ip: "127.0.0.1".to_string(),
                    status: "Online".to_string(),
                    is_local: true,
                }
            ]),
            vms: Mutex::new(vec![]),
            containers: Mutex::new(vec![]),
            stacks: Mutex::new(vec![]),
            storage: Mutex::new(vec![]),
        }
    }
}

// Global in-memory state for compute (since we don't have a DB yet for this)
lazy_static::lazy_static! {
    static ref COMPUTE_STORAGE: Arc<ComputeState> = Arc::new(ComputeState::default());
}

async fn get_vms() -> Json<Value> {
    let vms = COMPUTE_STORAGE.vms.lock().await;
    Json(serde_json::json!({ "vms": *vms }))
}

async fn post_vm(Json(payload): Json<VmInstance>) -> Json<Value> {
    let mut vms = COMPUTE_STORAGE.vms.lock().await;
    let mut new_vm = payload;
    new_vm.status = "Running".to_string();
    new_vm.uptime = "0s".to_string();
    vms.push(new_vm);
    Json(serde_json::json!({ "status": "success" }))
}

async fn get_containers() -> Json<Value> {
    let containers = COMPUTE_STORAGE.containers.lock().await;
    Json(serde_json::json!({ "containers": *containers }))
}

async fn post_container(Json(payload): Json<ContainerInstance>) -> Json<Value> {
    let mut containers = COMPUTE_STORAGE.containers.lock().await;
    let mut new_ct = payload;
    new_ct.status = "Running".to_string();
    new_ct.ip = format!("10.88.0.{}", containers.len() + 2);
    containers.push(new_ct);
    Json(serde_json::json!({ "status": "success" }))
}

async fn get_stacks() -> Json<Value> {
    let stacks = COMPUTE_STORAGE.stacks.lock().await;
    Json(serde_json::json!({ "stacks": *stacks }))
}

async fn post_stack(Json(payload): Json<StackInstance>) -> Json<Value> {
    let mut stacks = COMPUTE_STORAGE.stacks.lock().await;
    let mut new_stack = payload;
    new_stack.status = "Active".to_string();
    new_stack.uptime = "0s".to_string();
    new_stack.service_count = 2; // Simulated count from YAML
    stacks.push(new_stack);
    Json(serde_json::json!({ "status": "success" }))
}

async fn get_storage() -> Json<Value> {
    let storage = COMPUTE_STORAGE.storage.lock().await;
    Json(serde_json::json!({ "mounts": *storage }))
}

async fn post_storage(Json(payload): Json<StorageMount>) -> Json<Value> {
    let mut storage = COMPUTE_STORAGE.storage.lock().await;
    storage.push(payload);
    Json(serde_json::json!({ "status": "success" }))
}

pub async fn get_vms_internal() -> Vec<VmInstance> {
    COMPUTE_STORAGE.vms.lock().await.clone()
}

pub async fn get_containers_internal() -> Vec<ContainerInstance> {
    COMPUTE_STORAGE.containers.lock().await.clone()
}

async fn get_nodes() -> Json<Value> {
    let nodes = COMPUTE_STORAGE.nodes.lock().await;
    Json(json!({ "nodes": *nodes }))
}

async fn post_node(Json(payload): Json<ComputeNode>) -> Json<Value> {
    let mut nodes = COMPUTE_STORAGE.nodes.lock().await;
    let mut new_node = payload;
    new_node.status = "Online".to_string(); // In real app, we would probe it
    new_node.is_local = false;
    nodes.push(new_node);
    Json(json!({ "status": "success" }))
}

pub fn compute_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/nodes", get(get_nodes).post(post_node))
        .route("/vms", get(get_vms).post(post_vm))
        .route("/containers", get(get_containers).post(post_container))
        .route("/stacks", get(get_stacks).post(post_stack))
        .route("/storage", get(get_storage).post(post_storage))
}
