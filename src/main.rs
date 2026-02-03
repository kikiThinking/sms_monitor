use crate::module::config::Config;
use crate::module::message_push::{show_doc, telegram};
use crate::module::monitor_sms::Monitor;
use crossbeam::channel;
use std::{env, fs, thread};

mod module;
fn main() {
    let fp = match env::current_exe() {
        Ok(exe_path) => {
            let dir = exe_path.parent().expect("当前路径没有父目录");
            format!("{}/config.yml", dir.display())
        }
        Err(e) => panic!("failed to get current executable: {}", e),
    };

    let f_result = fs::read_to_string(fp.clone()).expect(format!("{}", fp).as_str());

    let application_config: Config = serde_yml::from_str(&f_result).unwrap();

    // 1. 强制刷新输出（或使用 eprintln!）
    println!("Starting connect device..");

    let mut serial = module::ml307c::Controller::new(
        application_config.name,
        application_config.baud,
        application_config.timeout,
    );
    serial.connect().expect("Connect failed");

    // 2. 使用 tokio 的异步通道
    let (tx, rx) = channel::unbounded();

    thread::spawn(move || {
        println!("Monitor thread started (inside thread)");

        let mut monitor = Monitor::new(tx, serial);

        match monitor.init() {
            Ok(_) => println!("Monitor initialized successfully"),
            Err(e) => {
                eprintln!("Init device failed: {}", e);
                return;
            }
        }

        monitor.start(); // 这里应该是一个循环
    });

    for received in rx.iter() {
        if application_config.telegram.token != "" && application_config.telegram.chat_id != "" {
            if let Err(err) = telegram(
                application_config.telegram.token.as_str(),
                application_config.telegram.chat_id.as_str(),
                &received,
                application_config.telegram.proxy.as_str(),
            ) {
                eprintln!("telegram device returned an error: {}", err);
            }
        }

        if application_config.show_doc.token != "" {
            if let Err(err) = show_doc(
                application_config.show_doc.token.as_str(),
                "新的短信提醒!",
                &received,
            ) {
                eprintln!("Failed to send show doc: {}", err);
            }
        }

        println!("Sent: {}", received);
    }
}
