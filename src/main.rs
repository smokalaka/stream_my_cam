use opencv::{
    core::{Mat, Vector}, imgcodecs, prelude::*, videoio,
};

use std::net::TcpListener;
use std::io::Write;
use std::fmt::Display;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on port 8080");

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).expect("Failed to get video capture");
    let mut frame = Mat::default();
    let mut buf = Vector::new();

    loop {
        let (mut stream, requester_address) = listener.accept().expect("Failed to accept connection");

        println!("Requester address: {}", requester_address);

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
        );

        stream.write_all(response.as_bytes()).unwrap();

        loop {
            cam.read(&mut frame).expect("Failed to capture frame");
            buf.clear();
            let _ = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new());

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
        }
    }
}

fn print_error<T: Display>(error: T) {
    println!("Error sending data, probably connection terminated. Error: {}", error);
}