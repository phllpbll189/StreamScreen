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

    #[derive(Deserialize, Serialize)]
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
            let mut incoming: Vec<u8> = Vec::new();

            let local_host = local_ip().unwrap().to_string();
            let tcplisten = TcpListener::bind(local_host+":7890").unwrap();
            let Ok(mut tuple) = tcplisten.accept() else {
                println!("Could not accept socket connection");
                continue
            };

            let Ok(_) = tuple.0.read_to_end(&mut incoming) else {
               println!("could recieve from stream");
               continue
            };

            println!("{:?}", String::from_utf8_lossy(&incoming));

            let Ok(new_client) = TcpStream::connect(tuple.1.to_string() + "7891") else {continue};

            let app_handle_clone = Arc::clone(&app_handle);
            let tcp_thread = spawn(move || {
                handle_request(new_client, app_handle_clone);
            });

            let _ = tcp_thread.join();
        }
    }


    fn handle_request(mut stream: TcpStream, app_handle: Arc<tauri::AppHandle>) {
        println!("handling request");
        let mut buf: Vec<u8> = Vec::new();
        stream.write(b"Connected").unwrap();

        buf.clear();
        loop{
            let Ok(_) = stream.read_to_end(&mut buf) else {
                println!("Connection interupted");
                break;
            };
            let result = handle_json_string(&String::from_utf8_lossy(&buf), app_handle.clone());
            println!("{:?}", result);
            buf.clear();
        }
    }


    pub fn handle_json_string(json_str: &str, app_handle: Arc<tauri::AppHandle>) -> Result<()> {
        let message: Message = serde_json::from_str(json_str)?;

        match message.action.as_str() {
            "start_stream" => {
                println!("Starting stream");
                if let Some(port) = message.udp_port {
                    println!("Starting stream on port {}", port);
                    match start_udp_stream(port, Arc::clone(&app_handle)) {
                        Ok(_) => Ok(()),
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
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
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