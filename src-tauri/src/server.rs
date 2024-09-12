pub mod server {
    // functions need to be public to be tested aka cringe
    use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, thread::{spawn, JoinHandle}};
    use serde_json::Result;
    use local_ip_address::local_ip;
    use serde::{de::Error, Deserialize, Serialize};
    use tauri::Manager;
    use std::sync::Arc;
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::net::UdpSocket;
    use std::sync::mpsc::{channel, Sender, Receiver};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Deserialize, Serialize, Debug)]
    struct Message {
        action: String,
        device_id: Option<String>,
        udp_port: Option<u16>,
    }

    lazy_static! {
        static ref ACTIVE_SOCKETS: Mutex<HashMap<u16, Sender<()>>> = Mutex::new(HashMap::new());
    }


    pub fn server_start(app_handle: tauri::AppHandle) -> JoinHandle<()> {
        let app_handle = Arc::new(app_handle);

        loop {
            let local_host = local_ip().unwrap().to_string();
            let tcplisten = TcpListener::bind(local_host+":7890").unwrap();

            let Ok(tuple) = tcplisten.accept() else {
                println!("Could not accept socket connection");
                continue
            };

            let app_handle_clone = Arc::clone(&app_handle);
            let tcp_thread = spawn(move || {
                handle_request(tuple.0, app_handle_clone);
            });

            let _ = tcp_thread.join();
        }
    }


    fn handle_request(mut stream: TcpStream, app_handle: Arc<tauri::AppHandle>) {
        println!("handling request");
        stream.write(b"Connected").unwrap();

        let mut buf = [0; 1024]; // Fixed-size buffer
        loop {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
            println!("reading from stream at {:?}", current_time);

            match stream.read(&mut buf) {
                Ok(size) if size > 0 => {
                    let received = &buf[..size];
                    let result = handle_json_string(&String::from_utf8_lossy(received), app_handle.clone());
                    println!("result: {:?}", result);
                }
                Ok(0) => {
                    println!("Connection closed by client at {:?}", current_time);
                    break;
                }
                Ok(_) => (), // Handle any other Ok case (though this should not occur)
                Err(e) => {
                    println!("Error reading from stream: {:?}", e);
                    break;
                }
            }
        }
    }


    pub fn handle_json_string(json_str: &str, app_handle: Arc<tauri::AppHandle>) -> Result<()> {
        let message: Message = serde_json::from_str(json_str)?;
        println!("message: {:?}", message);
        match message.action.as_str() {
            "start_stream" => {
                println!("Starting stream");

                if let Some(port) = message.udp_port {
                    println!("Starting stream on port {}", port);
                    match start_udp_stream(port, Arc::clone(&app_handle)) {
                        Ok(_) => {
                            println!("UDP stream started on port {}", port);
                            Ok(())
                        },
                        Err(e) => {
                            println!("Failed to start UDP stream: {}", e);
                            Err(serde_json::Error::custom("failed to start UDP stream"))
                        }
                    }
                } else {
                    println!("Port not provided for UDP stream");
                    Err(serde_json::Error::custom("port not provided"))
                }
            }
            "stop_stream" => {
                println!("Stopping stream");
                if let Some(port) = message.udp_port {
                    match stop_udp_stream(port) {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            println!("Failed to stop UDP stream: {}", e);
                            Err(serde_json::Error::custom("failed to stop UDP stream"))
                        }
                    }
                } else {
                    println!("Port not provided for UDP stream");
                    Err(serde_json::Error::custom("port not provided"))
                }
            }
            _ => {
                println!("Unknown action: {}", message.action);
                Err(serde_json::Error::custom("unknown action"))
            }
        }
    }




    pub fn start_udp_stream(port: u16, app_handle: Arc<tauri::AppHandle>) -> std::io::Result<()> {
        let local_ip = default_net::get_default_interface().unwrap().ipv4[0].addr.to_string();
        let socket = UdpSocket::bind(format!("{}:{}", local_ip, port))?;
        println!("UDP stream started on port {}", port);

        let (tx, rx): (Sender<()>, Receiver<()>) = channel();

        {
            let mut sockets = ACTIVE_SOCKETS.lock().unwrap();
            sockets.insert(port, tx);
        }

        std::thread::spawn(move || {
            let mut buf = [0u8; 65507];  // Maximum UDP packet size
            loop {
                if rx.try_recv().is_ok() {
                    println!("Stopping UDP stream on port {}", port);
                    break;
                }

                match socket.recv_from(&mut buf) {
                    Ok((size, src)) => {
                        println!("Received {} bytes from {}", size, src);
                        // Create a new vector with device identifier and frame data
                        let mut frame_with_id = Vec::new();
                        frame_with_id.extend_from_slice(src.to_string().as_bytes());
                        frame_with_id.push(0); // Null byte separator
                        frame_with_id.extend_from_slice(&buf[..size]);

                        app_handle.emit_all("frame_data", frame_with_id).unwrap();
                    }
                    Err(e) => {
                        println!("Error receiving UDP packet: {}", e);
                        break;
                    }
                }
            }

            let mut sockets = ACTIVE_SOCKETS.lock().unwrap();
            sockets.remove(&port);
        });

        Ok(())
    }

    pub fn stop_udp_stream(port: u16) -> std::io::Result<()> {
        let mut sockets = ACTIVE_SOCKETS.lock().unwrap();
        if let Some(tx) = sockets.remove(&port) {
            tx.send(()).unwrap();
            println!("Sent stop signal to UDP stream on port {}", port);
            Ok(())
        } else {
            println!("No active stream on port {}", port);
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No active stream on this port"))
        }
    }
}