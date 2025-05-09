use anyhow::Result;
use axum::{
    extract::{Multipart, State},
    Json,
};
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use std::env;
use tracing::{error, info};
use uuid;

use crate::audio::converter::webm_to_wav_bytes;
use crate::{
    api::models::{
        AppState, AudioResponse, ChatGPTRequest, ChatGPTResponse, ChatMessage, HealthStatus,
    },
    error::AppError,
};

pub async fn health_check() -> Json<HealthStatus> {
    info!("Health check requested");
    Json(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: std::time::Duration::from_secs(0),
    })
}

pub async fn metrics() -> String {
    use metrics_exporter_prometheus::PrometheusBuilder;
    let builder = PrometheusBuilder::new();
    let handle = builder.install_recorder().unwrap();
    handle.render()
}

pub async fn process_voice_agent(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Json<AudioResponse>, AppError> {
    info!("Processing voice agent request");
    state.metrics.record_request();
    let start_time = std::time::Instant::now();

    let (audio_data, session_id, conversation_history) =
        extract_audio_data_and_session(multipart).await?;

    if audio_data.is_empty() {
        error!("No audio data received");
        return Err(anyhow::anyhow!("No audio data received").into());
    }

    let (text, audio_response) =
        handle_voice_agent(&audio_data, &session_id, conversation_history, &state).await?;

    state.metrics.record_processing_time(start_time.elapsed());

    info!("Voice agent processed successfully");
    Ok(Json(AudioResponse {
        audio_data: audio_response,
        text,
    }))
}

async fn extract_audio_data_and_session(
    mut multipart: Multipart,
) -> Result<(Vec<u8>, String, Vec<ChatMessage>), AppError> {
    let mut audio_data = Vec::new();
    let mut session_id = String::new();
    let mut conversation_history = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("audio") => {
                let data = field.bytes().await?;
                if data.is_empty() {
                    error!("Empty audio data received");
                    return Err(anyhow::anyhow!("Empty audio data received").into());
                }
                audio_data = data.to_vec();
            }
            Some("session_id") => {
                session_id = field.text().await?;
            }
            Some("conversation_history") => {
                let history_text = field.text().await?;
                if !history_text.is_empty() {
                    conversation_history = serde_json::from_str(&history_text)?;
                }
            }
            _ => {}
        }
    }

    if audio_data.is_empty() {
        error!("No audio data found in request");
        return Err(anyhow::anyhow!("No audio data found in request").into());
    }

    if session_id.is_empty() {
        session_id = uuid::Uuid::new_v4().to_string();
    }

    Ok((audio_data, session_id, conversation_history))
}

async fn handle_voice_agent(
    audio_data: &[u8],
    _session_id: &str,
    mut conversation_history: Vec<ChatMessage>,
    _state: &AppState,
) -> Result<(String, Vec<u8>), AppError> {
    if conversation_history.is_empty() {
        conversation_history.push(ChatMessage {
            role: "system".to_string(),
            content: "You are a friendly and empathetic voice assistant. Speak naturally and conversationally, like a trusted friend. Keep responses concise but warm and engaging. Remember to maintain a friendly vibe in all interactions. Use a casual, approachable tone while staying helpful and informative. When appropriate, show personality and humor, but always prioritize being helpful and clear.".to_string(),
        });
    }
    let text = speech_to_text(audio_data).await?;
    let response = chat_with_gpt(&text, &mut conversation_history).await?;
    let audio_response = text_to_speech(&response).await?;
    Ok((response, audio_response))
}

async fn speech_to_text(audio_data: &[u8]) -> Result<String, AppError> {
    let client = Client::new();
    let api_key =
        env::var("OPENAI_API_KEY").map_err(|_| anyhow::anyhow!("Missing OpenAI API key"))?;

    let wav_data = webm_to_wav_bytes(audio_data)?;

    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(
            Form::new()
                .text("model", "whisper-1")
                .text("response_format", "text")
                .part(
                    "file",
                    Part::bytes(wav_data)
                        .file_name("audio.wav")
                        .mime_str("audio/wav")?,
                ),
        )
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

async fn chat_with_gpt(
    text: &str,
    conversation_history: &mut Vec<ChatMessage>,
) -> Result<String, AppError> {
    let client = Client::new();
    let api_key =
        env::var("OPENAI_API_KEY").map_err(|_| anyhow::anyhow!("Missing OpenAI API key"))?;

    conversation_history.push(ChatMessage {
        role: "user".to_string(),
        content: text.to_string(),
    });

    let request = ChatGPTRequest {
        model: "gpt-4.1".to_string(),
        messages: conversation_history.clone(),
        temperature: 0.7,
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?;

    // Add debug logging
    let response_text = response.text().await?;
    error!("API Response: {}", response_text);

    let response: ChatGPTResponse = serde_json::from_str(&response_text)?;

    let assistant_message = response.choices[0].message.content.clone();

    conversation_history.push(ChatMessage {
        role: "assistant".to_string(),
        content: assistant_message.clone(),
    });

    Ok(assistant_message)
}

async fn text_to_speech(text: &str) -> Result<Vec<u8>, AppError> {
    let client = Client::new();
    let api_key =
        env::var("OPENAI_API_KEY").map_err(|_| anyhow::anyhow!("Missing OpenAI API key"))?;

    let response = client
        .post("https://api.openai.com/v1/audio/speech")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "gpt-4o-mini-tts", // tts-1
            "input": text,
            "voice": "sage",
            "vibe" :"Friendly"
        }))
        .send()
        .await?
        .bytes()
        .await?;

    Ok(response.to_vec())
}
