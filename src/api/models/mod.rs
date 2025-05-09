use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{audio::processor::AudioProcessor, metrics::Metrics};

#[derive(Clone)]
pub struct AppState {
    pub audio_processor: Arc<Mutex<AudioProcessor>>,
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

#[derive(Debug, Deserialize)]
pub struct ChatGPTResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
}
