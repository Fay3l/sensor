use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

#[tokio::main]
async fn main() {
    match tokio_serial::new("COM8", 256000).open_native_async() {
        Ok(mut stream) => {
            println!("Port opened");
            let command = [0xFD,0xFC,0xFB,0xFA,0x04,0x00,0xFF,0x00,0x01,0x00,0x04,0x03,0x02,0x01];
            match stream.write(&command).await {
                Ok(_) => {
                    println!("Command sent: {:02X?}", command);
                }
                Err(e) => {
                    eprintln!("Failed to send command: {:?}", e);
                    return;
                }
            }
            let mut buf = [0u8; 1000];
            loop {
                match stream.read(&mut buf).await {
                    Ok(n) => {
                        println!("Received: {:2x?}", &buf[..n]);
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}

// TF-Luna commands
async fn output_format_setting(format: u8, stream: &mut SerialStream) {
    let command: [u8; 5] = [0x5A, 0x05, 0x05, format, 0x00];
    match stream.write(&command).await {
        Ok(_) => {
            println!("Command sent: {:02X?}", command);
        }
        Err(e) => {
            eprintln!("Failed to send command: {:?}", e);
            return;
        }
    }
}

async fn distance_limit_setting(dist_min: u16, dist_max: u16, stream: &mut SerialStream) {
    let command = [
        0x5A,
        0x09,
        0x3A,
        dist_min.to_be_bytes()[1],
        dist_min.to_be_bytes()[0],
        dist_max.to_be_bytes()[1],
        dist_max.to_be_bytes()[0],
        0x00,
        0x00,
    ];
    match stream.write(&command).await {
        Ok(_) => {
            println!("Command sent: {:02X?}", command);
        }
        Err(e) => {
            eprintln!("Failed to send command: {:?}", e);
            return;
        }
    }
}

async fn output_frequency(freq: u16, stream: &mut SerialStream) {
    let command = [
        0x5A,
        0x06,
        0x3,
        freq.to_be_bytes()[1],
        freq.to_be_bytes()[0],
        0x00,
    ];
    match stream.write(&command).await {
        Ok(_) => {
            println!("Command sent: {:02X?}", command);
        }
        Err(e) => {
            eprintln!("Failed to send command: {:?}", e);
            return;
        }
    }
}
