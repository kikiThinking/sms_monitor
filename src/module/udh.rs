use regex::Regex;

pub struct UDH {
    pub batch_id: u8,
    pub total: u8,
    pub index: u8,
    pub is_long: bool,
    pub content: String,
}

impl UDH {
    pub fn analysis(data: &str) -> Option<Self> {
        let (mut is_long, mut batch_id, mut total, mut index, mut content): (
            bool,
            u8,
            u8,
            u8,
            String,
        ) = (false, 0, 0, 0, String::new());

        if data.starts_with("050003") {
            is_long = true;
            batch_id = u8::from_str_radix(&data[6..8], 16).unwrap_or(1);
            total = u8::from_str_radix(&data[8..10], 16).unwrap_or(1);
            index = u8::from_str_radix(&data[10..12], 16).unwrap_or(1);
            content = Self::decode_ucs2(data[12..].to_string().as_str());
        } else if data.starts_with("060804") {
            is_long = true;
            batch_id = u8::from_str_radix(&data[6..10], 16).unwrap_or(1);
            total = u8::from_str_radix(&data[10..12], 16).unwrap_or(1);
            index = u8::from_str_radix(&data[12..14], 16).unwrap_or(1);
            content = Self::decode_ucs2(data[16..].to_string().as_str());
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
        // 1. 过滤掉非十六进制字符
        let hex_regex = Regex::new(r"[^0-9A-Fa-f]").unwrap();
        let clean_hex = hex_regex.replace_all(raw, "");

        // 2. 尝试解码十六进制
        let data = match hex::decode(&*clean_hex) {
            Ok(d) => d,
            Err(_) => return raw.to_string(),
        };

        // 3. 校验长度
        if data.is_empty() || data.len() % 2 != 0 {
            return raw.to_string();
        }

        // 4. 转换为 UTF-16BE
        let mut u16s = Vec::with_capacity(data.len() / 2);
        for i in (0..data.len()).step_by(2) {
            let code = (data[i] as u16) << 8 | (data[i + 1] as u16);
            u16s.push(code);
        }

        let decoded = String::from_utf16(&u16s).unwrap_or_else(|_| raw.to_string());

        // 5. 保底逻辑：如果解码结果为空，回退原始字符串
        if decoded.trim().is_empty() {
            raw.to_string()
        } else {
            decoded
        }
    }
}
