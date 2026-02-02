use crate::module::ml307c::Controller;
use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

pub struct Monitor {
    write: Sender<String>,
    control: Controller,
    sms_parts: HashMap<u8, String>, // 缓存分片
    total_parts: Option<u8>,        // 总分片数
    is_init: bool,
}

impl Monitor {
    pub fn new(write: Sender<String>, control: Controller) -> Monitor {
        Monitor {
            write,
            control,
            sms_parts: Default::default(),
            total_parts: None,
            is_init: false,
        }
    }

    // 初始化模块 设定静默模式 开启短信上报
    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        for command in vec![
            "AT",                // 测试连接
            "ATE0",              // 关闭回显 (关键：实现静默)
            "AT+CEREG=0",        // 关闭网络注册状态上报
            "AT+CSCS=\"UCS2\"",  // 强制模块输出十六进制，统一解析逻辑
            "AT+CMGF=1",         // 文本模式
            "AT+CNMI=2,2,0,0,0", // 新短信直接上报，不存储
        ]
        .iter()
        {
            self.control.write(command)?;
        }
        self.is_init = true;
        Ok(())
    }

    pub fn start(&mut self) {
        if !self.is_init {
            panic!("Attempted to start a module that has not been initialized yet");
        }

        loop {
            thread::sleep(Duration::from_millis(500));
            if let Ok(resp) = self.control.read() {
                println!("{}", resp);
                if resp.contains("+CMT:") {
                    if let Some(body_hex) = resp.split('\n').nth(1) {
                        // UDH 前 12 个 hex
                        if body_hex.len() > 12 {
                            let udh = &body_hex[..12];
                            let content = &body_hex[12..];

                            let total = u8::from_str_radix(&udh[8..10], 16).unwrap_or(1);
                            let seq = u8::from_str_radix(&udh[10..12], 16).unwrap_or(1);

                            self.total_parts = Some(total);
                            self.sms_parts.insert(seq, content.to_string());

                            // 如果收齐所有分片
                            if let Some(t) = self.total_parts {
                                if self.sms_parts.len() == t as usize {
                                    // 拼接
                                    let mut full_hex = String::new();
                                    let mut keys: Vec<u8> =
                                        self.sms_parts.keys().cloned().collect();
                                    keys.sort();
                                    for k in keys {
                                        full_hex.push_str(&self.sms_parts[&k]);
                                    }
                                    self.sms_parts.clear();
                                    self.total_parts = None;

                                    // 解码 UCS2
                                    let decoded = decode_ucs2(&full_hex);
                                    if let Err(e) = self.write.blocking_send(decoded) {
                                        eprintln!("Send to channel failed: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    // 还没收齐
                }
            } else {
                continue;
            }
        }
    }
}

/// 将 UCS2 十六进制字符串解码为 Unicode 文本
fn decode_ucs2(hex_str: &str) -> String {
    let mut result = String::new();
    for i in (0..hex_str.len()).step_by(4) {
        if i + 4 <= hex_str.len() {
            if let Ok(code) = u16::from_str_radix(&hex_str[i..i + 4], 16) {
                if let Some(ch) = std::char::from_u32(code as u32) {
                    result.push(ch);
                }
            }
        }
    }
    result
}
