use std::net::TcpStream;

pub mod server_test {
    use std::{cmp::Ordering, io::{Read, Write}, net::{TcpListener, TcpStream}, thread::{spawn, JoinHandle}, usize};
    use serde_json::{Result, Value};
    use local_ip_address::local_ip;

    
    pub fn server_start() -> JoinHandle<()>{
        loop {
            let local_host = local_ip().unwrap().to_string();
            let tcplisten = TcpListener::bind(local_host+":7890").unwrap();

            let tcp_thread = spawn(move || {
                handle_request(tcplisten);
            });


        }
    }


    fn handle_request(incoming: TcpListener) {
        let stream = incoming.accept();

        let mut stream = match stream {
            Ok((stream, address)) => {
                println!("this is the address of the client: {}", address);
                stream
            },
            Err(_) => panic!("could not accept the request")
        };

        let mut buf: Vec<u8> = Vec::new();
        let _ = stream.read_to_end(&mut buf).expect("Could not read from stream");
        println!("{:?}", String::from_utf8(buf));
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