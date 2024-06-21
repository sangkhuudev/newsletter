use newsletter::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);

    init_subscriber(subscriber);

    let settings = get_configuration().expect("Failed to get settings");
    let db_pool = PgPool::connect(settings.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool)?.await?;

    Ok(())
}
