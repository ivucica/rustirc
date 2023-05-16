struct ChatSession {
    nick : String,
    conn : tokio::net::TcpStream
}

#[derive(Clone)]  // Needed so that we can have Sender<ChatMessage> -- must be clonable. We choose not to implement our own, and just to derive.
pub struct ChatMessage {
    nick : String,
    msg : String
}

pub struct ChatServerState {
    conns : Vec<ChatSession>,
    sender : tokio::sync::broadcast::Sender<ChatMessage>
}

impl ChatServerState {
    pub fn new() -> Self {
        let (s, _) = tokio::sync::broadcast::channel(8);

        return ChatServerState{
            conns: Vec::new(),
            sender: s
        }
    }
}
