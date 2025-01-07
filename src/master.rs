use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};
use crate::Filters;

mod dns;

const RESPONSE_PACKET_BUFFER_SIZE: usize = 3000;  // allows to collect 500 Addresses
const ADDRESS_SIZE: usize = 6;  // 4 bytes + 1 ushort

pub struct MasterQuery {
    dns: dns::DNS,
}

impl MasterQuery {
    /// It's required to wait 10 seconds between packets in order to not get blocked.
    const RATE_LIMIT: Duration = Duration::from_secs(10);
    const UNSPECIFIED_ADDR: SocketAddr = SocketAddr::V4(
        SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)
    );

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
        #[cfg(feature = "signals")]
        mut packet_signal: tokio::sync::mpsc::Receiver<()>,
    ) -> std::io::Result<VSQTask> {
        let master_addr = SocketAddr::new(
            self
                .dns
                .lookup_ip(master_server.as_ref())
                .await
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::Other))?,
            master_port
        );

        let mut sock = UdpSocket::bind(Self::UNSPECIFIED_ADDR).await?;
        sock.connect(master_addr).await?;

        let handle = tokio::spawn(async move {
            let mut wait = true;
            let mut buf = [0u8; RESPONSE_PACKET_BUFFER_SIZE];
            let mut seed = Self::UNSPECIFIED_ADDR;

            let mut interval = interval(Self::RATE_LIMIT);
            interval.tick().await;

            while wait {
                let mut packet = vec![0x31, 0xFF];
                packet.extend(seed.to_string().into_bytes());
                packet.push(0x00);
                packet.extend(filters.as_bytes());
                packet.push(0x00);

                let read_buf = {
                    let len = Self::get_packet_response(
                        &sock,
                        &packet,
                        &mut buf,
                    ).await?;

                    if len == 0 {
                        break;
                    }

                    &buf[..len]
                };

                for raw_addr in read_buf.chunks(ADDRESS_SIZE) {
                    let addr = Self::parse_raw_addr(raw_addr);

                    if addr.ip() != IpAddr::from([u8::MAX, u8::MAX, u8::MAX, u8::MAX]) {
                        if callback.send(addr).await.is_err() {
                            wait = false;
                            break;
                        }

                        if addr.ip() == IpAddr::from([0, 0, 0, 0]) {
                            wait = false;
                            break;
                        }

                        seed = addr;

                        #[cfg(feature = "signals")]
                        packet_signal.recv().await;
                    }
                }

                interval.tick().await;
            }

            drop(callback);

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

    async fn get_packet_response(sock: &UdpSocket, packet: &Vec<u8>, buffer: &mut [u8]) -> std::io::Result<usize> {
        sock.send(&packet).await?;

        tokio::time::timeout(
            Duration::from_secs(4),
            sock.recv(buffer)
        ).await.map_err(|err| {
            error!("Failed to receive the packet from Master: {}", err);
            std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                err,
            )
        })?
    }
}

pub struct VSQTask {
    pub handle: JoinHandle<std::io::Result<()>>,
}
