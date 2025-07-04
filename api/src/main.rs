pub mod models;
pub mod routes;

use poem::{
    EndpointExt, Route, Server, get, handler,
    listener::TcpListener,
    middleware::Tracing,
    web::{Json, Path},
};
use routes::blockchain::generate_keypair;

use crate::routes::blockchain::{create_token, mint_token, sign_message, verify_message};

#[tokio::main]
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
