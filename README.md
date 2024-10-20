A Rust library for efficient  streaming with support for partial content delivery.

### Description

A web server built using Actix that allows streaming of video and audio files with support for partial content delivery. This means that clients can request specific byte ranges from the files, enabling features like fast seeking in videos and resuming downloads for audio files.

## Features

- Stream video files in MP4 format.
- Stream audio files in MP3 format.
- Support for HTTP byte-range requests for partial content delivery.
- Logging for monitoring requests and errors.

## Getting Started

1. **Clone the repository:**
   ```bash
   git clone https://github.com/fadedreams/rsstreamslice.git
   cd rsstreamslice
   ```

2. **Add your media files:**
   Place your video and audio files (e.g., `video.mp4`, `mp3.mp3`) in the root directory of the project.

3. **Build and run the server:**
   ```bash
   cargo run --bin rsstreamslice_server
   ```

4. **Access the server:**
   Open your web browser and navigate to:
   - [http://127.0.0.1:8080/](http://127.0.0.1:8080/) to stream the video.
   - [http://127.0.0.1:8080/video](http://127.0.0.1:8080/video) for video streaming.
   - [http://127.0.0.1:8080/mp3](http://127.0.0.1:8080/mp3) for audio streaming.

## API Usage

### Endpoints

- **GET /**: Streams the default video file (video.mp4).
- **GET /video**: Streams the video file (video.mp4).
- **GET /mp3**: Streams the audio file (mp3.mp3).

### HTTP Headers

- `Range`: Used to request specific byte ranges from the media files.

### Content Types

- `video/mp4` for MP4 video files.
- `audio/mpeg` for MP3 audio files.

### Partial Streaming

This server supports partial streaming, allowing clients to download specific byte ranges of files. 

**Example `curl` commands:**

- To download a part of the video file:
  ```bash
  curl -H "Range: bytes=0-1023" http://127.0.0.1:8080 --output video_part.mp4
  ```

- To download the full video file:
  ```bash
  curl http://127.0.0.1:8080/video --output full_video.mp4
  ```

## Logging

The server uses `env_logger` for logging. You can set the logging level using the `RUST_LOG` environment variable. For example, to enable debug logging, run:

```bash
RUST_LOG=debug cargo run --bin rsstreamslice_server
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any enhancements or bug fixes.
