use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub audio: AudioConfig,
    pub server: ServerConfig,
}

#[derive(Clone, Debug)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            audio: AudioConfig {
                sample_rate: env::var("AUDIO_SAMPLE_RATE")
                    .unwrap_or_else(|_| "48000".to_string())
                    .parse()?,
                channels: env::var("AUDIO_CHANNELS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()?,
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()?,
            },
        })
    }
}
