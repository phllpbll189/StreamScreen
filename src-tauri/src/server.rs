pub mod server_test {
    use std::{cmp::Ordering, io::Read, net::{TcpListener, TcpStream}, thread::{spawn, JoinHandle}};
    use serde_json::{Result, Value};
    use local_ip_address::local_ip;

    
    pub fn server_start() -> JoinHandle<()>{
        let local_host = local_ip().unwrap().to_string();

        let tcp_thread = spawn(move || {
                let tcplisten = TcpListener::bind(local_host+":7890").unwrap();
                let incoming = tcplisten.accept();

                match incoming {
                    Ok((stream, address)) => {
                        let mut stream = stream;
                        println!("this is the address: {}", address);
                        handle_request(&mut stream) 
                    },
                    Err(e) => println!("{}", e)
                }
        });

        tcp_thread
    }


    fn handle_request(stream: &mut TcpStream) {
        let mut buffer = [0;10];
        let mut message = Vec::new();

        let len = stream.read(&mut buffer).unwrap();
        let (message_length, initial) = get_message_size(&mut buffer, len);
        let len: u16 = u16::try_from(len).unwrap();
        
        match message_length.cmp(&len) {
            Ordering::Less => message.extend(&buffer[..usize::from(len)]),
            Ordering::Greater => {
                message.extend(&buffer[initial..]);
                message.extend(
                    read_rest_of_message(
                        stream, 
                        message_length-(len - u16::try_from(initial).unwrap()),
                        &mut message.clone()
                    )
                );
            },
            Ordering::Equal => message.extend(&buffer[initial..])
        }

        println!("{}", String::from_utf8_lossy(&message));
    }

     
    //second return number is where the leading bytes ends and message starts.
    fn get_message_size(buffer: &[u8;10], len: usize) -> (u16, usize) {
            for i in 0..len{
                if buffer[i] == b':'{
                    return (u16::from_str_radix(&String::from_utf8_lossy(&buffer[..i]), 10).unwrap(), i+1);
                } 
            }       
        return (0, 0);
    }


    fn handle_json_result() {//TODO
        //take in json from message
        //make it a struct
        //do something based on its qualities
            //ex: 
            //turn on stream
            //register device
    }
    

    //reads to the end of the client-set size variable.
    fn read_rest_of_message(stream: &mut TcpStream, length: u16, message: &mut Vec<u8>) -> Vec<u8> {
        let mut buf = [0; 10];
        let mut length = length;

        while length > 0{
           let len = u16::try_from(stream.read(&mut buf).unwrap()).unwrap();
            
            match length.cmp(&len){
                Ordering::Less => {
                    message.extend(&buf[..usize::try_from(length).unwrap()]);
                    length = 0;
                },
                Ordering::Greater => {
                    message.extend(buf);
                    length = length - len;
                },
                Ordering::Equal => {
                    message.extend(&buf)
                }
            }
        }
        
        return message.to_vec();
    }
}
