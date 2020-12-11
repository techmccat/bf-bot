use serenity::prelude::*;
use std::{env, fs};
use toml::Value;

#[tokio::main]
async fn main() {
    let config = read_config().parse::<Value>().unwrap();
    let token = match config["token"].as_str() {
        Some(t) => t.to_owned(),
        None => env::var("DISCORD_TOKEN").expect("No token provided"),
    };
    let prefix = config["prefix"].as_str().expect("No prefix provided");
    let handler = bf_bot::Handler::new(prefix.to_owned());
    let mut client = Client::builder(&token)
        .event_handler(handler)
        .await
        .expect("Can't create client");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn read_config() -> String {
    let config_path = {
        let mut p = env::current_exe().unwrap();
        p.pop();
        p.push("config.toml");
        p
    };
    fs::read_to_string(config_path).unwrap_or_else(|_| {
        fs::read_to_string({
            let mut p = env::current_dir().unwrap();
            p.push("config.toml");
            p
        })
        .unwrap()
    })
}
