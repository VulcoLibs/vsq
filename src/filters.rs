#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Filters {
    pub app_id: u32,
    pub no_password: bool,
}

impl Filters {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = format!("\\appid\\{}", self.app_id).into_bytes();

        if self.no_password {
            buf.extend(b"\\password\\0");
        }

        buf.push(0x00);
        buf
    }
}
