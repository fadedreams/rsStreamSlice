use actix_web::{Error, HttpRequest, HttpResponse};
use futures::stream::Stream;
use log::{error, info};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub const BUFSIZE: usize = 1024 * 8;
pub const ACCEPT_RANGES: &str = "bytes";

// Define supported content types
pub fn get_content_type(file_path: &str) -> &'static str {
    match Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
    {
        Some("mp4") => "video/mp4",
        Some("mp3") => "audio/mpeg",
        _ => "application/octet-stream",
    }
}

pub async fn handler(req: HttpRequest, file_path: &str) -> Result<HttpResponse, Error> {
    info!("Handling request for file: {}", file_path);

    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open file {}: {}", file_path, e);
            return Err(Error::from(e));
        }
    };

    let file_size = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            error!("Failed to get file metadata for {}: {}", file_path, e);
            return Err(Error::from(e));
        }
    };

    let content_type = get_content_type(file_path);

    if let Some(range_header) = req.headers().get("Range") {
        info!("Serving partial content for file: {}", file_path);
        Ok(serve_partial_file(
            file,
            file_size,
            range_header.to_str().unwrap_or(""),
            content_type,
        ))
    } else {
        info!("Serving full file: {}", file_path);
        Ok(serve_full_file(file, file_size, content_type))
    }
}

pub fn serve_full_file(file: File, file_size: u64, content_type: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(content_type)
        .insert_header(("Accept-Ranges", ACCEPT_RANGES))
        .insert_header(("Content-Length", file_size.to_string()))
        .insert_header((
            "Content-Range",
            format!("bytes 0-{}/{}", file_size - 1, file_size),
        ))
        .streaming(file_stream(file))
}

pub fn serve_partial_file(
    file: File,
    file_size: u64,
    range_header: &str,
    content_type: &str,
) -> HttpResponse {
    let (start, end) = parse_range(range_header, file_size);
    HttpResponse::PartialContent()
        .content_type(content_type)
        .insert_header(("Accept-Ranges", ACCEPT_RANGES))
        .insert_header(("Content-Length", (end - start + 1).to_string()))
        .insert_header((
            "Content-Range",
            format!("bytes {}-{}/{}", start, end, file_size),
        ))
        .streaming(file_stream_partial(file, start, end))
}

pub fn file_stream(
    file: File,
) -> impl Stream<Item = Result<actix_web::web::Bytes, std::io::Error>> {
    futures::stream::unfold(file, move |mut file| async move {
        let mut buffer = vec![0; BUFSIZE];
        match file.read(&mut buffer) {
            Ok(0) => None,
            Ok(n) => Some((Ok(actix_web::web::Bytes::from(buffer[..n].to_vec())), file)),
            Err(e) => Some((Err(e), file)),
        }
    })
}

pub fn file_stream_partial(
    file: File,
    start: u64,
    end: u64,
) -> impl Stream<Item = Result<actix_web::web::Bytes, std::io::Error>> {
    futures::stream::unfold((file, start), move |(mut file, start)| async move {
        if start > end {
            return None;
        }
        let mut buffer = vec![0; BUFSIZE];
        match file.seek(SeekFrom::Start(start)) {
            Ok(_) => (),
            Err(e) => return Some((Err(e), (file, start))),
        }
        match file.read(&mut buffer) {
            Ok(0) => None,
            Ok(n) => {
                let n = std::cmp::min(n as u64, end - start + 1) as usize;
                Some((
                    Ok(actix_web::web::Bytes::from(buffer[..n].to_vec())),
                    (file, start + n as u64),
                ))
            }
            Err(e) => Some((Err(e), (file, start))),
        }
    })
}

pub fn parse_range(range_header: &str, file_size: u64) -> (u64, u64) {
    let range_param = range_header.split('=').nth(1).unwrap_or("");
    let split_params: Vec<&str> = range_param.split('-').collect();
    let mut start = 0u64;
    let mut end = file_size - 1;
    if !split_params.is_empty() {
        if let Ok(start_val) = split_params[0].parse::<u64>() {
            start = start_val;
        }
    }
    if split_params.len() > 1 {
        if let Ok(end_val) = split_params[1].parse::<u64>() {
            end = end_val;
        }
    }
    (start, end)
}
