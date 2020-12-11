use bf_bot::{Config, Handler} ;
use serenity::prelude::*;
use std::{env, fs, time::Duration};
use toml::Value;

#[tokio::main]
async fn main() {
    let toml = read_config().parse::<Value>().unwrap();
    let token = match toml["token"].as_str() {
        Some(t) => t.to_owned(),
        None => env::var("DISCORD_TOKEN").expect("No token provided"),
    };
    let config = Config {
        prefix: toml["prefix"].as_str().expect("No prefix provided").to_string(),
        timeout: if let Some(s) = toml["timeout"].as_str() { Some(Duration::from_secs(s.parse().unwrap())) } else { None },
        tmppath: if let Some(s) = toml.get("tmppath") { let mut p = env::current_dir().unwrap();
            p.push(s.to_string()); Some(p) } else { None }
    };
    let handler = Handler::new(config);
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
