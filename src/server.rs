use std::mem::size_of;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

const CHALLENGE_SIZE: usize = size_of::<i32>();
const CHALLENGE_COUNTER_MAX: usize = 10;

mod req_packet;
mod res_packet;

use req_packet::*;
use res_packet::*;

#[repr(transparent)]
pub struct ServerQuery {
    sock: UdpSocket,
}

impl ServerQuery {
    pub async fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        sock.connect(addr).await?;

        Ok(Self {
            sock,
        })
    }

    async fn send_packet(&self, mut packet: ReqPacket) -> std::io::Result<ResPacket> {
        packet.send(&self.sock).await?;

        let mut challenge_counter = 0;
        let mut res = ResPacket::rcv(&self.sock).await?;

        while res.header == ResPacket::HEADER_CHALLENGE && challenge_counter < CHALLENGE_COUNTER_MAX {
            if res.payload.len() != CHALLENGE_SIZE {
                error!("Received packet with a challenge header, but invalid payload!");
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
            }

            let challenge = i32::from_be_bytes(unsafe {
                let mut buf: [u8; CHALLENGE_SIZE] = std::mem::zeroed();
                buf.copy_from_slice(&*res.payload);
                buf
            });

            #[cfg(debug_assertions)]
            if let Ok(addr) = self.sock.peer_addr() {
                debug!("[{}] Received challenge packet [0x{:X}] [{:b}]!", addr, res.header, challenge);
            } else {
                debug!("Received challenge packet [0x{:X}] [{:b}]!", res.header, challenge);
            }

            if packet.header == ResPacket::HEADER_A2S_RULES {
                packet.payload = None;
            }

            packet.challenge = Some(challenge);
            packet.send(&self.sock).await?;

            res = ResPacket::rcv(&self.sock).await?;
            challenge_counter += 1;
        }

        Ok(res)
    }

    pub async fn a2s_info(&self) -> std::io::Result<ResPacket> {
        self.send_packet(
            ReqPacket::from_type(PacketType::INFO)
        ).await
    }

    pub async fn a2s_rules(&self) -> std::io::Result<ResPacket> {
        self.send_packet(
            ReqPacket::from_type(PacketType::RULES)
        ).await
    }
}
