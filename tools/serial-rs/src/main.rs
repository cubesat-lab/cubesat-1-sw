use clap::Parser;
use serialport::SerialPort;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const PACKET_LEN: usize = 64;

/// Command-line arguments
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    port: String,

    #[arg(short, long, default_value_t = 115200)]
    baudrate: u32,

    #[arg(short, long, default_value = "loopback_usb")]
    test: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let port = serialport::new(&args.port, args.baudrate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open serial port");

    let port = Arc::new(Mutex::new(port));
    let port_clone = Arc::clone(&port);

    // Communication thread
    thread::spawn(move || loopback_test(port_clone));

    // Main thread waits forever
    loop {
        thread::park();
    }
}

/// Generate a test packet: [0, 1, 2, ..., 63]
fn build_packet() -> Vec<u8> {
    (0..PACKET_LEN as u8).collect()
}

/// Perform loopback test: send + receive + verify + measure throughput
fn loopback_test(port: Arc<Mutex<Box<dyn SerialPort>>>) {
    let mut bytes_sent: usize = 0;
    let mut bytes_received: usize = 0;
    let mut last_print = Instant::now();

    loop {
        let packet = build_packet();

        {
            let mut port = port.lock().unwrap();
            port.write_all(&packet).expect("Failed to write packet");
        }
        bytes_sent += packet.len();

        let mut received = vec![0u8; PACKET_LEN];
        let mut received_len = 0;

        while received_len < PACKET_LEN {
            let mut port = port.lock().unwrap();
            match port.read(&mut received[received_len..]) {
                Ok(n) => {
                    received_len += n;
                    bytes_received += n;
                }
                Err(_) => break,
            }
        }

        if received[..received_len] != packet {
            println!("Mismatch:");
            println!("  Received: {:02X?}", &received[..received_len]);
            println!("  Expected: {:02X?}", &packet);
        }

        // Print data rate once per second
        let now = Instant::now();
        if now.duration_since(last_print).as_secs_f64() >= 1.0 {
            let kbps = (bytes_sent + bytes_received) as f64 / 1024.0;
            println!("Data rate: {:.1} kB/s", kbps);
            bytes_sent = 0;
            bytes_received = 0;
            last_print = now;
        }
    }
}
