struct ChatSession {
    nick: String,
    sender: tokio::sync::broadcast::Sender<ChatMessage>,
    receiver: tokio::sync::broadcast::Receiver<ChatMessage>,
}

#[derive(Clone, Debug)] // Needed so that we can have Sender<ChatMessage> -- must be clonable. We choose not to implement our own, and just to derive. for printing, we derive Debug.
pub struct ChatMessage {
    nick: String,
    msg: String,
}

pub struct ChatServerState {
    // TODO: consider whether we need to add Vec for connections for uses other than sending
    sender: tokio::sync::broadcast::Sender<ChatMessage>,
}

impl ChatServerState {
    pub fn new() -> Self {
        let (s, _) = tokio::sync::broadcast::channel(8);

        return ChatServerState { sender: s };
    }

    pub async fn run(&mut self, addr: &str) -> Result<(), String> {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| e.to_string())?;

        loop {
            let (conn, sockaddr) = listener.accept().await.map_err(|e| e.to_string())?;
            let s = self.sender.clone();
            tokio::spawn(async {
                let mut session = ChatSession::new(s);

                // n.b. in this line, ownership of conn passed into chat_conn() -- i.e. conn is "moved"
                // (same with 's' above)
                session.handler(conn).await
            });
        }

        Ok(())
    }
}

impl ChatSession {
    fn new(sender: tokio::sync::broadcast::Sender<ChatMessage>) -> Self {
        ChatSession {
            nick: "anonymous".to_string(),
            receiver: sender.subscribe(),
            sender: sender,
        }
    }

    async fn handler(&mut self, mut conn: tokio::net::TcpStream) -> Result<(), String> {
        // adding 'use' for traits inside a function makes them in scope only within a function.
        use tokio::io::{AsyncBufRead, AsyncBufReadExt};
        use tokio::io::{AsyncReadExt, AsyncWriteExt}; // adds new methods on TcpStream etc. // allows for conn.lines()

        let (r, mut w) = conn.split();

        w.write_all("Name? ".as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        let buffered_reader = tokio::io::BufReader::new(r);
        let mut line_reader = buffered_reader.lines();

        let name = line_reader
            .next_line()
            .await
            .map_err(|e| e.to_string())?
            .unwrap(); // TODO: fix panic by using unwrap_or_default() or similar

        self.nick = name;

        loop {
            tokio::select! {
                cm_result = self.receiver.recv() => {
                    println!("received something into {:?}, {:?}", self.nick, cm_result);
                    if let Ok(cm) = &cm_result {
                        // right now it's unreasonable to filter own messages.
                        // we do want to see locally what we sent (echo back to the current user).
                        // even in interactive scenarios this would be nice as a confirmation of an action.
                        //
                        // but, if we DID want to avoid displaying our own message,
                        // something like self.nick != cm.nick would be used.
                        //
                        // of course, nick is also a bad identifier for a session...

                        w.write_all(format!("<{}> {}\n", cm.nick, cm.msg).as_bytes()).await;
                        // TODO: handle error from write_all
                    }
                    if let Err(err) = &cm_result {
                        println!("{}: error receiving a message: {}", self.nick, err); // TODO: use a better per-conn identifier
                    }
                }

                msg_result = line_reader.next_line() => {
                    match msg_result {
                        Ok(maybe_msg) => {
                            if let Some(msg) = maybe_msg {
                                println!("sending: <{}> {}", self.nick, msg);
                                self.sender.send(ChatMessage { nick: self.nick.clone(), msg: msg });
                                // TODO: handle error. using .expect("error while sending msg") would crash the program

                                // TODO: handle slash-commands such as /names (to see who's in the room).
                                // TODO: handle multiple channels and joining+leaving channels.
                            } else {
                                // TODO: handle that we got none from the socket (EOF? closed conn?)
                                println!("{}: got none from line reader (EOF?)", self.nick);
                                break // stop handling the interrupted connection
                            }

                        }
                        Err(err) => {
                            println!("{}: error sending a message: {}", self.nick, err); // TODO: use a better per-conn identifier
                        }
                    }

                }
            }
        }

        // alternative if we don't have to both grab line AND handle incoming msg
        //while let Some(line) = line_reader.next_line().await.map_err(|e| e.to_string())? {
        // broadcast(&name, line).await?;
        //}

        return Ok(());
    }
}
