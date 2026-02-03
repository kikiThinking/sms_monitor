use regex::Regex;

pub struct PDU {
    pub batch_id: u8,
    pub total: u8,
    pub index: u8,
    pub is_long: bool,
    pub content: String,
}

impl PDU {
    pub fn analysis(data: &str) -> Option<Self> {
        let (mut is_long, mut batch_id, mut total, mut index, mut content): (
            bool,
            u8,
            u8,
            u8,
            String,
        ) = (false, 0, 0, 0, String::new());

        if data.starts_with("050003") && data.len() > 12 {
            is_long = true;
            batch_id = u8::from_str_radix(&data[6..8], 16).unwrap_or(1);
            total = u8::from_str_radix(&data[8..10], 16).unwrap_or(1);
            index = u8::from_str_radix(&data[10..12], 16).unwrap_or(1);
            content = Self::decode_ucs2(data[12..].to_string().as_str());
        } else if data.starts_with("060804") && data.len() > 16 {
            is_long = true;
            batch_id = u8::from_str_radix(&data[6..10], 16).unwrap_or(1);
            total = u8::from_str_radix(&data[10..12], 16).unwrap_or(1);
            index = u8::from_str_radix(&data[12..14], 16).unwrap_or(1);
            content = Self::decode_ucs2(data[16..].to_string().as_str());
        } else {
            content = Self::decode_ucs2(data);
        }

        Some(Self {
            batch_id,
            total,
            index,
            is_long,
            content: content.clone(),
        })
    }

    fn decode_ucs2(raw: &str) -> String {
        // 1. 如果不是纯十六进制，直接返回
        if !raw.chars().all(|c| c.is_ascii_hexdigit()) || raw.len() % 2 != 0 {
            return raw.to_string();
        }

        // 2. 尝试解码
        let data = hex::decode(raw).unwrap_or_default();
        if data.is_empty() {
            return raw.to_string();
        }

        let mut u16s = Vec::new();
        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                u16s.push(u16::from_be_bytes([chunk[0], chunk[1]]));
            }
        }

        let decoded = String::from_utf16(&u16s).unwrap_or_else(|_| raw.to_string());

        // 3. 如果解码结果全是 ASCII，就返回原始字符串
        if decoded.chars().all(|c| c.is_ascii()) {
            raw.to_string()
        } else {
            decoded
        }
    }
}
