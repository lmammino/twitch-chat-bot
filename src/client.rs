use std::future::Future;
use crate::parser::{parse_msg, Msg, PrivMsg, User};
use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use tokio::sync::Mutex;
use websocket_lite::{AsyncClient, AsyncNetworkStream, Message, Opcode, Result};

type PrivMessageListener = Box<dyn Fn(PrivMsg, &ConnectClient) -> dyn Future<Output=()>>;
type JoinOrPartListener = Box<dyn Fn(User, &ConnectClient) -> dyn Future<Output=()>>;

pub struct Client {
    channel: String,
    nick: String,
    on_priv_msg: Vec<PrivMessageListener>,
    on_join: Vec<JoinOrPartListener>,
    on_part: Vec<JoinOrPartListener>,
}

pub struct ConnectClient {
    client: Client,
    socket: Mutex<AsyncClient<Box<dyn AsyncNetworkStream + Send + Sync + Unpin>>>,
}

impl Client {
    pub fn new(channel: String, nick: String) -> Self {
        Self {
            channel,
            nick,
            on_priv_msg: Vec::new(),
            on_join: Vec::new(),
            on_part: Vec::new(),
        }
    }

    pub async fn connect(self, twitch_token: String) -> Result<ConnectClient> {
        let builder = websocket_lite::ClientBuilder::new("wss://irc-ws.chat.twitch.tv:443")?;
        let mut ws_stream = builder.async_connect().await?;

        // Send capabilities requests
        let cap_payload = "CAP REQ :twitch.tv/membership twitch.tv/tags";
        ws_stream.send(Message::text(cap_payload)).await?;

        // Send auth details
        let auth_payload = format!("PASS oauth:{}", twitch_token);
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

        Ok(ConnectClient {
            client: self,
            socket: Mutex::new(ws_stream),
        })
    }

    pub fn add_priv_msg_listener(&mut self, listener: PrivMessageListener) {
        self.on_priv_msg.push(listener);
    }

    pub fn add_join_listener(&mut self, listener: JoinOrPartListener) {
        self.on_join.push(listener);
    }

    pub fn add_part_listener(&mut self, listener: JoinOrPartListener) {
        self.on_part.push(listener);
    }
}

impl ConnectClient {
    pub async fn send_msg(&self, msg: &str) -> Result<()> {
        self.socket
            .lock()
            .await
            .send(Message::text(&format!(
                "PRIVMSG #{} :{}!",
                self.client.channel, msg
            )))
            .await?;
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        while let Some(msg) = self.socket.lock().await.next().await {
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
                                    self.socket
                                        .lock()
                                        .await
                                        .send(Message::text(pong_payload))
                                        .await?;
                                }
                                Msg::PrivMsg(msg) => {
                                    for listener in &self.client.on_priv_msg {
                                        listener(msg.clone(), self).await;
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
                    Opcode::Ping => {
                        self.socket
                            .lock()
                            .await
                            .send(Message::pong(m.into_data()))
                            .await?
                    }
                    Opcode::Close => {
                        println!("Received close message");
                        let _ = self.socket.lock().await.send(Message::close(None)).await;
                        break;
                    }
                    Opcode::Pong | Opcode::Binary => {}
                }
            } else {
                println!("Error reading message: {:?}", msg);
                let _ = self.socket.lock().await.send(Message::close(None)).await;
                break;
            }
        }

        Ok(())
    }
}
