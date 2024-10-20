use actix_web::{web, App, HttpServer};
use env_logger::Env;
use rsstreamslice_server::handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Define file paths
    let video_path = "video.mp4";

    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(move |req| handler(req, video_path)))
            .route("/video", web::get().to(move |req| handler(req, video_path)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

