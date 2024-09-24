use std::mem::size_of;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

const CHALLENGE_SIZE: usize = size_of::<i32>();

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

    pub async fn send_packet(&self, mut packet: ReqPacket) -> std::io::Result<ResPacket> {
        packet.send(&self.sock).await?;

        let mut res = ResPacket::rcv(&self.sock).await?;

        if res.header == ResPacket::HEADER_CHALLENGE {
            if res.payload.len() != CHALLENGE_SIZE {
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
            }

            let challenge = i32::from_be_bytes(unsafe {
                let mut buf: [u8; CHALLENGE_SIZE] = std::mem::zeroed();
                buf.copy_from_slice(&*res.payload);
                buf
            });

            packet.challenge = Some(challenge);
            packet.send(&self.sock).await?;

            res = ResPacket::rcv(&self.sock).await?;
        }

        Ok(res)
    }

    pub async fn a2s_info(&self) -> std::io::Result<ResPacket> {
        self.send_packet(
            ReqPacket::from_type(PacketType::INFO)
        ).await
    }
}
