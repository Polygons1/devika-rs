use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::prelude::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use lazy_static::lazy_static;

use crate::socket_instance::{self, emit_agent};

lazy_static! {
    static ref TIKTOKEN_ENC: &'static str = "cl100k_base";
}

#[derive(Serialize, Deserialize)]
struct AgentStateModel {
    project: String,
    state_stack: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct AgentState {
    db_file: PathBuf,
}

impl AgentState {
    pub fn new(db_file: &str) -> Self {
        Self {
            db_file: PathBuf::from(db_file),
        }
    }

    pub fn new_state() -> Value {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        json!({
            "internal_monologue": "",
            "browser_session": {
                "url": null,
                "screenshot": null
            },
            "terminal_session": {
                "command": null,
                "output": null,
                "title": null
            },
            "step": 0,
            "message": null,
            "completed": false,
            "agent_is_active": true,
            "token_usage": 0,
            "timestamp": timestamp
        })
    }

    pub fn create_state(&self, project: &str) {
        let mut new_state = Self::new_state();
        new_state["step"] = json!(1);
        new_state["internal_monologue"] = json!("I'm starting the work...");

        let agent_state = AgentStateModel {
            project: project.to_string(),
            state_stack: vec![new_state.clone().to_owned()],
        };

        let mut file = File::create(&self.db_file).unwrap();
        let serialized = serde_json::to_string(&agent_state).unwrap();
        file.write_all(serialized.as_bytes()).unwrap();

        socket_instance::emit_agent("agent-state", json!({"new_state": new_state }));
    }

    pub fn delete_state(&self, project: &str) {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let mut agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            agent_states.retain(|state| state.project != project);
            let serialized = serde_json::to_string(&agent_states).unwrap();
            file.write_all(serialized.as_bytes()).unwrap();
        }
    }

    pub fn add_to_current_state(&self, project: &str, state: &Value) {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let mut agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter_mut().find(|state| state.project == project) {
                agent_state.state_stack.push(state.clone().to_owned());
            }
            let serialized = serde_json::to_string(&agent_states).unwrap();
            file.write_all(serialized.as_bytes()).unwrap();
        }

        socket_instance::emit_agent("agent-state", json!({"state": state}));
    }

    pub fn get_current_state(&self, project: &str) -> Option<Vec<Value>> {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter().find(|state| state.project == project) {
                return Some(agent_state.state_stack.clone());
            }
        }
        None
    }

    pub fn update_latest_state(&self, project: &str, state: Value) {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let mut agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter_mut().find(|state| state.project == project) {
                agent_state.state_stack.pop();
                agent_state.state_stack.push(state.clone());
            }
            let serialized = serde_json::to_string(&agent_states).unwrap();
            file.write_all(serialized.as_bytes()).unwrap();
        }

        socket_instance::emit_agent("agent-state", json!({"state": state.clone()}));
    }

    pub fn get_latest_state(&self, project: &str) -> Option<Value> {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter().find(|state| state.project == project) {
                return agent_state.state_stack.last().cloned();
            }
        }
        None
    }

    pub fn set_agent_active(&self, project: &str, is_active: bool) {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let mut agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter_mut().find(|state| state.project == project) {
                if let Some(latest_state) = agent_state.state_stack.last_mut() {
                    latest_state["agent_is_active"] = json!(is_active);
                }
            }
            let serialized = serde_json::to_string(&agent_states).unwrap();
            file.write_all(serialized.as_bytes()).unwrap();
        }

        socket_instance::emit_agent("agent-state", json!({"is_active": is_active}));
    }

    pub fn is_agent_active(&self, project: &str) -> Option<bool> {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter().find(|state| state.project == project) {
                if let Some(latest_state) = agent_state.state_stack.last() {
                    return Some(latest_state["agent_is_active"].as_bool().unwrap_or(false));
                }
            }
        }
        None
    }

    pub fn set_agent_completed(&self, project: &str, is_completed: bool) {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let mut agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter_mut().find(|state| state.project == project) {
                if let Some(latest_state) = agent_state.state_stack.last_mut() {
                    latest_state["internal_monologue"] = json!("Agent has completed the task.");
                    latest_state["completed"] = json!(is_completed);
                }
            }
            let serialized = serde_json::to_string(&agent_states).unwrap();
            file.write_all(serialized.as_bytes()).unwrap();
        }

        socket_instance::emit_agent("agent-state", json!({"is_completed": is_completed}));
    }

    pub fn is_agent_completed(&self, project: &str) -> Option<bool> {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter().find(|state| state.project == project) {
                if let Some(latest_state) = agent_state.state_stack.last() {
                    return Some(latest_state["completed"].as_bool().unwrap_or(false));
                }
            }
        }
        None
    }

    pub fn update_token_usage(&self, project: &str, token_usage: i32) {
        if let Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let mut agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter_mut().find(|state| state.project == project) {
                if let Some(latest_state) = agent_state.state_stack.last_mut() {
                    let current_usage: i64 = latest_state["token_usage"].as_i64().unwrap_or(0);
                    latest_state["token_usage"] = json!(current_usage + token_usage as i64);
                }
            }
            let serialized = serde_json::to_string(&agent_states).unwrap();
            file.write_all(serialized.as_bytes()).unwrap();
        }
    }

    pub fn get_latest_token_usage(&self, project: &str) -> Option<i64> {
        if let         Ok(mut file) = File::open(&self.db_file) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let agent_states: Vec<AgentStateModel> = serde_json::from_str(&contents).unwrap();
            if let Some(agent_state) = agent_states.iter().find(|state| state.project == project) {
                if let Some(latest_state) = agent_state.state_stack.last() {
                    return Some(latest_state["token_usage"].as_i64()).unwrap_or(Some(0));
                }
            }
        }
        None
    }

    pub fn get_project_files(&self, project_name: &str) -> Vec<Value> {
        if project_name.is_empty() {
            return vec![];
        }

        let project_directory = project_name.replace(" ", "-");
        let mut directory = std::env::current_dir().unwrap();
        directory.push("data");
        directory.push("projects");
        directory.push(&project_directory);

        if !directory.exists() {
            return vec![];
        }

        let mut files = vec![];
        let entries = fs::read_dir(directory).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let relative_path = path.strip_prefix(std::env::current_dir().unwrap()).unwrap().to_str().unwrap().to_string();
                let content = fs::read_to_string(&path).unwrap();
                files.push(json!({
                    "file": relative_path,
                    "code": content,
                }));
            }
        }
        files
    }
    pub fn update_global_token_usage(&self, string: &str, project_name: &str) {
        let token_usage = tiktoken::count_text(&TIKTOKEN_ENC, string);
        self.update_token_usage(project_name, token_usage.try_into().unwrap());
        let usage: i32 = token_usage.try_into().unwrap();
    
        let total = &self.get_latest_token_usage(project_name).unwrap() + i64::from(usage);
        emit_agent("tokens", serde_json::json!({ "token_usage": total }));
    }
}