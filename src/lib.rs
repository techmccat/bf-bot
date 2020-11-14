use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::{collections::HashMap, sync::Mutex};

pub struct Handler {
    user_lock: Mutex<HashMap<u64, Mutex<HashMap<u64, String>>>>,
}
impl Handler {
    pub fn new() -> Handler {
        Handler {
            user_lock: Mutex::new(HashMap::new()),
        }
    }
}
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let prog = if let Some(c) = self.user_lock.lock().unwrap().get(&msg.channel_id.0) {
            if let Some(m) = c.lock().unwrap().remove(&msg.author.id.0) {
                Some(m)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(text) = prog {
            if let Err(err) = msg
                .channel_id
                .say(
                    &ctx.http,
                    match bf_lib::run(&text[2..], Some(msg.content)) {
                        Ok(ok) => ok,
                        Err(err) => err,
                    },
                )
                .await
            {
                println!("Error sending message: {:?}", err);
            }
        } else {
            if msg.content.len() > 2 {
                if msg.content[..2] == *"< " {
                    if bf_lib::wants_input(&msg.content[..]) {
                        add_to_map(
                            &self.user_lock,
                            msg.channel_id.0,
                            msg.author.id.0,
                            msg.content,
                        );
                        if let Err(err) = msg
                            .channel_id
                            .say(
                                &ctx.http,
                                "Program requires input, next message from user will be read",
                            )
                            .await
                        {
                            println!("Error sending message: {:?}", err);
                        }
                    } else {
                        if let Err(err) = msg
                            .channel_id
                            .say(
                                &ctx.http,
                                match bf_lib::run(&msg.content[2..], None) {
                                    Ok(ok) => ok,
                                    Err(err) => err,
                                },
                            )
                            .await
                        {
                            println!("Error sending message: {:?}", err);
                        }
                    }
                }
            }
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn add_to_map(
    user_lock: &Mutex<HashMap<u64, Mutex<HashMap<u64, String>>>>,
    chid: u64,
    uid: u64,
    content: String,
) {
    let mut channels = user_lock.lock().unwrap();
    if let Some(mutex) = channels.get(&chid) {
        mutex.lock().unwrap().insert(uid, content);
    } else {
        channels.insert(
            chid,
            Mutex::new({
                let map: HashMap<u64, String> = [(uid, content)].iter().cloned().collect();
                map
            }),
        );
    }
}
