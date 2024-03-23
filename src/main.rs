use opencv::{
    core::{Mat, Vector},
    imgcodecs,
    prelude::*,
    videoio,
};
use std::net::TcpListener;
use std::io::Write;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use crossbeam::channel;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to port 8080");
    println!("Server listening on port 8080");

    let (frame_sender, frame_receiver) = channel::bounded(10);
    let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).expect("Failed to get video capture");

    thread::spawn(move || {
        let cam_mutex = Arc::new(Mutex::new(cam));
        capture_frames(cam_mutex, frame_sender);
    });

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(mut stream) => {
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
                );
                if let Err(err) = stream.write_all(response.as_bytes()) {
                    print_error(err);
                    println!("Was not able to handle client");
                    return;
                }

                let frame_receiver_clone = frame_receiver.clone();

                thread::spawn(move || {
                    let mut stream = stream;
                    handle_client(&mut stream, frame_receiver_clone);
                });
            }
            Err(err) => {
                println!("Error accepting connection: {}", err);
            }
        }
    }
}

fn capture_frames(cam: Arc<Mutex<videoio::VideoCapture>>, frame_sender: channel::Sender<Vec<u8>>) {
    let mut frame = Mat::default();
    let mut buf = Vector::new();

    let mut cam = cam.lock().unwrap();

    loop {
        if let Err(err) = cam.read(&mut frame) {
            print_error(err);
            break;
        }

        if let Err(err) = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new()) {
            print_error(err);
            break;
        }

        if let Err(err) = frame_sender.send(buf.to_vec()) {
            println!("Error sending frame: {}", err);
            break;
        }
    }
}

fn handle_client(stream: &mut std::net::TcpStream, frame_receiver: channel::Receiver<Vec<u8>>) {
    println!("New client connected");

    loop {
        let frame_buffer = match frame_receiver.recv() {
            Ok(data) => data,
            Err(err) => {
                println!("Error receiving frame: {}", err);
                break;
            }
        };

        let image_data = format!(
            "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
            frame_buffer.len()
        );

        if let Err(err) = stream.write_all(image_data.as_bytes()) {
            print_error(err);
            break;
        }
        if let Err(err) = stream.write_all(&frame_buffer) {
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

fn print_error<T: Display>(error: T) {
    println!("Error sending data, probably connection terminated. Error: {}", error);
}