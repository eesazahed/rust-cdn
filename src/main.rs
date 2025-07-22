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

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer[..]);

            let path = if let Some(line)  = request.lines().next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    parts[1]
                } else {
                    "/"
                }
            } else {
                "/"
            };

            let base_dir = env::var("MP3_DIR").unwrap_or_else(|_| ".".to_string());
            let path = path.trim_start_matches("/");
            let full_path = Path::new(&base_dir).join(path);

            let (status_line, content, content_type) = if path == "" {
                match fs::read_dir(base_dir) {
                    Ok(entries) => {
                        let mut mp3s = Vec::new();
                        for entry in entries.flatten() {
                            let file_name = entry.file_name();
                            let name = file_name.to_string_lossy();
                            if name.ends_with(".mp3") {
                                mp3s.push(format!("\"{}\"", name));
                            }
                        }
                        let json = format!("[{}]", mp3s.join(","));
                        (
                            "HTTP/1.1 200 OK",
                            json.into_bytes(),
                            "application/json",
                        )
                    }
                    Err(_) => (
                        "HTTP/1.1 500 INTERNAL SERVER ERROR",
                        b"500 Internal Server Error".to_vec(),
                        "text/plain",
                    ),
                }
            } else if full_path.exists() && full_path.is_file() {
                match fs::read(&full_path) {
                    Ok(data) => ("HTTP/1.1 200 OK", data, "audio/mpeg"),
                    Err(_) => (
                        "HTTP/1.1 500 INTERNAL SERVER ERROR",
                        b"500 Internal Server Error".to_vec(),
                        "text/plain",
                    ),
                }
            } else {
                (
                    "HTTP/1.1 404 NOT FOUND",
                    b"404 Not Found".to_vec(),
                    "text/plain",
                )
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