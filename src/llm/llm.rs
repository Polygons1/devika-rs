use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::llm::ollama_client::Ollama;
use crate::llm::groq_client::Groq;

use crate::state::AgentState;
use crate::config::Config;
use crate::logger::Logger;
use crate::socket_instance::emit_agent;

use lazy_static::lazy_static;

lazy_static! {
    static ref TIKTOKEN_ENC: &'static str = "cl100k_base";
}

lazy_static! {
    static ref MODEL_MAPPING: HashMap<String, Arc<dyn InferenceModel>> = {
        let mut map = HashMap::new();
        map.insert("OLLAMA".to_string(), Arc::new(Ollama::new()) as Arc<dyn InferenceModel>);
        map.insert("GROQ".to_string(), Arc::new(Groq::new()) as Arc<dyn InferenceModel>);
        map
    };
}

pub struct LLM {
    model_id: Option<String>,
    log_prompts: bool,
    timeout_inference: Duration,
    models: HashMap<String, Vec<(String, String)>>,
}

impl LLM {
    pub fn new(model_id: Option<String>) -> Self {
        let config = Config::new().unwrap();
        let ollama = Ollama::new();
        
        let mut models = HashMap::new();
        models.insert("CLAUDE".to_string(), vec![
            ("Claude 3 Opus".to_string(), "claude-3-opus-20240229".to_string()),
            ("Claude 3 Sonnet".to_string(), "claude-3-sonnet-20240229".to_string()),
            ("Claude 3 Haiku".to_string(), "claude-3-haiku-20240307".to_string()),
        ]);
        models.insert("OPENAI".to_string(), vec![
            ("GPT-4o".to_string(), "gpt-4o".to_string()),
            ("GPT-4 Turbo".to_string(), "gpt-4-turbo".to_string()),
            ("GPT-3.5 Turbo".to_string(), "gpt-3.5-turbo-0125".to_string()),
        ]);
        models.insert("GOOGLE".to_string(), vec![
            ("Gemini 1.0 Pro".to_string(), "gemini-pro".to_string()),
        ]);
        models.insert("MISTRAL".to_string(), vec![
            ("Mistral 7b".to_string(), "open-mistral-7b".to_string()),
            ("Mistral 8x7b".to_string(), "open-mixtral-8x7b".to_string()),
            ("Mistral Medium".to_string(), "mistral-medium-latest".to_string()),
            ("Mistral Small".to_string(), "mistral-small-latest".to_string()),
            ("Mistral Large".to_string(), "mistral-large-latest".to_string()),
        ]);
        models.insert("GROQ".to_string(), vec![
            ("LLAMA3 8B".to_string(), "llama3-8b-8192".to_string()),
            ("LLAMA3 70B".to_string(), "llama3-70b-8192".to_string()),
            ("LLAMA2 70B".to_string(), "llama2-70b-4096".to_string()),
            ("Mixtral".to_string(), "mixtral-8x7b-32768".to_string()),
            ("GEMMA 7B".to_string(), "gemma-7b-it".to_string()),
        ]);
        if let Some(client) = ollama.client {
            models.insert("OLLAMA".to_string(), tokio::runtime::Runtime::new().unwrap().block_on(client.list_local_models()).unwrap().iter().map(|model| (model.name.clone(), model.name.clone())).collect());
        }

        LLM {
            model_id,
            log_prompts: config.get_logging_prompts(),
            timeout_inference: Duration::from_secs(config.get_timeout_inference()),
            models,
        }
    }

    pub fn list_models(&self) -> &HashMap<String, Vec<(String, String)>> {
        &self.models
    }

    fn model_enum(&self, model_name: &str) -> Option<(String, String)> {
        self.models.iter()
            .flat_map(|(model_enum, models)| {
                models.iter().map(move |(name, id)| (name.clone(), (model_enum.clone(), id.clone())))
            })
            .collect::<HashMap<String, (String, String)>>()
            .get(model_name)
            .cloned()
    }

    fn update_global_token_usage(string: &str, project_name: &str) {
        let token_usage = tiktoken::count_text(&TIKTOKEN_ENC, string);
        let agent_state = AgentState::new(&Config::new().unwrap().get_sqlite_db());
        agent_state.update_token_usage(project_name, token_usage.try_into().unwrap());

        let usage: i32 = token_usage.try_into().unwrap();

        let total = agent_state.get_latest_token_usage(project_name) .unwrap() + i64::from(usage);
        emit_agent("tokens", serde_json::json!({ "token_usage": total }));
    }

    fn inference(&self, prompt: &str, project_name: &str) -> Result<String, String> {
        Self::update_global_token_usage(prompt, project_name);

        let (model_enum, model_name) = match self.model_enum(self.model_id.as_deref().unwrap_or("")) {
            Some(model) => model,
            None => return Err(format!("Model {} not supported", self.model_id.clone().unwrap_or_default())),
        };

        println!("Model: {:?}, Enum: {:?}", self.model_id, model_enum);

        let model = MODEL_MAPPING.get(model_enum.as_str()).ok_or_else(|| format!("Model {} not supported", model_enum))?;

        let logger = Logger::new("devika_agent.log");

        let start_time = Instant::now();
        let result = Arc::new(Mutex::new(None));

        let handle = {
            let result = Arc::clone(&result);
            let model_name = model_name.clone();
            let prompt = prompt.to_string();

            thread::spawn(move || {
                let inference_result = model.inference(&model_name, &prompt);
                *result.lock().unwrap() = Some(inference_result);
            })
        };

        loop {
            let elapsed_time = start_time.elapsed().as_secs_f32();
            let elapsed_seconds = format!("{:.2}", elapsed_time);
            emit_agent("inference", serde_json::json!({ "type": "time", "elapsed_time": elapsed_seconds }));

            if elapsed_time >= 5.0 {
                emit_agent("inference", serde_json::json!({ "type": "warning", "message": "Inference is taking longer than expected" }));
            }
            if elapsed_time > self.timeout_inference.as_secs_f32() {
                emit_agent("inference", serde_json::json!({ "type": "error", "message": "Inference took too long. Please try again." }));
                logger.error(&format!("Inference failed. Took too long. Model: {}, Model ID: {:?}", model_enum, self.model_id));
                return Err("Inference took too long. Please try again.".to_string());
            }

            if let Some(response) = result.lock().unwrap().as_ref() {
                match response {
                    Ok(response) => {
                        let response = response.trim().to_string();
                        if self.log_prompts {
                            logger.debug(&format!("Response ({}): --> {}", model_enum, response));
                        }
                        Self::update_global_token_usage(&response, project_name);
                        return Ok(response);
                    }
                    Err(e) => {
                        emit_agent("inference", serde_json::json!({ "type": "error", "message": e }));
                        return Err(e.clone());
                    }
                }
            }

            thread::sleep(Duration::from_millis(500));
        }
    }
}

trait InferenceModel: Send + Sync {
    fn inference(&self, model_name: &str, prompt: &str) -> Result<String, String>;
}

impl InferenceModel for Ollama {
    fn inference(&self, model_name: &str, prompt: &str) -> Result<String, String> {
        Ok(self.inference(model_name, prompt))
    }
}

impl InferenceModel for Groq {
    fn inference(&self, model_name: &str, prompt: &str) -> Result<String, String> {
        Ok(self.inference(model_name, prompt))
    }
}