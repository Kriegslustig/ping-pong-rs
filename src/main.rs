use std::str;
use std::io;
use std::net;
use std::env;
use std::fs;
use std::io::Read;
use std::time;

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
        let mut buffer = [0; 5];
        let (_, remote_addr) = socket.recv_from(&mut buffer)?;
        let str = str::from_utf8(&buffer[0..4]).unwrap();
        let len = u8::from(buffer[4]);

        if str.eq(&"helo".to_string()) {
            socket.connect(remote_addr)?;
            let mut i = 1;
            while i < len {
                socket.send(&generate_response_packet(false))?;
                i += 1;
            }
            let response = generate_response_packet(true);
            socket.send(&response)?;
        }
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
    let mut mreq: [u8; 5] = [0; 5];
    let (left, right) = mreq.split_at_mut(4);
    left.copy_from_slice(&helo[0..4]);
    right[0] = config.packet_amount;
    let req = [left, right].concat();
    socket.send(&req)?;

    let download_time = time::Instant::now();

    loop {
        let mut buffer = [0; 2048];
        socket.recv(&mut buffer).unwrap();

        let response_str: String = buffer[0..buffer.len() - 1]
            .into_iter()
            .map(|c| {
                char::from(c.clone())
            })
            .collect();

        if buffer.ends_with(&[0]) {
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

    Ok(())
}

enum Mode {
    Server,
    Client
}

struct Config {
    mode: Mode,
    packet_amount: u8,
    remote_ip: net::Ipv6Addr,
    remote_port: u16,
    local_ip: net::Ipv6Addr,
    local_port: u16,
}

impl Config {
    fn new(mut args: env::Args) -> Config {
        let raw_mode = args.nth(1).unwrap();
        let opt_packet_amount = args.nth(0);

        let mode;
        let remote_port;
        let mut remote_ip = "::1".parse().unwrap();
        let local_port;
        if "server".eq(&raw_mode) {
            mode = Mode::Server;
            local_port = 1234;
            remote_port = 1235;
        } else if "client".eq(&raw_mode) {
            let opt_remote_ip = args.nth(0);
            println!("{:?}", args);
            mode = Mode::Client;
            local_port = 1235;
            remote_port = 1234;
            remote_ip = opt_remote_ip.unwrap().parse().unwrap();
        } else {
            panic!("Unsupported mode");
        }

        let packet_amount = match opt_packet_amount {
            Some(n) => n.parse().unwrap(),
            None => u8::from(255),
        };

        Config {
            mode,
            packet_amount,
            remote_ip,
            local_ip: "::1".parse().unwrap(),
            remote_port,
            local_port,
        }
    }
}
