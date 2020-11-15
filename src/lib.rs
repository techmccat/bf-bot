use serenity::{
    async_trait,
    model::{channel::Message, error::Error, gateway::Ready},
    prelude::*,
};
use std::{collections::HashMap, sync::Mutex};

pub struct Handler {
    user_lock: Mutex<HashMap<u64, HashMap<u64, MapData>>>,
}

#[derive(Clone)]
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
        if let Some(user) = channels.get_mut(&chid) {
            user.insert(
                uid,
                MapData {
                    text: content,
                    botmsg,
                },
            );
        } else {
            channels.insert(chid, {
                let mut map = HashMap::new();
                map.insert(
                    uid,
                    MapData {
                        text: content,
                        botmsg,
                    },
                );
                map
            });
        }
    }

    fn get_user_lock(&self, chid: u64, uid: u64) -> Option<MapData> {
        if let Some(c) = self.user_lock.lock().unwrap().get_mut(&chid) {
            if let Some(m) = c.remove(&uid) {
                Some(m)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mapdata = self.get_user_lock(msg.channel_id.0, msg.author.id.0);
        let prog = if let Some(d) = mapdata.clone() {
            Some(d.text)
        } else if msg.content.len() > 2 {
            if msg.content[..2] == *"< " {
                Some(String::from(&msg.content[2..]))
            } else {
                None
            }
        } else if msg.content == "<" && msg.attachments.len() > 0 {
            match msg.attachments[0].download().await {
                Ok(chars) => Some(String::from_utf8_lossy(&chars).into_owned()),
                Err(err) => {
                    println!("Error downloading attachment: {:?}", err);
                    None
                }
            }
        } else {
            None
        };
        let input = if let Some(d) = mapdata {
            if let Err(err) = d.botmsg.delete(&ctx.http).await {
                println!("Error deleting message: {:?}", err);
            }
            Some(msg.content)
        } else {
            None
        };
        let output = if let Some(prog) = prog {
            if bf_lib::wants_input(&prog) && input == None {
                let botmsg = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        format!(
                            "Program requires input, next message from {} will be read",
                            msg.author.name
                        ),
                    )
                    .await
                    .expect("Error sending message");
                self.add_to_map(msg.channel_id.0, msg.author.id.0, prog, botmsg);
                None
            } else {
                let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
                let o = match bf_lib::run(&prog, input) {
                    Ok(ok) => ok,
                    Err(err) => err,
                };
                typing.stop();
                Some(o)
            }
        } else {
            None
        };
        if let Some(o) = output {
            if let Err(e) = msg.channel_id.say(&ctx.http, &o).await {
                if let serenity::Error::Model(Error::MessageTooLong(_)) = e {
                    if let Err(e) = msg
                        .channel_id
                        .send_files(&ctx.http, vec![(o.as_bytes(), "output.txt")], |m| {
                            m.content("Program output was too long, sending as file")
                        })
                        .await
                    {
                        println!("Error sending message: {}", e)
                    }
                } else {
                    println!("Error sending message: {}", e)
                }
            };
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
