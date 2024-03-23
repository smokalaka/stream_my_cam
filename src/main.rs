use opencv::{
    core::{Mat, Vector}, imgcodecs, prelude::*, videoio,
};

use std::net::TcpListener;
use std::io::Write;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to port 8080");
    println!("Server listening on port 8080");

    let cam = Arc::new(Mutex::new(videoio::VideoCapture::new(0, videoio::CAP_ANY).expect("Failed to get video capture")));

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(mut stream) => {
                let cam = cam.clone();
                let _ = thread::spawn(move || {
                    handle_client(&mut stream, &cam);
                });
            }
            Err(err) => {
                println!("Error accepting connection: {}", err);
            }
        }
    }
}

fn handle_client(stream: &mut std::net::TcpStream, cam: &Arc<Mutex<videoio::VideoCapture>>) {
    println!("New client connected");

    loop {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
        );
        if let Err(err) = stream.write_all(response.as_bytes()) {
            print_error(err);
            break;
        }

        loop {
            let mut cam = cam.lock().expect("Failed to lock VideoCapture");

            let mut frame = Mat::default();
            let mut buf = Vector::new();

            if let Err(err) = cam.read(&mut frame) {
                print_error(err);
                break;
            }

            buf.clear();
            if let Err(err) = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new()) {
                print_error(err);
                break;
            }

            let image_data = format!(
                "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                buf.len()
            );

            if let Err(err) = stream.write_all(image_data.as_bytes()) {
                print_error(err);
                break;
            }
            if let Err(err) = stream.write_all(buf.as_slice()) {
                print_error(err);
                break;
            }
            if let Err(err) = stream.write_all(b"\r\n") {
                print_error(err);
                break;
            }
            if let Err(err) = stream.flush() {
                print_error(err);
                break;
            }

            drop(cam);
        }
    }
}

fn print_error<T: Display>(error: T) {
    println!("Error sending data, probably connection terminated. Error: {}", error);
}