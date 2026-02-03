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
        let (mut is_long, mut batch_id, mut total, mut index): (
            bool,
            u8,
            u8,
            u8,
        ) = (false, 0, 0, 0);
        let content:String;


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
        // 1. 基本合法性校验：必须是偶数长度且全是 16 进制字符
        if raw.len() % 2 != 0 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
            return raw.to_string();
        }

        // 2. 尝试将 16 进制转为字节数组
        let data = match hex::decode(raw) {
            Ok(d) => d,
            Err(_) => return raw.to_string(),
        };

        // 3. 将字节转为 UTF-16 (UCS-2)
        let mut u16s = Vec::new();
        for chunk in data.chunks_exact(2) {
            u16s.push(u16::from_be_bytes([chunk[0], chunk[1]]));
        }

        match String::from_utf16(&u16s) {
            Ok(decoded) => {
                // 核心改进：判断是否包含“有意义”的非 ASCII 字符（如中文）
                // 或者判断解码后的字符是否包含不可打印字符。
                // 如果你希望 4F604EEC 转换，而 31(即'1') 不转换：
                if decoded.chars().any(|c| !c.is_ascii()) {
                    decoded
                } else {
                    // 如果全是 ASCII（比如解码出来是 "1"），通常说明原字符串可能就是想表达 "31" 这个文本
                    // 这里你可以根据业务决定：是返回解码后的 "1" 还是原始文本 "31"
                    raw.to_string()
                }
            }
            Err(_) => raw.to_string(),
        }
    }
}
