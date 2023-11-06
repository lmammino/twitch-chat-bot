use crate::parser::{parse_msg, Msg};
use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use parser::{PrivMsg, User};
use websocket_lite::{AsyncClient, AsyncNetworkStream, Message, Opcode, Result};

mod parser;

type PrivMessageListener = Box<dyn FnMut(PrivMsg)>;
type JoinOrPartListener = Box<dyn FnMut(User)>;

struct Client {
    channel: String,
    nick: String,
    twitch_token: String,
    socket: Option<AsyncClient<Box<dyn AsyncNetworkStream + Send + Sync + Unpin>>>,
    on_priv_msg: Vec<PrivMessageListener>,
    on_join: Vec<JoinOrPartListener>,
    on_part: Vec<JoinOrPartListener>,
}

impl Client {
    fn new(channel: String, nick: String, twitch_token: String) -> Self {
        Self {
            channel,
            nick,
            twitch_token,
            socket: None,
            on_priv_msg: Vec::new(),
            on_join: Vec::new(),
            on_part: Vec::new(),
        }
    }

    async fn connect(&mut self) -> Result<()> {
        let builder = websocket_lite::ClientBuilder::new("wss://irc-ws.chat.twitch.tv:443")?;
        let mut ws_stream = builder.async_connect().await?;

        // Send capabilities requests
        let cap_payload = "CAP REQ :twitch.tv/membership twitch.tv/tags";
        ws_stream.send(Message::text(cap_payload)).await?;

        // Send auth details
        let auth_payload = format!("PASS oauth:{}", self.twitch_token);
        ws_stream.send(Message::text(auth_payload)).await?;
        // let msg = ws_stream.next().await;
        // println!("AUTH RESPONSE: {:?}", msg);

        // Send nickname
        let nick_payload = format!("NICK {}", self.nick);
        ws_stream.send(Message::text(nick_payload)).await?;
        // let msg = ws_stream.next().await;
        // println!("NICK RESPONSE: {:?}", msg);

        // Join channel
        let join_payload = format!("JOIN #{}", self.channel);
        ws_stream.send(Message::text(join_payload)).await?;

        self.socket = Some(ws_stream);

        Ok(())
    }

    fn add_priv_msg_listener(&mut self, listener: PrivMessageListener) {
        self.on_priv_msg.push(listener);
    }

    fn add_join_listener(&mut self, listener: JoinOrPartListener) {
        self.on_join.push(listener);
    }

    fn add_part_listener(&mut self, listener: JoinOrPartListener) {
        self.on_part.push(listener);
    }

    async fn send_msg(&mut self, msg: &str) -> Result<()> {
        let ws_stream = self.socket.as_mut().unwrap();
        ws_stream
            .send(Message::text(&format!(
                "PRIVMSG #{} :{}!",
                self.channel, msg
            )))
            .await?;
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        let ws_stream = self.socket.as_mut().unwrap();
        while let Some(msg) = ws_stream.next().await {
            if let Ok(m) = msg {
                match m.opcode() {
                    Opcode::Text => {
                        let text = m.as_text().unwrap();
                        for line in text.lines() {
                            let msg = parse_msg(line);

                            println!("MESSAGE RECEIVED: {:?}", msg);
                            match msg {
                                Msg::Ping { server_name } => {
                                    let pong_payload = format!("PONG :{}", server_name);
                                    ws_stream.send(Message::text(pong_payload)).await?;
                                }
                                Msg::Join(user) => {
                                    for listener in &mut self.on_join {
                                        listener(user.clone());
                                    }

                                    // ws_stream
                                    //     .send(Message::text(format!(
                                    //         "PRIVMSG #{} :Hi @{}!",
                                    //         user.channel, user.nick
                                    //     )))
                                    //     .await?;
                                }
                                _ => {
                                    // TODO: implement additional behaviours if needed
                                }
                            }
                        }
                    }
                    Opcode::Ping => ws_stream.send(Message::pong(m.into_data())).await?,
                    Opcode::Close => {
                        println!("Received close message");
                        let _ = ws_stream.send(Message::close(None)).await;
                        break;
                    }
                    Opcode::Pong | Opcode::Binary => {}
                }
            } else {
                println!("Error reading message: {:?}", msg);
                let _ = ws_stream.send(Message::close(None)).await;
                break;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let twitch_token = std::env::var("TWITCH_TOKEN").expect("Missing TWITCH_TOKEN");
    let mut client = Client::new("loige".to_string(), "loigebot".to_string(), twitch_token);
    client.connect().await?;

    client.add_priv_msg_listener(Box::new(|m: PrivMsg<'_>| {
        println!("PRIV MSG: {:?}", m);
        // TODO: the code below does not work because we end up borrrowing mutably more than once! We probably need a Mutex!
        // async {
        //     client
        //         .send_msg(format!("Hello {}! How are you?", m.user.nick).as_str())
        //         .await;
        // };
    }));

    client.run().await?;

    println!("Exiting the main loop");

    Ok(())
}
