# Voice Agent with GPT-4.1

A powerful voice-based AI agent that uses OpenAI's GPT-4.1 for natural language processing and advanced text-to-speech capabilities. This project provides a seamless voice interaction experience with state-of-the-art AI models.

## Features

- **Advanced Voice Processing**: 
  - WebM to WAV conversion using ffmpeg-next
  - Audio normalization and filtering
  - Real-time audio processing with cpal
  - Ring buffer for efficient audio handling

- **GPT-4.1 Integration**: 
  - Uses the latest GPT-4.1 model for intelligent conversation
  - Context-aware responses
  - Client-side conversation history management

- **High-Quality Text-to-Speech**: 
  - Uses GPT-4o-mini-tts model
  - Natural voice synthesis
  - Multiple voice options

- **Robust Infrastructure**:
  - Built with Axum web framework
  - Async runtime with Tokio
  - Prometheus metrics integration
  - Comprehensive error handling
  - CORS support
  - Health check endpoints

## Project Structure

```
src/
├── api/
│   ├── handlers/    # Request handlers
│   ├── models/      # Data structures
│   └── mod.rs       # API router setup
├── audio/
│   ├── converter/   # Audio format conversion
│   ├── processor/   # Audio processing
│   └── mod.rs
├── error/           # Custom error types
├── metrics/         # Prometheus metrics
├── config/          # Configuration management
└── main.rs         # Application entry point
```

## API Endpoints

### POST /process_voice_agent
Process voice input and return AI response as audio.

**Request Format**:
- Multipart form data with the following fields:
  - `audio`: Audio file (WebM format)
  - `session_id`: Optional session identifier
  - `conversation_history`: Optional JSON array of previous messages

**Response Format**:
```json
{
    "audio_data": "base64_encoded_audio",
    "text": "transcribed_text"
}
```

### GET /health
Check system health status.

**Response Format**:
```json
{
    "status": "healthy",
    "version": "1.0.0",
    "uptime": "duration"
}
```

### GET /metrics
Get Prometheus metrics.

## Setup and Installation

1. **Prerequisites**:
   - Rust (latest stable version)
   - FFmpeg development libraries
   - OpenAI API key

2. **System Dependencies**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libavcodec-dev libavformat-dev libavutil-dev libavdevice-dev libavfilter-dev libswresample-dev libswscale-dev ffmpeg
   ```

3. **Environment Configuration**:
   Create a `.env` file in the project root with the following configuration:

   ```env
   # Required Configuration
   OPENAI_API_KEY=your_openai_api_key_here

   # Optional Configuration (with defaults)
   # Server Configuration
   # SERVER_HOST=127.0.0.1
   # SERVER_PORT=3000

   # Audio Processing Configuration
   # AUDIO_SAMPLE_RATE=48000
   # AUDIO_CHANNELS=1
   # AUDIO_BUFFER_SIZE=4096

   # Logging Configuration
   # RUST_LOG=info
   # RUST_BACKTRACE=1

   # Metrics Configuration
   # METRICS_PORT=9090
   ```

   **Configuration Details**:

   **Required Settings**:
   - `OPENAI_API_KEY`: Your OpenAI API key (required for all OpenAI API calls)

   **Optional Settings** (with defaults):
   - `SERVER_HOST`: Host address to bind the server (default: 127.0.0.1)
   - `SERVER_PORT`: Port to run the server (default: 3000)
   - `AUDIO_SAMPLE_RATE`: Sample rate for audio processing (default: 48000)
   - `AUDIO_CHANNELS`: Number of audio channels (default: 1 for mono)
   - `AUDIO_BUFFER_SIZE`: Size of audio processing buffer (default: 4096)
   - `RUST_LOG`: Logging level (default: info, options: error, warn, info, debug, trace)
   - `RUST_BACKTRACE`: Enable backtrace for debugging (default: 0, set to 1 to enable)
   - `METRICS_PORT`: Port for Prometheus metrics (default: 9090)

   **Security Notes**:
   - Never commit the `.env` file to version control
   - Keep your API keys secure
   - Use different API keys for development and production
   - Consider using a secrets management service in production

4. **Build and Run**:
   ```bash
   cargo build --release
   cargo run
   ```

## Audio Processing Pipeline

1. **Input Processing**:
   - Accepts WebM audio input
   - Converts to WAV format using ffmpeg-next
   - Applies audio normalization and filtering
   - Uses ring buffer for efficient processing

2. **Speech-to-Text**:
   - Uses OpenAI's Whisper model
   - Handles multiple languages
   - Provides accurate transcription

3. **AI Processing**:
   - GPT-4.1 model for context-aware responses
   - Maintains conversation context
   - Generates natural, coherent responses

4. **Text-to-Speech**:
   - Uses GPT-4o-mini-tts model
   - High-quality voice synthesis
   - Natural intonation and pacing

## Error Handling

The system implements comprehensive error handling:
- Custom error types in `src/error/mod.rs`
- Proper error propagation using Anyhow
- Detailed error messages
- Graceful failure handling

## Performance Monitoring

Built-in Prometheus metrics for monitoring:
- Request counts
- Processing times
- Error rates
- System health
- Audio processing metrics

## Security Considerations

- API key management through environment variables
- CORS configuration with Tower
- Input validation
- Error message sanitization
- Rate limiting (to be implemented)

## Docker Support

The project includes a Dockerfile for containerized deployment:
```bash
docker build -t voice-agent .
docker run -p 3000:3000 voice-agent
```

## Future Improvements

1. **Planned Features**:
   - Rate limiting
   - Authentication system
   - Multiple voice options
   - Streaming responses
   - WebSocket support

2. **Performance Optimizations**:
   - Caching layer with Moka
   - Response compression
   - Connection pooling
   - Load balancing

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- OpenAI for providing the AI models
- The Rust community for excellent tools and libraries
- FFmpeg project for audio processing capabilities
- All contributors to this project (which us currently me :D)