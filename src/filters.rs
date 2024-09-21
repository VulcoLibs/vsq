#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Filters {
    pub app_id: u32,
    pub no_password: bool,
}

impl Filters {
    pub fn as_filter_string(&self) -> String {
        let mut buf = format!("\\appid\\{}", self.app_id);

        if self.no_password {
            buf.push_str("\\\\password\\0");
        }

        buf
    }
}
