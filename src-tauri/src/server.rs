pub mod server_test {
    use std::{cmp::Ordering, io::{copy, Bytes, Read, Write}, net::{TcpListener, TcpStream}, thread::{spawn, JoinHandle}};
    use serde_json::{Result, Value};
    use local_ip_address::local_ip;

    
    pub fn server_start() -> JoinHandle<()>{
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
            println!("here");
            println!("{:?}", String::from_utf8_lossy(&incoming));
            println!("here");
            let Ok(new_client) = TcpStream::connect(tuple.1.to_string() + "7891") else {continue};
            
            let tcp_thread = spawn(move || {
                handle_request(new_client);
            });

            let _ = tcp_thread.join();
        }
    }


    fn handle_request(mut stream: TcpStream) {
        println!("handling request");
        let mut buf: Vec<u8> = Vec::new();
        stream.write(b"Connected").unwrap();

        let _ = stream.read_to_end(&mut buf).expect("Could not read from stream");
        println!("{:?}", String::from_utf8_lossy(&buf));
       
        buf.clear();
        loop{
            let Ok(_) = stream.read_to_end(&mut buf) else {
                println!("Connection interupted");
                break;
            };

            println!("{:?}", String::from_utf8_lossy(&buf));
            buf.clear();
        }
    }


   // fn handle_json_result() {//TODO
        //take in json from message
        //make it a struct
        //do something based on its qualities
            //ex: 
            //turn on stream
            //register device
   // }
} 