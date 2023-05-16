mod state;

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut server = state::ChatServerState::new();

    server.run("127.0.0.1:8080").await?;
    return Ok(());
}

//fn main() {
//let runtime = tokio::runtime::Runtime::new().unwrap();
//runtime.spawn(async {amain()});
//}
