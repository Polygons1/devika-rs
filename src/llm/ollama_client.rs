use ollama_rs::generation::completion::request::GenerationRequest;

use crate::config::Config;

pub struct Ollama {
    url: String,
    pub client: Option<ollama_rs::Ollama>
}

impl Ollama {
    pub fn new() -> Self {
        let config = Config::new().unwrap();
        let url = url::Url::parse(config.get_ollama_api_endpoint()).unwrap();
        let ollama = ollama_rs::Ollama::new(url.host().unwrap().to_string(), url.port().unwrap());
        Self { url: config.get_ollama_api_endpoint().to_string(), client: Some(ollama) }
    }
    pub fn inference(&self, model_id: &str, prompt: &str) -> String {
        let client =  self.client.as_ref().unwrap();
        let genres = client.generate(GenerationRequest::new(model_id.to_string(), prompt.to_string()));

        let res = tokio::runtime::Runtime::new().unwrap().block_on(genres).unwrap();



        return res.response
    }
}