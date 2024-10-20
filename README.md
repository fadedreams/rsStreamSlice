curl -H "Range: bytes=0-1023" http://127.0.0.1:8080 --output video_part.mp4
curl http://127.0.0.1:8080/video --output full_video.mp4
