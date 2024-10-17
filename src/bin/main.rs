// src/bin/main.rs

use actix_web::{web, App, HttpServer};
use video_streaming_lib::handler;

// const FILE_PATH: &str = "/home/m/Videos/ex1.mp4";
const FILE_PATH: &str = "ex1.mp4"; // Use a relative path

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(|req| handler(req, FILE_PATH))))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
