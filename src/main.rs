use std::collections::HashMap;
// Uncomment this block to pass the first stage
use std::fmt::{self, format, Debug, Display};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

use itertools::Itertools;
struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
}

const LINE_ENDING: &str = "\r\n";

/*
GET /index.html HTTP/1.1
Host: localhost:4221
User-Agent: curl/7.64.1
*/
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
            println!("{}", line);
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

/*
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 3

abc
*/
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
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                std::thread::spawn(move || {
                    println!("accepted new connection");
                    handle_stream(&mut stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
