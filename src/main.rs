use itertools::Itertools;
use std::collections::HashMap;
use std::env;
use std::{
    fmt::{self},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
}

const LINE_ENDING: &str = "\r\n";

impl FromStr for HttpRequest {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        let mut lines = s.lines();
        let first_line = lines.next().unwrap();
        let (method, path, version) = {
            let mut parts = first_line.split_whitespace();
            (
                parts.next().unwrap().to_string(),
                parts.next().unwrap().to_string(),
                parts.next().unwrap().to_string(),
            )
        };

        let mut headers: HashMap<String, String> = HashMap::new();
        for line in lines {
            if let Some((header, value)) = line.split_once(':') {
                headers.insert(
                    header.trim().to_lowercase().to_string(),
                    value.trim().to_string(),
                );
            }
        }

        Ok(Self {
            method,
            path,
            version,
            headers,
        })
    }
}

fn handle_stream(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    let res = stream.read(&mut buffer);
    let str = String::from_utf8_lossy(&buffer);
    if let Ok(req) = str.parse::<HttpRequest>() {
        println!(
            "Request: {}, path: {}, version: {}",
            req.method, req.path, req.version
        );
        let mut paths = req.path.split('/');
        if let Some(path) = paths.next() {
            match path {
                "" => {
                    if let Some(path2) = paths.next() {
                        match path2 {
                            "echo" => {
                                let content = paths.collect_vec().join("/");
                                ok_with_text_content(stream, content.as_str());
                            }
                            "user-agent" => {
                                let content = req.headers.get("user-agent").unwrap();
                                ok_with_text_content(stream, content.as_str());
                            }
                            "files" => {
                                let filename = paths.collect_vec().join("/");
                                if let Some(content) = get_file_content(filename.as_str()) {
                                    ok_with_octet_stream(stream, content.as_str());
                                } else {
                                    not_found(stream);
                                }
                            }
                            "" => {
                                empty_ok(stream);
                            }
                            _ => {
                                println!("not found {}", path);
                                not_found(stream);
                            }
                        }
                    } else {
                        not_found(stream);
                    }
                }
                _ => {
                    println!("not found {}", path);
                    not_found(stream);
                }
            }
        }
    } else {
        println!("error: {:?}", res);
    }
}

fn ok_with_octet_stream(stream: &mut TcpStream, content: &str) {
    let response = create_response(HttpStatusCode::Ok, "application/octet-stream", content);
    send_response(stream, response.as_str())
}

fn ok_with_text_content(stream: &mut TcpStream, content: &str) {
    let response = create_response(HttpStatusCode::Ok, "text/plain", content);
    send_response(stream, response.as_str())
}

fn not_found(stream: &mut TcpStream) {
    let response = create_response(HttpStatusCode::NotFound, "", "");
    send_response(stream, response.as_str())
}

fn empty_ok(stream: &mut TcpStream) {
    let response = create_response(HttpStatusCode::Ok, "", "");
    send_response(stream, response.as_str())
}

fn get_file_content(filename: &str) -> Option<String> {
    let cmd_args: Vec<String> = env::args().collect();
    let directory_path = &cmd_args[2];

    if let Ok(contents) = std::fs::read_to_string(format!("{}/{}", directory_path, filename)) {
        Some(contents)
    } else {
        None
    }
}

#[derive(Debug)]
enum HttpStatusCode {
    Ok,
    NotFound = 404,
}

impl fmt::Display for HttpStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpStatusCode::Ok => {
                write!(f, "200 OK")
            }
            HttpStatusCode::NotFound => {
                write!(f, "404 NOT FOUND")
            }
        }
    }
}

fn create_response(status_code: HttpStatusCode, content_type: &str, content: &str) -> String {
    let mut response = String::from("");
    response.push_str("HTTP/1.1 ");
    response.push_str(format!("{}{}", status_code, LINE_ENDING).as_str());
    if !content.is_empty() {
        response.push_str(format!("Content-Type: {}{}", content_type, LINE_ENDING).as_str());
        response.push_str(format!("Content-Length: {}{}", content.len(), LINE_ENDING).as_str());
        response.push_str(LINE_ENDING);
        response.push_str(content);
    } else {
        response.push_str(LINE_ENDING);
        response.push_str(LINE_ENDING);
    }

    response
}

fn send_response(stream: &mut TcpStream, response: &str) {
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    println!("Listening on port: 4221");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                std::thread::spawn(move || {
                    println!("New connection");
                    handle_stream(&mut stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
