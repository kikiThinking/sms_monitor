use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Baud")]
    pub baud: u32,
    #[serde(rename = "Timeout", default = "timeout_default")]
    pub timeout: u64,
    #[serde(rename = "Telegram")]
    pub telegram: Telegram,
    #[serde(rename = "ShowDoc")]
    pub show_doc: ShowDoc,
}

#[derive(Debug, Deserialize)]
pub struct Telegram {
    #[serde(rename = "Token", default)]
    pub token: String,
    #[serde(rename = "ChatID", default)]
    pub chat_id: String,
    #[serde(rename = "Proxy", default)]
    pub proxy: String,
}

#[derive(Debug, Deserialize)]
pub struct ShowDoc {
    #[serde(rename = "Token", default)]
    pub token: String,
}

fn timeout_default() -> u64 {
    30
}
