pub mod models;
pub mod routes;

use poem::{
    EndpointExt, Route, Server, get, handler, listener::TcpListener, middleware::Tracing, web::Path,
};
use routes::blockchain::generate_keypair;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("/keypair", poem::post(generate_keypair));

    println!("Server running at http://localhost:3000");
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
