use crate::module::message_push::{show_doc, telegram};
use crate::module::monitor_sms::Monitor;

mod module;

#[tokio::main]
async fn main() {
    // 1. 强制刷新输出（或使用 eprintln!）
    println!("Starting connect device..");

    let mut serial = module::ml307c::Controller::new("/dev/ttyUSB2".to_string(), 115200);
    serial.connect().expect("Connect failed");

    // 2. 使用 tokio 的异步通道
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    
    // 3. 将阻塞的 monitor 放到 spawn_blocking 运行
    // 这样它不会阻塞主线程的事件循环
    tokio::task::spawn_blocking(move || {
        let mut monitor = Monitor::new(tx, serial);
        if let Err(e) = monitor.init() {
            eprintln!("Init device failed: {}", e);
            return;
        }
        println!("Monitor thread started.");
        monitor.start(); // 这里死循环也没关系，它在独立线程池运行
    });

    println!("Main loop is now listening for SMS...");

    // 4. 异步接收并推送
    while let Some(received) = rx.recv().await {
        println!("New SMS: {}", received);

        // 使用 tokio::spawn 并发推送，防止网络超时卡住接收逻辑
        let msg = received.clone();
        tokio::spawn(async move {
            let _ = telegram(
                "8001087713:AAHniNA7Df5vWG-lgyuQtFYg8wMOOaoMeSY",
                "-5110051584",
                &msg,
            )
            .await;
            let _ = show_doc(
                "b5eb898252101c380929b7aff8114b9f1865901162",
                "新的短信提醒!",
                &msg,
            )
            .await;
        });
    }
}
