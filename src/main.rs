// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

struct HttpRequest {
    method: String,
    path: String,
    version: String,
}

/*
GET /index.html HTTP/1.1
Host: localhost:4221
User-Agent: curl/7.64.1
*/
impl FromStr for HttpRequest {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

        Ok(Self {
            method,
            path,
            version,
        })
    }
}

fn handle_stream(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    let res = stream.read(&mut buffer);
    let str = String::from_utf8_lossy(&buffer);
    if let Ok(req) = str.parse::<HttpRequest>() {
        println!(
            "method: {}, path: {}, version: {}",
            req.method, req.path, req.version
        );
        match req.path.as_str() {
            "/" => {
                stream
                    .write_all("HTTP/1.1 200 OK \r\n\r\n".as_bytes())
                    .unwrap();
                stream.flush().unwrap();
            }
            _ => {
                stream
                    .write_all("HTTP/1.1 404 NOT FOUND \r\n\r\n".as_bytes())
                    .unwrap();
                stream.flush().unwrap();
            }
        }
    } else {
        println!("error: {:?}", res);
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                handle_stream(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
