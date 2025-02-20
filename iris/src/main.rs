use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use tokio::{spawn, try_join};

mod commands;
mod entities;
mod handler;
mod server;

const ADDRESS: &str = "0.0.0.0:8080";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting server at http://{}", ADDRESS);

    let (connection_server, server_tx) = server::ConnectionServer::new();
    let connection_server = spawn(connection_server.run());

    let http_server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(middleware::NormalizePath::trim())
            .app_data(web::Data::new(server_tx.clone()))
            .service(handler::register_user)
            .service(handler::register_device)
    })
    .workers(4)
    .bind(ADDRESS)?
    .run();

    try_join!(http_server, async move { connection_server.await.unwrap() })?;

    Ok(())
}
