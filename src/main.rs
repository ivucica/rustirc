use tokio::io::{AsyncBufRead, AsyncBufReadExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // adds new methods on TcpStream etc. // allows for conn.lines()

async fn broadcast(name: &String, line: String) -> Result<(), String> {
    println!("<{name}> {line}");
    return Ok(());
}

// TODO: implement broadcast

async fn chat_conn(mut conn: tokio::net::TcpStream) -> Result<(), String> {
    conn.write_all("Name? ".as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    let buffered_reader = tokio::io::BufReader::new(conn);
    let mut line_reader = buffered_reader.lines();

    let name = line_reader
        .next_line()
        .await
        .map_err(|e| e.to_string())?
        .unwrap(); // TODO: fix panic by using unwrap_or_default() or similar

    while let Some(line) = line_reader.next_line().await.map_err(|e| e.to_string())? {
        broadcast(&name, line).await?;
    }

    return Ok(());
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .map_err(|e| e.to_string())?;

    loop {
        let (conn, sockaddr) = listener.accept().await.map_err(|e| e.to_string())?;
        tokio::spawn(async { chat_conn(conn).await }); // in this line, ownership of conn passed into chat_conn() -- i.e. conn is "moved"
    }
    return Ok(());
}

//fn main() {
//let runtime = tokio::runtime::Runtime::new().unwrap();
//runtime.spawn(async {amain()});
//}
