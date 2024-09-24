use super::*;

const PACKET_SIZE: usize = 64_000 + 32_000;

pub struct ResPacket {
    pub header: u8,
    pub payload: Vec<u8>
}

impl ResPacket {
    pub const HEADER_CHALLENGE: u8 = 0x41;
    pub const HEADER_A2S_RULES: u8 = 0x56;

    pub async fn rcv(sock: &UdpSocket) -> std::io::Result<Self> {
        let mut buf = vec![0u8; PACKET_SIZE];
        let len = sock.recv(&mut buf).await?;

        let read_buf = &buf[4..len];

        Ok(Self {
            header: read_buf[0],
            payload: {
                let payload = &read_buf[1..];
                Vec::from(payload)
            }
        })
    }
}
