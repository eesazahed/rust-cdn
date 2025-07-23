use std::{env, fs, io::{BufRead, BufReader, Read, Write}};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn main() {
    load_env_file(".env");

    let port = env::var("PORT").unwrap_or_else(|_| "12345".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).unwrap();
    println!("Server running on http://{}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

fn load_env_file(path: &str) {
    if let Ok(file) = fs::File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Some((key, val)) = parse_env_line(&line) {
                unsafe { env::set_var(key, val) };
            }
        }
    } else {
        eprintln!("Failed to open .env file at {}", path);
    }
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("#") {
        return None;
    }
    let mut parts = line.splitn(2, "=");
    let key = parts.next()?.trim().to_string();
    let val = parts.next()?.trim().trim_matches('"').to_string();
    Some((key, val))
}


fn url_encode(input: &str) -> String {
    input.chars().map(|c| {
        match c {
            ' ' => "%20".to_string(),
            '"' => "%22".to_string(),
            '#' => "%23".to_string(),
            c => c.to_string(),
        }
    }).collect()
}

fn url_decode(input: &str) -> String {
    let mut chars = input.chars().peekable();
    let mut output = String::new();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hi = chars.next();
            let lo = chars.next();

            if let (Some(hi), Some(lo)) = (hi, lo) {
                let hex = format!("{}{}", hi, lo);
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    output.push(byte as char);
                } else {
                    output.push('%');
                    output.push(hi);
                    output.push(lo);
                }
            } else {
                output.push(c);
            }
        } else {
            output.push(c);
        }
    }

    output
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer[..]);

            let path = if let Some(line) = request.lines().next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    parts[1]
                } else {
                    "/"
                }
            } else {
                "/"
            };

            let static_dir = "static";
            let mp3_dir = env::var("MP3_DIR").unwrap_or_else(|_| ".".to_string());

            let decoded_path = url_decode(path);
            let trimmed_path = decoded_path.trim_start_matches("/");

            let (status_line, content, content_type) = if trimmed_path == "" || trimmed_path == "index.html" {
                let index_path = Path::new(static_dir).join("index.html");
                match fs::read(&index_path) {
                    Ok(data) => ("HTTP/1.1 200 OK", data, "text/html"),
                    Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR", b"500 Internal Server Error".to_vec(), "text/plain"),
                }
            }
        
            else if trimmed_path.starts_with("assets/") {
                let file_path = Path::new(static_dir).join(trimmed_path);
                if file_path.exists() && file_path.is_file() {
                    let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
                        Some("css") => "text/css",
                        Some("js") => "application/javascript",
                        Some("html") => "text/html",
                        _ => "application/octet-stream",
                    };
                    match fs::read(&file_path) {
                        Ok(data) => ("HTTP/1.1 200 OK", data, content_type),
                        Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR", b"500 Internal Server Error".to_vec(), "text/plain"),
                    }
                } else {
                    ("HTTP/1.1 404 NOT FOUND", b"404 Not Found".to_vec(), "text/plain")
                }
            }

            else if trimmed_path.starts_with("tracks/") {
                let tracks_slug = trimmed_path.trim_start_matches("tracks/");
                let file_path = Path::new(&mp3_dir).join(tracks_slug);

                if file_path.exists() && file_path.is_file() {
                    match fs::read(&file_path) {
                        Ok(data) => ("HTTP/1.1 200 OK", data, "audio/mpeg"),
                        Err(_) => ("HTTP1/1.1 500 INTERNAL SERVER ERROR", b"500 Internal Server Error".to_vec(), "text/plain")
                    }
                } else {
                    ("HTTP/1.1 404 NOT FOUND", b"404 Not Found".to_vec(), "text/plain")
                }
            }

            else if trimmed_path == "tracks" {
                match fs::read_dir(&mp3_dir) {
                    Ok(entries) => {
                        let mut mp3s = Vec::new();
                        for entry in entries.flatten() {
                            let file_name = entry.file_name();
                            let name = file_name.to_string_lossy();

                            if name.ends_with(".mp3") {
                                let encoded = url_encode(&name);
                                mp3s.push(format!("\"{}\"", encoded));
                            }
                        }
                        let json = format!("[{}]", mp3s.join(","));
                        ("HTTP/1.1 200 OK", json.into_bytes(), "application/json")
                    }
                    Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR", b"500 Internal Server Error".to_vec(), "text/plain"),
                }
            }

            else {
                ("HTTP/1.1 404 NOT FOUND", b"404 Not Found".to_vec(), "text/plain")
            };

            let headers = format!(
                "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                status_line,
                content_type,
                content.len()
            );

            let mut response = headers.as_bytes().to_vec();
            response.extend_from_slice(&content);

            if let Err(e) = stream.write_all(&response) {
                eprintln!("Failed to write to stream: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to read from connection: {}", e),
    }
}