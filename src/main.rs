use std::str;
use std::io;
use std::net;
use std::env;
use std::fs;
use std::io::Read;
use std::time;
use std::thread;

fn main() {
    let config = Config::new(env::args());
    match config.mode {
        Mode::Client => client(&config).unwrap(),
        Mode::Server => server(&config).unwrap()
    };
}

fn server(config: &Config) -> Result<(), io::Error> {
    loop {
        let socket = net::UdpSocket::bind((config.local_ip, config.local_port))?;

        socket.set_read_timeout(Some(time::Duration::new(0, 30000)))?;

        let mut buffer = [0; 4];
        let remote_addr;
        match socket.recv_from(&mut buffer) {
            Ok((_, addr)) => {
                remote_addr = addr;
            }
            Err(_) => {
                continue;
            }
        }
        let str = str::from_utf8(&buffer).unwrap();

        if str.eq(&"helo".to_string()) {
            socket.connect(remote_addr)?;
            let mut i = 1;
            loop {
                println!("send packet no. {}", i);
                match socket.send(&generate_response_packet(false)) {
                    Ok(_) => {},
                    Err(_) => break
                };
                i += 1;

                if i % 100 == 0 {
                    let mut res_buffer = [0; 3];
                    match socket.recv(&mut res_buffer) {
                        Ok(_) => {
                            let res_str = str::from_utf8(&res_buffer).unwrap();
                            if res_str.eq(&"bye".to_string()) {
                                break;
                            }
                        },
                        Err(_) => {}
                    };
                }
            }
        }
        println!("connection closed");
    }
}

fn generate_response_packet(is_last: bool) -> [u8; 2048] {
    let mut result_buffer = [0; 2048];
    let mut random_file = fs::File::open("/dev/urandom").unwrap();
    random_file.read(&mut result_buffer).unwrap();
    let mut i = 0;
    let result_len = result_buffer.len();
    while i < result_len {
        result_buffer[i] = printable_char_encode(&result_buffer[i]);

        if is_last && (i == result_len - 1) {
            result_buffer[i] = 0;
        }
        i += 1;
    }
    result_buffer
}

fn printable_char_encode(n: &u8) -> u8 {
    let lower_range = n % 36;
    let result;
    if lower_range > 9 {
        result = lower_range + 7;
    } else {
        result = lower_range;
    }

    result + 48
}

fn client(config: &Config) -> Result<(), io::Error> {
    let socket = net::UdpSocket::bind((config.local_ip, config.local_port))?;
    socket.connect((config.remote_ip, config.remote_port))?;

    let helo = "helo".to_string().into_bytes();
    socket.send(&helo)?;

    let mut buff = [0; 2048];
    socket.recv(&mut buff).unwrap();

    let download_time = time::Instant::now();

    let mut packet_number = 0;
    loop {
        packet_number += 1;
        let mut buffer = [0; 2048];
        socket.recv(&mut buffer).unwrap();

        if packet_number > config.packet_amount {
            break;
        }
    }

    let raw_elapsed_ms = download_time.elapsed().subsec_millis();
    let elapsed_ms;
    if raw_elapsed_ms == 0 {
        elapsed_ms = 1;
    } else {
        elapsed_ms = raw_elapsed_ms;
    }
    let packets_per_seconds = f64::from(elapsed_ms * 1000) / f64::from(config.packet_amount);
    println!("Packets per second: {}", packets_per_seconds);

    loop {
        socket.set_read_timeout(Some(time::Duration::new(0, 30000)))?;
        match socket.recv(&mut buff) {
            Ok(_) => {},
            Err(_) => break,
        };
    }

    loop {
        let mut buffer = [0; 1];
        match socket.recv(&mut buffer) {
            Ok(_) => {
                println!("{:?}", buffer);
                println!("telling server to stop");
                let bye = "bye".to_string().into_bytes();
                socket.send(&bye);
                thread::sleep(time::Duration::new(1, 0));
            },
            Err(_) => {
                break;
            }
        };
    }

    Ok(())
}

enum Mode {
    Server,
    Client
}

struct Config {
    mode: Mode,
    packet_amount: u32,
    remote_ip: net::Ipv4Addr,
    remote_port: u16,
    local_ip: net::Ipv4Addr,
    local_port: u16,
}

impl Config {
    fn new(mut args: env::Args) -> Config {
        let raw_mode = args.nth(1).unwrap();

        let mode;
        let remote_port;
        let mut remote_ip = "127.0.0.1".parse().unwrap();
        let mut packet_amount = 0;
        let local_ip;
        let local_port;

        local_ip = match args.nth(0) {
            Some(ip) => ip.parse().unwrap(),
            None => "127.0.0.1".parse().unwrap()
        };

        if "server".eq(&raw_mode) {
            mode = Mode::Server;
            local_port = 1234;
            remote_port = 1235;
        } else if "client".eq(&raw_mode) {
            mode = Mode::Client;
            local_port = 1235;
            remote_port = 1234;

            match args.nth(0) {
                Some(ip) => remote_ip = ip.parse().unwrap(),
                None => {}
            };

            packet_amount = match args.nth(0) {
                Some(n) => n.parse().unwrap(),
                None => 255,
            };
        } else {
            panic!("Unsupported mode");
        }

        Config {
            mode,
            packet_amount,
            remote_ip,
            local_ip,
            remote_port,
            local_port,
        }
    }
}
