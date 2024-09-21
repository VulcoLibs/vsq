use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;

mod dns;

#[tokio::main]
async fn main() {
    println!("DNS lookup start");
    let dns = dns::DNS::new().await;
    let master_ip = dns.lookup_ip("hl2master.steampowered.com").await.unwrap();
    println!("DNS lookup end");

    let master_addr = SocketAddr::new(
        master_ip,
        27011
    );

    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();
    println!("UDP binned");

    sock.connect(master_addr).await.unwrap();
    println!("UDP connected");

    let mut addr_map = Vec::new();

    let mut wait = true;
    let mut buf = [0u8; 3000];
    let mut seed = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 0);
    while wait {
        let mut packet = vec![0x31];
        packet.push(0xFF);
        packet.extend(seed.to_string().into_bytes());
        packet.extend("\0".as_bytes());
        packet.extend("\\appid\\252490\0".as_bytes());

        sock.send(&packet).await.unwrap();
        println!("UDP sent");

        let len = sock.recv(&mut buf).await.unwrap();

        if len == 0 {
            break;
        }

        let read_buf = &buf[..len];

        for raw_addr in read_buf.chunks(6) {
            let addr = parse_raw_addr(raw_addr);

            if addr_map.contains(&addr) {
                println!("[CLONE] {}", addr);
            } else {
                addr_map.push(addr);
                println!("[ OK  ] {}", addr);
            }

            if addr.ip() == IpAddr::from([0, 0, 0, 0]) {
                wait = false;
                break;
            } else {
                seed = addr;
            }
        }

        buf = unsafe { std::mem::zeroed() };

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

fn parse_raw_addr(arr: &[u8]) -> SocketAddr {
    let ip = IpAddr::from([arr[0], arr[1], arr[2], arr[3]]);
    let port = u16::from_be_bytes([arr[4], arr[5]]);
    SocketAddr::new(ip, port)
}
