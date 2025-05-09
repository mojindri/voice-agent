use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::metrics::Metrics;

#[derive(Clone)]
pub struct AppState {
    pub metrics: Arc<Metrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime: std::time::Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioResponse {
    pub audio_data: Vec<u8>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatGPTRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

#[derive(Debug, Serialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
}

#[derive(Debug, Serialize)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: Parameters,
}

#[derive(Debug, Serialize)]
pub struct Parameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: serde_json::Value,
    pub required: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatGPTResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
}
