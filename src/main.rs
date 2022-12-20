use std::net::TcpListener;

use sqlx::{PgConnection, Connection};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind 8000 port");

    let connection = PgConnection::connect(
        &configuration.database.connection_string()
    ).await.
    expect("Failed to connect to Postgres.");

    run(listener, connection)?.await
}
