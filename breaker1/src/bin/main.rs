use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use breaker1::{ThreadPool, BreakerError};

fn main_old() -> Result<(), BreakerError>{
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4)?;
    
    for stream in listener.incoming() {
        pool.execute(|| {
            handle_connection(stream.unwrap());
        });
    }

    Ok(())
}

// fn main_limited() -> Result<(), BreakerError> {
//     let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
//     let pool = ThreadPool::new(4)?;

//     for stream in listener.incoming().take(2) {
//         pool.execute(|| {
//             handle_connection(stream.unwrap());
//         });
//     }

//     println!("Shutting down.");
//     Ok(())
// }

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let home = b"GET / HTTP/1.1\r\n";
    let other = b"GET /other HTTP/1.1\r\n";
    let contents: String;
    let mut status = String::from("200 OK");

    if buffer.starts_with(home) {
        contents = fs::read_to_string("html/index.html").unwrap();
    } else if buffer.starts_with(other) {
        contents = fs::read_to_string("html/other.html").unwrap();
    } else {
        contents = fs::read_to_string("html/404.html").unwrap();
        status = String::from("404 NOT FOUND");
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}