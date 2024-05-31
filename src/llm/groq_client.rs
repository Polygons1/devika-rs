use crate::config::Config;
use serde::{Serialize, Deserialize};

pub struct Groq {
    client: reqwest::Client,
    key: String
}

impl Groq {
    pub fn new() -> Self {
        let config = Config::new().unwrap();
        let api_key = config.get_groq_api_key();
        Self { client: reqwest::Client::new(), key: api_key.to_string() }
    }

    pub fn inference(&self, model_id: &str, prompt: &str) -> String {
        let req = self.client.post("https://api.groq.com/openai/v1/chat/completions").bearer_auth(self.key.clone()).header("content-type", "application/json").body(format!("{{\"messages\": [{{\"role\":\"user\", \"content\":\"{}\"}}], \"model\":\"{}\"}}", prompt, model_id)).build().unwrap();
        let bytes = req.body().unwrap().as_bytes().unwrap();
        let string = String::from_utf8(bytes.to_vec()).unwrap();

        let response = serde_json::from_str::<GroqResponse>(&string).unwrap();

        response.choices.get(0).unwrap().message.content.clone()
    }
}

#[derive(Serialize, Deserialize)]
struct GroqResponse {
    choices: Vec<Choice>
}

#[derive(Serialize, Deserialize)]
struct Choice {
    message: Message
}

#[derive(Serialize, Deserialize)]
struct Message {
    content: String  // Changed &'static str to String for more flexibility
}