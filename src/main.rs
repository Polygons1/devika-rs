pub mod state;
pub mod config;
pub mod socket_instance;
pub mod llm;
pub mod logger;
pub mod project;

#[macro_use] extern crate rocket;
extern crate serde;

use logger::Logger;
use project::ProjectManager;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::fs::NamedFile;
use rocket::State;
use std::str::FromStr;
use std::sync::{Mutex, Arc};
use std::path::PathBuf;
use serde_json::json;
use state::AgentState;
use config::Config;
use lazy_static::lazy_static;

lazy_static! {
    static ref TIKTOKEN_ENC: &'static str = "cl100k_base";
}

#[derive(Serialize, Deserialize)]
struct AppState {
    config: Mutex<Config>,
    agent_state: Arc<AgentState>, // Use Arc to manage shared state
    // Add other shared states here
}

#[get("/api/data")]
async fn data(state: &State<Arc<AppState>>) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let project = ProjectManager::new().unwrap().get_project_list().await;
    let models = llm::llm::LLM::new(Some(String::new())).list_models();
    let search_engines = vec!["Bing", "Google", "DuckDuckGo"];
    Json(json!({"projects": project, "models": models, "search_engines": search_engines}))
}

#[post("/api/messages", format = "application/json", data = "<data>")]
async fn get_messages(state: &State<Arc<AppState>>, data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let project_name = data["project_name"].as_str().unwrap();
    let messages = agent_state.get_messages_for_project(project_name).await;
    Json(json!({"messages": messages}))
}

#[post("/api/is-agent-active", format = "application/json", data = "<data>")]
async fn is_agent_active(state: &State<Arc<AppState>>, data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let project_name = data["project_name"].as_str().unwrap();
    let is_active = agent_state.is_agent_active(project_name).unwrap_or(false);
    Json(json!({"is_active": is_active}))
}

#[post("/api/get-agent-state", format = "application/json", data = "<data>")]
async fn get_agent_state(state: &State<Arc<AppState>>, data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let project_name = data["project_name"].as_str().unwrap();
    let agent_state_result = agent_state.get_latest_state(project_name);
    Json(json!({"state": agent_state_result}))
}

#[get("/api/get-project-files?<project_name>")]
async fn project_files(state: &State<Arc<AppState>>, project_name: String) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let files = agent_state.get_project_files(&project_name);
    Json(json!({"files": files}))
}

#[get("/api/get-browser-snapshot?<snapshot_path>")]
async fn browser_snapshot(snapshot_path: String) -> Option<NamedFile> {
    NamedFile::open(PathBuf::from_str(snapshot_path.as_str()).unwrap()).await.ok()
}

#[get("/api/get-browser-session?<project_name>")]
async fn get_browser_session(state: &State<Arc<AppState>>, project_name: String) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let agent_state_result = agent_state.get_latest_state(&project_name);
    if let Some(state) = agent_state_result {
        Json(json!({"session": state["browser_session"]}))
    } else {
        Json(json!({"session": "null"}))
    }
}

#[get("/api/get-terminal-session?<project_name>")]
async fn get_terminal_session(state: &State<Arc<AppState>>, project_name: String) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let agent_state_result = agent_state.get_latest_state(&project_name);
    if let Some(state) = agent_state_result {
        Json(json!({"terminal_state": state["terminal_session"]}))
    } else {
        Json(json!({"terminal_state": "null"}))
    }
}

#[post("/api/run-code", format = "application/json", data = "<data>")]
fn run_code(state: &State<Arc<AppState>>, data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let project_name = data["project_name"].as_str().unwrap();
    let code = data["code"].as_str().unwrap();
    // implement code execution logic here
    Json(json!({"message": "Code execution started"}))
}

#[post("/api/calculate-tokens", format = "application/json", data = "<data>")]
async fn calculate_tokens(state: &State<Arc<AppState>>, data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    let prompt = data["prompt"].as_str().unwrap();
    let tokens = tiktoken::count_text(&TIKTOKEN_ENC, prompt);
    Json(json!({"token_usage": tokens}))
}

#[get("/api/token-usage?<project_name>")]
async fn token_usage(state: &State<Arc<AppState>>, project_name: String) -> Json<serde_json::Value> {
    let agent_state = state.agent_state.clone();
    let token_count = agent_state.get_latest_token_usage(&project_name).unwrap_or(0);
    Json(json!({"token_usage": token_count}))
}

#[get("/api/logs")]
async fn real_time_logs(_state: &State<Arc<AppState>>) -> Json<serde_json::Value> {
    let logs = Logger::new("").read_log_file().unwrap();
    Json(json!({"logs": logs}))
}

#[post("/api/settings", format = "application/json", data = "<data>")]
async fn set_settings(state: &State<Arc<AppState>>, data: Json<Config>) -> Json<serde_json::Value> {
    let mut config = state.config.lock().unwrap();
    *config = data.into_inner();
    Json(json!({"message": "Settings updated"}))
}

#[get("/api/settings")]
async fn get_settings(state: &State<Arc<AppState>>) -> Json<serde_json::Value> {
    let config = state.config.lock().unwrap();
    Json(json!({"settings": *config}))
}

#[get("/api/status")]
async fn status() -> Json<serde_json::Value> {
    Json(json!({"status": "server is running!"}))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(initialize_app_state())
        .mount("/", routes![
            data,
            get_messages,
            is_agent_active,
            get_agent_state,
            project_files,
            browser_snapshot,
            get_browser_session,
            get_terminal_session,
            run_code,
            calculate_tokens,
            token_usage,
            real_time_logs,
            set_settings,
            get_settings,
            status,
        ])
}

fn initialize_app_state() -> Arc<AppState> {
    let config = Config::new().unwrap();
    let agent_state = Arc::new(AgentState::new("sqlite://database_url"));

    Arc::new(AppState {
        config: Mutex::new(config),
        agent_state,
        // Initialize other shared states here
    })
}

// Removed functions moved to state module
