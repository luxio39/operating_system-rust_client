use std::io::prelude::*;
use std::io;
use std::collections::HashMap;
use std::net::{Shutdown, TcpStream, IpAddr};

fn prompt_input(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .expect("failed to read line");
    return input.trim_end().to_owned();
}

fn send_command(stream: &mut TcpStream, command: String) -> Result<HashMap<String, String>, String> {
    let mut rx_buffer = [0u8; 1024];
    stream.write_all(command.as_bytes()).expect("failed to send data to the server");

    let mut entrys = HashMap::new();

    loop {
        let read_characters: usize = stream.read(&mut rx_buffer).expect("failed to receive data from the server");
        let rx_data = std::str::from_utf8(&rx_buffer[0 .. read_characters]).expect("valid utf8");
        let packets = rx_data.trim_matches(char::from(0)).split("----------\n").collect::<Vec<&str>>();
        for packet in packets {
            let packet = packet.trim_end();
            if (packet.is_empty()) {
                continue;
            }
            let lines = packet.split("\n").collect::<Vec<&str>>();
            let mut response = HashMap::new();
            for line in lines {
                if line.is_empty() {
                    continue;
                }
                let tokens = line.split(": ").collect::<Vec<&str>>();
                if tokens.len() < 2 {
                    dbg!(rx_data);
                    return Err("server returned malformed data".to_string());
                }
                response.insert(tokens[0].to_string(), tokens[1].to_string());
            }
            if !response.contains_key("type") {
                return Err("malformed response from server".to_string());
            }
            if response.get("type").unwrap() == "err" {
                return Err("server responded with error".to_string());
            }
            if response.get("type").unwrap() == "ack" {
                return Ok(entrys);
            }
            if response.get("type").unwrap() == "entry" {
                entrys.insert(response.get("key").unwrap().to_string(), response.get("value").unwrap().to_string());
            }
        }
    }
}

fn main() {
    let mut ip_adress = prompt_input("Enter ip address (blank for localhost): ");
    if ip_adress.is_empty() {
        ip_adress.push_str("127.0.0.1");
    }
    let ip_adress : IpAddr = ip_adress.parse::<IpAddr>()
        .expect("unable to parse server ip address");

    println!("Connecting to {}...", ip_adress);
    let mut stream = TcpStream::connect((ip_adress, 8080))
        .expect("failed to connect to server");

    loop {
        let input = prompt_input("Enter command ('exit' to exit, 'help' for help): ");
        match input.trim_end() {
            "insert" => {
                let key = prompt_input("key: ");
                if key.is_empty() {
                    println!("key may not be empty!");
                    continue;
                }
                let value = prompt_input("value: ");
                if value.is_empty() {
                    println!("value may not be empty!");
                    continue;
                }
                send_command(&mut stream, format!("type: request\ncommand: insert\nkey: {key}\nvalue: {value}\n"))
                    .expect("communication with the server failed");
            },
            "query" => {
                let key = prompt_input("key: ");
                let response = send_command(&mut stream, format!("type: request\ncommand: query\nkey: {key}\n"))
                    .expect("communication with the server failed");
                if (!response.is_empty()) {
                    println!("entry(s):");
                    for (key, value) in &response {
                        println!("{key} --> {value}");
                    }
                } else {
                    println!("no entrys with key \"{key}\" found!");
                }

            },
            "remove" => {
                let key = prompt_input("key: ");
                send_command(&mut stream, format!("type: request\ncommand: remove\nkey: {key}\n"))
                    .expect("communication with the server failed");
            },
            "help" => {
                println!("Available commands: 'insert', 'query', 'remove'");
            },
            "exit" => break,
            _ => continue
        }
    }
    send_command(&mut stream, format!("type: disconnect\n"))
                    .expect("communication with the server failed");
    println!("Disconnect request acknowledged by server!");
    stream.shutdown(Shutdown::Both).expect("shutdown call failed");
}
