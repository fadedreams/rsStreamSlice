// src/bin/main.rs

use actix_web::{web, App, HttpServer};
use rsstreamslice_server::handler; // Use the correct crate name

// const FILE_PATH: &str = "/home/x/Videos/ex1.mp4";
const FILE_PATH: &str = "video.mp4"; // Use a relative path

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(|req| handler(req, FILE_PATH))))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
