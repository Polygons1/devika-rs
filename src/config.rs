use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ConfigData {
    API_ENDPOINTS: ApiEndpoints,
    API_KEYS: ApiKeys,
    STORAGE: Storage,
    LOGGING: Logging,
    TIMEOUT: Timeout,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ApiEndpoints {
    BING: String,
    GOOGLE: String,
    OLLAMA: String,
    OPENAI: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ApiKeys {
    BING: String,
    GOOGLE_SEARCH: String,
    GOOGLE_SEARCH_ENGINE_ID: String,
    CLAUDE: String,
    OPENAI: String,
    GEMINI: String,
    MISTRAL: String,
    GROQ: String,
    NETLIFY: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Storage {
    SQLITE_DB: String,
    SCREENSHOTS_DIR: String,
    PDFS_DIR: String,
    PROJECTS_DIR: String,
    LOGS_DIR: String,
    REPOS_DIR: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Logging {
    LOG_REST_API: bool,
    LOG_PROMPTS: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Timeout {
    INFERENCE: u64,
}

static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::new().unwrap()));

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    config: ConfigData,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");
        let sample_config_path = Path::new("sample.config.toml");

        if !config_path.exists() {
            fs::copy(sample_config_path, config_path)?;
        }

        let config: ConfigData = toml::from_str(&fs::read_to_string(config_path)?)?;

        fs::write(config_path, toml::to_string(&config)?)?;

        Ok(Self { config })
    }

    pub fn get_instance() -> &'static Mutex<Config> {
        &CONFIG
    }

    pub fn get_config(&self) -> &ConfigData {
        &self.config
    }

    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");
        fs::write(config_path, toml::to_string(&self.config)?)?;
        Ok(())
    }

    // Define getters for each configuration field
    pub fn get_bing_api_endpoint(&self) -> &String {
        &self.config.API_ENDPOINTS.BING
    }

    pub fn get_bing_api_key(&self) -> &String {
        &self.config.API_KEYS.BING
    }

    pub fn get_google_search_api_key(&self) -> &String {
        &self.config.API_KEYS.GOOGLE_SEARCH
    }

    pub fn get_google_search_engine_id(&self) -> &String {
        &self.config.API_KEYS.GOOGLE_SEARCH_ENGINE_ID
    }

    pub fn get_google_search_api_endpoint(&self) -> &String {
        &self.config.API_ENDPOINTS.GOOGLE
    }

    pub fn get_ollama_api_endpoint(&self) -> &String {
        &self.config.API_ENDPOINTS.OLLAMA
    }

    pub fn get_claude_api_key(&self) -> &String {
        &self.config.API_KEYS.CLAUDE
    }

    pub fn get_openai_api_key(&self) -> &String {
        &self.config.API_KEYS.OPENAI
    }

    pub fn get_openai_api_base_url(&self) -> &String {
        &self.config.API_ENDPOINTS.OPENAI
    }

    pub fn get_gemini_api_key(&self) -> &String {
        &self.config.API_KEYS.GEMINI
    }

    pub fn get_mistral_api_key(&self) -> &String {
        &self.config.API_KEYS.MISTRAL
    }

    pub fn get_groq_api_key(&self) -> &String {
        &self.config.API_KEYS.GROQ
    }

    pub fn get_netlify_api_key(&self) -> &String {
        &self.config.API_KEYS.NETLIFY
    }

    pub fn get_sqlite_db(&self) -> &String {
        &self.config.STORAGE.SQLITE_DB
    }

    pub fn get_screenshots_dir(&self) -> &String {
        &self.config.STORAGE.SCREENSHOTS_DIR
    }

    pub fn get_pdfs_dir(&self) -> &String {
        &self.config.STORAGE.PDFS_DIR
    }

    pub fn get_projects_dir(&self) -> &String {
        &self.config.STORAGE.PROJECTS_DIR
    }

    pub fn get_logs_dir(&self) -> &String {
        &self.config.STORAGE.LOGS_DIR
    }

    pub fn get_repos_dir(&self) -> &String {
        &self.config.STORAGE.REPOS_DIR
    }

    pub fn get_logging_rest_api(&self) -> bool {
        self.config.LOGGING.LOG_REST_API
    }

    pub fn get_logging_prompts(&self) -> bool {
        self.config.LOGGING.LOG_PROMPTS
    }

    pub fn get_timeout_inference(&self) -> u64 {
        self.config.TIMEOUT.INFERENCE
    }

    // Define setters for each configuration field
    pub fn set_bing_api_key(&mut self, key: String) {
        self.config.API_KEYS.BING = key;
        self.save_config().unwrap();
    }

    pub fn set_bing_api_endpoint(&mut self, endpoint: String) {
        self.config.API_ENDPOINTS.BING = endpoint;
        self.save_config().unwrap();
    }

    pub fn set_google_search_api_key(&mut self, key: String) {
        self.config.API_KEYS.GOOGLE_SEARCH = key;
        self.save_config().unwrap();
    }

    pub fn set_google_search_engine_id(&mut self, key: String) {
        self.config.API_KEYS.GOOGLE_SEARCH_ENGINE_ID = key;
        self.save_config().unwrap();
    }

    pub fn set_google_search_api_endpoint(&mut self, endpoint: String) {
        self.config.API_ENDPOINTS.GOOGLE = endpoint;
        self.save_config().unwrap();
    }

    pub fn set_ollama_api_endpoint(&mut self, endpoint: String) {
        self.config.API_ENDPOINTS.OLLAMA = endpoint;
        self.save_config().unwrap();
    }

    pub fn set_claude_api_key(&mut self, key: String) {
        self.config.API_KEYS.CLAUDE = key;
        self.save_config().unwrap();
    }

    pub fn set_openai_api_key(&mut self, key: String) {
        self.config.API_KEYS.OPENAI = key;
        self.save_config().unwrap();
    }

    pub fn set_openai_api_endpoint(&mut self, endpoint: String) {
        self.config.API_ENDPOINTS.OPENAI = endpoint;
        self.save_config().unwrap();
    }

    pub fn set_gemini_api_key(&mut self, key: String) {
        self.config.API_KEYS.GEMINI = key;
        self.save_config().unwrap();
    }

    pub fn set_mistral_api_key(&mut self, key: String) {
        self.config.API_KEYS.MISTRAL = key;
        self.save_config().unwrap();
    }

    pub fn set_groq_api_key(&mut self, key: String) {
        self.config.API_KEYS.GROQ = key;
        self.save_config().unwrap();
    }

    pub fn set_netlify_api_key(&mut self, key: String) {
        self.config.API_KEYS.NETLIFY = key;
        self.save_config().unwrap();
    }

    pub fn set_logging_rest_api(&mut self, value: bool) {
        self.config.LOGGING.LOG_REST_API = value;
        self.save_config().unwrap();
    }

    pub fn set_logging_prompts(&mut self, value: bool) {
        self.config.LOGGING.LOG_PROMPTS = value;
        self.save_config().unwrap();
    }

    pub fn set_timeout_inference(&mut self, value: u64) {
        self.config.TIMEOUT.INFERENCE = value;
        self.save_config().unwrap();
    }

    pub fn update_config(config: &Config) -> Result<(), std::io::Error> {
        let mut config_guard = CONFIG.lock().unwrap();
        *config_guard = config.to_owned().clone();
        config.save_config().unwrap();
        Ok(())
    }
}
