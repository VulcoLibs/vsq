use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;

mod filters;
mod dns;

pub use filters::*;

const RESPONSE_PACKET_BUFFER_SIZE: usize = 3000;  // allows to collect 500 Addresses
const ADDRESS_SIZE: usize = 6;  // 4 bytes + 1 ushort

pub struct VSQ {
    dns: dns::DNS,
}

impl VSQ {
    pub async fn new() -> Self {
        let dns = dns::DNS::new().await;

        Self {
            dns,
        }
    }

    pub async fn start(
        &self,
        master_server: impl AsRef<str>,
        master_port: u16,
        filters: Filters,
        callback: tokio::sync::mpsc::Sender<SocketAddr>,
    ) -> std::io::Result<VSQTask> {
        let master_addr = SocketAddr::new(
            self
                .dns
                .lookup_ip(master_server.as_ref())
                .await
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::Other))?,
            master_port
        );

        let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        sock.connect(master_addr).await?;

        let handle = tokio::spawn(async move {
            let mut wait = true;
            let mut buf = [0u8; RESPONSE_PACKET_BUFFER_SIZE];
            let mut seed = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));

            while wait {
                let mut packet = vec![0x31, 0xFF];
                packet.extend(seed.to_string().into_bytes());
                packet.push(0x00);
                packet.extend(filters.as_filter_string().into_bytes());
                packet.push(0x00);

                sock.send(&packet).await?;

                let read_buf = {
                    let len = sock.recv(&mut buf).await?;

                    if len == 0 {
                        break;
                    }

                    &buf[..len]
                };

                for raw_addr in read_buf.chunks(ADDRESS_SIZE) {
                    let addr = Self::parse_raw_addr(raw_addr);

                    if addr.ip() == IpAddr::from([0, 0, 0, 0]) {
                        wait = false;
                        break;
                    } else if addr.ip() != IpAddr::from([u8::MAX, u8::MAX, u8::MAX, u8::MAX]) {
                        if callback.send(addr).await.is_err() {
                            wait = false;
                            break;
                        }
                        seed = addr;
                    }
                }
            }

            Ok(())
        });

        Ok(VSQTask {
            handle,
        })
    }

    fn parse_raw_addr(arr: &[u8]) -> SocketAddr {
        let ip = IpAddr::from([arr[0], arr[1], arr[2], arr[3]]);
        let port = u16::from_be_bytes([arr[4], arr[5]]);
        SocketAddr::new(ip, port)
    }
}

pub struct VSQTask {
    pub handle: JoinHandle<std::io::Result<()>>,
}
