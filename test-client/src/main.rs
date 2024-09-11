use screenshots::Screen;
use std::time::Duration;
use std::thread;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use default_net::get_default_interface;
use ipnetwork::Ipv4Network;
use rayon::prelude::*;
use std::cmp;
use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

fn find_server() -> Option<String> {
    let interface = get_default_interface().ok()?;
    let gateway = interface.gateway.unwrap().ip_addr;

    if let IpAddr::V4(gateway_v4) = gateway {
        let network = Ipv4Network::new(gateway_v4, 24).ok()?;

        println!("gateway: {}", gateway_v4);
        println!("network: {:?}", network);

        let server_ip = network.iter().par_bridge().find_any(|&ip| {
            let socket_addr = SocketAddr::new(IpAddr::V4(ip), 7890);
            println!("Checking connection to: {}", socket_addr);
            match TcpStream::connect_timeout(&socket_addr, Duration::from_millis(100)) {
                Ok(_) => {
                    println!("Connection successful to: {}", socket_addr);
                    true
                }
                Err(_) => {
                    println!("Connection failed to: {}", socket_addr);
                    false
                }
            }
        });

        println!("server_ip: {:?}", server_ip);

        server_ip.map(|ip| ip.to_string())
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address = match find_server() {
        Some(addr) => addr,
        None => {
            eprintln!("Failed to find server. Using localhost as fallback.");
            "127.0.0.1".to_string()
        }
    };

    let mut client = match Client::new(&server_address) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            eprintln!("Ensure the server is running and listening on 127.0.0.1:7890");
            return Err(e.into());
        }
    };

    let screen = Screen::all().expect("Failed to get screens");
    let primary_screen = screen.into_iter().next().expect("No screens found");

    // Start a UDP stream
    client.start_udp_stream(8000).expect("Failed to start UDP stream");

    // wait for the server to start
    thread::sleep(Duration::from_secs(1));
    loop {
        // Capture and send frames
        let image = primary_screen.capture()?;
        let buffer = image.buffer();

        // Send frame data through UDP in chunks
        client.send_udp_chunked(&buffer, 1400)?;
        println!("Sent frame: {} bytes", buffer.len());

        // Add a small delay to control frame rate
        thread::sleep(Duration::from_millis(100)); // ~16 fps

    }



    Ok(())
}



pub struct Client {
    tcp_stream: TcpStream,
    udp_socket: Option<UdpSocket>,
    server_address: String,
}

impl Client {
    pub fn new(server_address: &str) -> std::io::Result<Self> {
        let tcp_stream = TcpStream::connect(format!("{}:7890", server_address))?;
        Ok(Client {
            tcp_stream,
            udp_socket: None,
            server_address: server_address.to_string(),
        })
    }

    pub fn send_message(&mut self, action: &str, device_id: Option<&str>, udp_port: Option<u16>) -> std::io::Result<()> {
        let message = json!({
            "action": action,
            "device_id": device_id,
            "udp_port": udp_port,
        });

        let message_str = serde_json::to_string(&message).expect("Failed to convert message to string");
        self.tcp_stream.write_all(message_str.as_bytes()).expect("Failed to send message");
        self.tcp_stream.flush().expect("Failed to flush stream");

        println!("Sent message: {}", message_str);

        Ok(())
    }

    pub fn start_udp_stream(&mut self, port: u16) -> std::io::Result<()> {

        // is this the correct way to get the hostname?
        // is hostname good enough, or do we need the local ip?
        let hostname = gethostname::gethostname().to_string_lossy().into_owned();

        println!("Sending start_stream message");

        self.send_message("start_stream", Some(&hostname), Some(port)).expect("Failed to send start_stream message");

        let local_ip = default_net::get_default_interface().unwrap().ipv4[0].addr.to_string();

        println!("local_ip: {}", local_ip);

        let udp_socket = UdpSocket::bind(format!("{}:{}", local_ip, port)).expect("Failed to bind UDP socket");

        udp_socket.connect(format!("{}:8000", self.server_address)).expect("Failed to connect to server");
        println!("udp_socket: {:?}", udp_socket);

        self.udp_socket = Some(udp_socket);
        Ok(())
    }

    pub fn stop_udp_stream(&mut self, port: u16) -> std::io::Result<()> {
        self.send_message("stop_stream", None, Some(port))?;
        self.udp_socket = None;
        Ok(())
    }

    pub fn send_udp(&self, data: &[u8]) -> std::io::Result<()> {
        if let Some(udp_socket) = &self.udp_socket {
            udp_socket.send(data)?;
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "UDP stream not started"))
        }
    }

    pub fn send_udp_chunked(&self, data: &[u8], chunk_size: usize) -> std::io::Result<()> {
        if let Some(udp_socket) = &self.udp_socket {
            for chunk in data.chunks(chunk_size) {
                udp_socket.send(chunk)?;
            }
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "UDP stream not started"))
        }
    }
}