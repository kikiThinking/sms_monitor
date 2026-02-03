use regex::Regex;
use std::sync::OnceLock;

pub struct PDU {
    pub batch_id: u8,
    pub total: u8,
    pub index: u8,
    pub is_long: bool,
    pub content: String,
}

impl PDU {
    pub fn analysis(data: &str) -> Option<Self> {
        let (mut is_long, mut batch_id, mut total, mut index): (bool, u8, u8, u8) =
            (false, 0, 0, 0);
        let content: String;

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
        // 1. 过滤干扰字符 (类似 Go 的 hexRegex)
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"[^0-9a-fA-F]").unwrap());
        let clean_hex = re.replace_all(raw, "");

        // 2. 尝试解码十六进制
        let data = match hex::decode(clean_hex.as_ref()) {
            Ok(d) => d,
            Err(_) => return raw.to_string(),
        };

        // 3. 智能判断：长度校验
        // UCS2 是双字节编码，Hex 长度必须是 4 的倍数 (每个字 4 位 Hex)
        // 如果你传入 "1" (len=1) 或 "4F60" (len=4)，这里会做区分
        if data.is_empty() || data.len() % 2 != 0 {
            return raw.to_string();
        }

        // 4. 执行 UTF-16BE 解码
        let u16s: Vec<u16> = data
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();

        match String::from_utf16(&u16s) {
            Ok(decoded) => {
                // 5. 二次校验：智能回退
                // 如果解码后的字符串包含大量的不可打印字符，或者你认为它不该被转换
                // 判断标准：如果转换后全是正常的 ASCII 可打印字符（如 "123"），
                // 且原始长度很短，往往原样返回更安全。
                let has_non_printable = decoded.chars().any(|c| {
                    // 允许换行、回车、制表符，其他控制字符视为“乱码”风险
                    c.is_control() && c != '\n' && c != '\r' && c != '\t'
                });

                // 核心逻辑：如果包含中文(非ASCII)且没有乱码控制符，则认为是正确的解码
                let has_cjk = decoded.chars().any(|c| c > '\x7F');

                if has_cjk && !has_non_printable {
                    decoded
                } else if !has_non_printable && decoded.len() * 4 == raw.len() {
                    // 如果全是可打印 ASCII (如 "abc")，且原串确实是对应的 Hex
                    decoded
                } else {
                    raw.to_string()
                }
            }
            Err(_) => raw.to_string(),
        }
    }
}
