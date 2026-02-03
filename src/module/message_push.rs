// use reqwest::Client;
use reqwest::blocking::Client;
use serde_json::json;
use std::error::Error;

pub fn telegram(
    token: &str,
    chat_id: &str,
    message: &str,
    proxy: &str,
) -> Result<(), Box<dyn Error>> {
    let body = json!({
        "chat_id":chat_id,
        "text":message,
    });

    let client: Client;

    if proxy != "" {
        client = Client::builder()
            .proxy(reqwest::Proxy::all(proxy)?)
            .build()?;
    } else {
        client = Client::new();
    }

    let resp = client
        .post(format!("https://api.telegram.org/bot{}/sendMessage", token).as_str())
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?;

    if !resp.status().is_success() {
        return Err(Box::<dyn Error>::from("Telegram bot returned an error"));
    }
    Ok(())
}

pub fn show_doc(token: &str, title: &str, content: &str) -> Result<(), Box<dyn Error>> {
    let body = json!({
        "title":title,
        "content":content,
    });

    let resp = Client::new()
        .post(format!("https://push.showdoc.com.cn/server/api/push/{}", token).as_str())
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?;

    if !resp.status().is_success() {
        return Err(Box::<dyn Error>::from("show doc returned an error"));
    }
    Ok(())
}
