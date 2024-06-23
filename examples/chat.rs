use std::{fmt::Display, net::SocketAddr, sync::Arc};

use anyhow::Result;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt as _};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, warn};

const MAX_CHANNEL_LENGTH: usize = 32;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[derive(Debug)]
enum Message {
    UserJoin(String),
    UserLeave(String),
    Chat {
        sender: String, //发送的人，是username
        content: String,
    },
}

impl Message {
    fn user_joind(username: &str) -> Self {
        Self::UserJoin(format!("{} joined the chat", username))
    }
    fn user_left(username: &str) -> Self {
        Self::UserLeave(format!("{} left the chat", username))
    }
    fn chat(sender: String, content: String) -> Self {
        Self::Chat { sender, content }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserJoin(msg) => write!(f, "[System: {}]", msg),
            Self::UserLeave(msg) => write!(f, "[System: {}]", msg),
            Self::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("Failed to send message to {}: {:?}", peer.key(), e);
                // if send failed, remove the peer
                self.peers.remove(peer.key());
            }
        }
    }

    async fn add_peer(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (sender, mut receiver) = mpsc::channel(MAX_CHANNEL_LENGTH);
        let (mut stream_sender, stream_receiver) = stream.split();
        self.peers.insert(addr, sender);
        // 当这个client上线后，开启一个mpsc，把sender放到peers里面，其他client的消息就可以通过这个sender发送出去
        // 自己保留一个receiver，用来接收其他client的消息(这里需要开启一个task来处理receiver的消息)
        // read message from other peers
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                // tokio只有stream trait， 这里需要cargo add futures
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {:?}", addr, e);
                    break;
                }
            }
        });
        Peer {
            username,
            stream: stream_receiver,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // let layer = Layer::new().with_filter(LevelFilter::INFO);
    // tracing_subscriber::registry().with(layer).init();
    console_subscriber::init();

    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on: {}", addr);
    let state = Arc::new(State::default());
    loop {
        let state = state.clone();
        let (stream, addr) = listener.accept().await?;
        info!("Accept connection from: {}", addr);
        tokio::spawn(async move {
            if let Err(e) = handle_client(state, addr, stream).await {
                warn!("Failed to handle client{}, error: {}", addr, e);
            }
        });
    }
}

/// 当一个cliet连接上来后，需要输入用户名，然后加入到peers里面，然后广播一条消息，告诉其他client有人加入了
/// add peer的时候先会创建一个mpsc， producer放到state中，其他peers共用这个producer给这个client发送消息
/// 将stream的sender split出来，开启一个task来处理别的peer发到mpsc consumer的消息
/// 然后这个client就可以发送消息了，其他client就可以接收到这个消息
async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Please enter your username:").await?;
    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Err(anyhow::anyhow!("Client disconnected")),
    };
    let mut peer = state.add_peer(addr, username, stream).await;

    let message = Arc::new(Message::user_joind(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line from {}: {:?}", addr, e);
                break;
            }
        };
        let message = Arc::new(Message::chat(peer.username.clone(), line));
        info!("Broadcast Message: {}", message);
        state.broadcast(addr, message).await;
    }

    // 当走到这里的时候，说明这个client断开了连接
    let leave_message = Arc::new(Message::user_left(&peer.username));
    info!("{}", leave_message);
    state.broadcast(addr, leave_message).await;
    state.peers.remove(&addr);
    Ok(())
}
