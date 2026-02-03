use crate::module::monitor_sms::Monitor;
use crossbeam::channel;
use std::thread;
use std::time::Duration;

mod module;

fn main() {
    // 1. 强制刷新输出（或使用 eprintln!）
    println!("Starting connect device..");

    let mut serial = module::ml307c::Controller::new("/dev/ttyUSB2".to_string(), 115200);
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
        println!("Monitor received: {}", received);
    }

    println!("Process end")

    // // 4. 异步接收并推送
    // while let Some(received) = rx.recv().await {
    //     println!("New SMS: {}", received);
    //
    //     // 使用 tokio::spawn 并发推送，防止网络超时卡住接收逻辑
    //     let msg = received.clone();
    //     tokio::spawn(async move {
    //         let _ = telegram(
    //             "8001087713:AAHniNA7Df5vWG-lgyuQtFYg8wMOOaoMeSY",
    //             "-5110051584",
    //             &msg,
    //         )
    //             .await;
    //         let _ = show_doc(
    //             "b5eb898252101c380929b7aff8114b9f1865901162",
    //             "新的短信提醒!",
    //             &msg,
    //         )
    //             .await;
    //     });
    // }
}
