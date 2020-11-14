use std::env;
use serenity::{
    prelude::*,
};

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("No token provided");
    let handler = bf_bot::Handler::new();
    let mut client = Client::builder(&token)
        .event_handler(handler)
        .await
        .expect("Can't create client");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
