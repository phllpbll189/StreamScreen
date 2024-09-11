use screenshots::Screen;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new("127.0.0.1")?;
    let screen = Screen::from_point(0, 0)?;

    // Start a UDP stream
    client.start_udp_stream(8000)?;

    // wait for the server to start
    thread::sleep(Duration::from_secs(1));

    // Capture and send frames
    loop {
        let image = screen.capture()?;
        let buffer = image.buffer();

        // Send frame data through UDP
        client.send_udp(&buffer)?;
        println!("Sent frame: {} bytes", buffer.len());

        // Add a small delay to control frame rate
        thread::sleep(Duration::from_millis(33)); // ~30 fps
    }

    // When done
    client.stop_udp_stream(8000)?;

    Ok(())
}

use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write};
use serde_json::json;

pub struct Client {
    tcp_stream: TcpStream,
    udp_socket: Option<UdpSocket>,
}

impl Client {
    pub fn new(server_address: &str) -> std::io::Result<Self> {
        let tcp_stream = TcpStream::connect(format!("{}:7890", server_address))?;
        Ok(Client {
            tcp_stream,
            udp_socket: None,
        })
    }

    pub fn send_message(&mut self, action: &str, device_id: Option<&str>, udp_port: Option<u16>) -> std::io::Result<()> {
        let message = json!({
            "action": action,
            "device_id": device_id,
            "udp_port": udp_port,
        });

        let message_str = serde_json::to_string(&message)?;
        self.tcp_stream.write_all(message_str.as_bytes())?;
        Ok(())
    }

    pub fn start_udp_stream(&mut self, port: u16) -> std::io::Result<()> {
        self.send_message("start_stream", None, Some(port))?;
        self.udp_socket = Some(UdpSocket::bind(format!("0.0.0.0:{}", port))?);
        Ok(())
    }

    pub fn stop_udp_stream(&mut self, port: u16) -> std::io::Result<()> {
        self.send_message("stop_stream", None, Some(port))?;
        self.udp_socket = None;
        Ok(())
    }

    pub fn receive_udp(&self) -> std::io::Result<Vec<u8>> {
        if let Some(udp_socket) = &self.udp_socket {
            let mut buf = [0u8; 65507];
            let (size, _) = udp_socket.recv_from(&mut buf)?;
            Ok(buf[..size].to_vec())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "UDP stream not started"))
        }
    }

    pub fn send_udp(&self, data: &[u8]) -> std::io::Result<()> {
        if let Some(udp_socket) = &self.udp_socket {
            udp_socket.send(data)?;
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "UDP stream not started"))
        }
    }
}