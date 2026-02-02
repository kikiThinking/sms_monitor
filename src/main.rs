use std::thread;
use std::time::Duration;
use crate::module::monitor_sms::Monitor;
use crossbeam::channel;

mod module;

fn main() {
    // 1. 强制刷新输出（或使用 eprintln!）
    println!("Starting connect device..");

    let mut serial = module::ml307c::Controller::new("/dev/ttyUSB2".to_string(), 115200);
    serial.connect().expect("Connect failed");

    // 2. 使用 tokio 的异步通道
    let (tx, rx) = channel::unbounded();

    // 3. 将阻塞的 monitor 放到 spawn_blocking 运行
    // 这样它不会阻塞主线程的事件循环
    // 保存线程句柄
    let monitor_handle = thread::spawn(move || {
        println!("Monitor thread started (inside thread)");

        let mut monitor = Monitor::new(tx, serial);

        match monitor.init() {
            Ok(_) => println!("Monitor initialized successfully"),
            Err(e) => {
                eprintln!("Init device failed: {}", e);
                // 发送错误信号到主线程
                return;
            }
        }

        println!("Starting monitor loop...");
        monitor.start(); // 这里应该是一个循环
    });

    println!("Main loop is now listening for SMS...");

    // 给线程一点时间启动
    thread::sleep(Duration::from_millis(100));

    // 修改主循环，添加超时处理
    loop {
        // 检查线程是否还在运行
        if monitor_handle.is_finished() {
            println!("Monitor thread has finished!");
            break;
        }

        // 非阻塞接收消息
        match rx.try_recv() {
            Ok(msg) => {
                println!("Response: {}", msg);

                // 如果收到特定消息可以退出
                if msg.contains("EXIT") || msg.contains("ERROR") {
                    println!("Received exit/error signal, stopping...");
                    break;
                }
            }
            Err(crossbeam::channel::TryRecvError::Empty) => {
                // 没有消息，短暂休眠后继续
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(crossbeam::channel::TryRecvError::Disconnected) => {
                println!("Channel disconnected, monitor thread may have died");
                break;
            }
        }
    }

    // 等待线程结束
    match monitor_handle.join() {
        Ok(_) => println!("Monitor thread joined successfully"),
        Err(e) => eprintln!("Monitor thread panicked: {:?}", e),
    }

    println!("Main program exiting");

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
