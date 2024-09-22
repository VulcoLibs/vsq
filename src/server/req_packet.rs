use tokio::net::UdpSocket;

static PACKET_PREFIX: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

pub(super) enum PacketType {
    INFO,
}

pub(super) struct ReqPacket {
    pub header: u8,
    pub payload: String,
    pub challenge: Option<i32>,
}

impl ReqPacket {
    pub fn new(header: u8, payload: impl ToString, challenge: Option<i32>) -> Self {
        Self {
            header,
            payload: payload.to_string(),
            challenge,
        }
    }

    pub fn from_type(packet_type: PacketType) -> Self {
        match packet_type {
            PacketType::INFO => {
                Self::new(0x54, "Source Engine Query", None)
            }
        }
    }

    pub async fn send(&self, sock: &UdpSocket) -> std::io::Result<()> {
        sock.send(
            &*self.to_raw()
        ).await?;

        Ok(())
    }

    fn to_raw(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.extend(PACKET_PREFIX);

        buffer.push(self.header);
        buffer.extend(self.payload.as_bytes());
        buffer.push(0x00);

        if let Some(challenge) = self.challenge {
            buffer.extend(challenge.to_be_bytes());
        }

        buffer
    }
}
