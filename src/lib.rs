use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::{collections::HashMap, sync::Mutex};

pub struct Handler {
    user_lock: Mutex<HashMap<u64, Mutex<HashMap<u64, MapData>>>>,
}

struct MapData {
    text: String,
    botmsg: Message,
}

impl Handler {
    pub fn new() -> Handler {
        Handler {
            user_lock: Mutex::new(HashMap::new()),
        }
    }
    fn add_to_map(&self, chid: u64, uid: u64, content: String, botmsg: Message) {
        let mut channels = self.user_lock.lock().unwrap();
        if let Some(mutex) = channels.get(&chid) {
            mutex.lock().unwrap().insert(
                uid,
                MapData {
                    text: content,
                    botmsg,
                },
            );
        } else {
            channels.insert(
                chid,
                Mutex::new({
                    let mut map = HashMap::new();
                    map.insert(
                        uid,
                        MapData {
                            text: content,
                            botmsg,
                        },
                    );
                    map
                }),
            );
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mapdata = if let Some(c) = self.user_lock.lock().unwrap().get(&msg.channel_id.0) {
            if let Some(m) = c.lock().unwrap().remove(&msg.author.id.0) {
                Some(m)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(d) = mapdata {
            if let Err(err) = msg
                .channel_id
                .say(
                    &ctx.http,
                    match bf_lib::run(&d.text[2..], Some(msg.content)) {
                        Ok(ok) => ok,
                        Err(err) => err,
                    },
                )
                .await
            {
                println!("Error sending message: {:?}", err);
            }
            if let Err(err) = d
                .botmsg
                    .delete(&ctx.http)
                    .await {
                println!("Error deleting message: {:?}", err);
            }
        } else {
            if msg.content.len() > 2 {
                if msg.content[..2] == *"< " {
                    if bf_lib::wants_input(&msg.content[..]) {
                        let botmsg = match msg
                            .channel_id
                            .say(
                                &ctx.http,
                                format!(
                                    "Program requires input, next message from {} will be read",
                                    msg.author.name
                                ),
                            )
                            .await
                        {
                            Ok(msg) => Some(msg),
                            Err(err) => {
                                println!("Error sending message: {:?}", err);
                                None
                            }
                        };
                        if let Some(bm) = botmsg {
                            self.add_to_map(msg.channel_id.0, msg.author.id.0, msg.content, bm);
                        }
                    } else {
                        let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
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
                        typing.stop();
                    }
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
