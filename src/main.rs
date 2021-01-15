use bf_bot::{Config, Handler};
use serenity::prelude::*;
use std::{env, fs};

#[tokio::main]
async fn main() {
    let config = read_config();
    let handler = Handler::new(config.clone());
    let mut client = Client::builder(&config.token)
        .event_handler(handler)
        .await
        .expect("Can't create client");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn read_config() -> Config {
    let config_path = {
        let mut p = env::current_exe().unwrap();
        p.pop();
        p.push("config.toml");
        p
    };
    let string = fs::read_to_string(config_path).unwrap_or_else(|_| {
        fs::read_to_string({
            let mut p = env::current_dir().unwrap();
            p.push("config.toml");
            p
        })
        .expect("No config file provided")
    });
    toml::from_str(&string).unwrap()
}
