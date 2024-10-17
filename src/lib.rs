// lib.rs

use actix_web::{web, Error, HttpRequest, HttpResponse};
use mime_guess::from_path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub struct StreamingConfig {
    pub buffer_size: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        StreamingConfig {
            buffer_size: 1024 * 8,
        }
    }
}

pub const ACCEPT_RANGES: &str = "bytes";

pub async fn handler(
    req: HttpRequest,
    file_path: &str,
    config: &StreamingConfig,
) -> Result<HttpResponse, Error> {
    let file = File::open(file_path).map_err(Error::from)?;
    let file_size = file.metadata().map_err(Error::from)?.len();
    let content_type = get_content_type(file_path);

    if let Some(range_header) = req.headers().get("Range") {
        Ok(serve_partial_file(
            file,
            file_size,
            range_header.to_str().unwrap_or(""),
            config,
            &content_type,
        ))
    } else {
        Ok(serve_full_file(file, file_size, config, &content_type))
    }
}

fn get_content_type(file_path: &str) -> String {
    from_path(file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_owned()
}

pub fn serve_full_file(
    file: File,
    file_size: u64,
    config: &StreamingConfig,
    content_type: &str,
) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(content_type)
        .insert_header(("Accept-Ranges", ACCEPT_RANGES))
        .insert_header(("Content-Length", file_size.to_string()))
        .insert_header((
            "Content-Range",
            format!("bytes 0-{}/{}", file_size - 1, file_size),
        ))
        .streaming(file_stream(file, config.buffer_size))
}

pub fn serve_partial_file(
    file: File,
    file_size: u64,
    range_header: &str,
    config: &StreamingConfig,
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
        .streaming(file_stream_partial(file, start, end, config.buffer_size))
}

pub fn file_stream(
    mut file: File,
    buffer_size: usize,
) -> impl futures::Stream<Item = Result<web::Bytes, std::io::Error>> {
    futures::stream::unfold(file, move |mut file| async move {
        let mut buffer = vec![0; buffer_size];
        match file.read(&mut buffer) {
            Ok(0) => None,
            Ok(n) => Some((Ok(web::Bytes::from(buffer[..n].to_vec())), file)),
            Err(e) => Some((Err(e), file)),
        }
    })
}

pub fn file_stream_partial(
    mut file: File,
    start: u64,
    end: u64,
    buffer_size: usize,
) -> impl futures::Stream<Item = Result<web::Bytes, std::io::Error>> {
    futures::stream::unfold((file, start), move |(mut file, start)| async move {
        if start > end {
            return None;
        }
        let mut buffer = vec![0; buffer_size];
        match file.seek(SeekFrom::Start(start)) {
            Ok(_) => (),
            Err(e) => return Some((Err(e), (file, start))),
        }
        match file.read(&mut buffer) {
            Ok(0) => None,
            Ok(n) => {
                let n = std::cmp::min(n as u64, end - start + 1) as usize;
                Some((
                    Ok(web::Bytes::from(buffer[..n].to_vec())),
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
