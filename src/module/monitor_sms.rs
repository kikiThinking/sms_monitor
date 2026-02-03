use crate::module::ml307c::Controller;
use crate::module::pdu::PDU;
use crossbeam::channel::Sender;
use std::collections::HashMap;
use std::error::Error;
use std::string::String;
use std::thread;
use std::time::Duration;

pub struct Monitor {
    write: Sender<String>,
    control: Controller,
    is_init: bool,
}

impl Monitor {
    pub fn new(write: Sender<String>, control: Controller) -> Monitor {
        Monitor {
            write,
            control,
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

        let mut response: HashMap<u8, Vec<PDU>> = HashMap::new();

        loop {
            thread::sleep(Duration::from_millis(500));
            if let Ok(resp) = self.control.read() {
                if resp.contains("+CMT:") {
                    if let Some(body_hex) = resp.split('\n').nth(1) {
                        if let Some(pdu_decode) = PDU::analysis(body_hex) {
                            if pdu_decode.is_long {
                                let (batch_id, total): (u8, u8) =
                                    (pdu_decode.batch_id, pdu_decode.total);

                                response
                                    .entry(pdu_decode.batch_id)
                                    .or_insert_with(Vec::new)
                                    .push(pdu_decode);

                                if let Some(pdu_list) = response.get_mut(&batch_id) {
                                    if pdu_list.len() == usize::from(total) {
                                        // 收集完毕 开始组装
                                        let mut decode_result = String::new();
                                        pdu_list.sort_by_key(|udh| udh.index);

                                        for udh in pdu_list {
                                            decode_result.push_str(udh.content.as_str());
                                        }

                                        self.write.send(decode_result).unwrap();
                                        // 删除切片
                                        response.remove(&batch_id);
                                    }
                                }
                            } else {
                                self.write.send(pdu_decode.content).unwrap();
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
