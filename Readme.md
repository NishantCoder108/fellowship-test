# Fellowship Test

Write http server to interact with solana program

## Setup

```bash
cargo install 
```

## Run

```bash
cargo run    
```


## Routes
```rust
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/keypair", poem::post(generate_keypair))
        .at("/token/create", poem::post(create_token))
        .at("/token/mint", poem::post(mint_token))
        .at("/message/sign", poem::post(sign_message))
        .at("/message/verify", poem::post(verify_message))
        .at("/send/sol", poem::post(routes::blockchain::send_sol))
        .at("/send/token", poem::post(routes::blockchain::send_token));

    println!("Server running at http://localhost:3000");
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
```