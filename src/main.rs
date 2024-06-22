use newsletter::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);

    init_subscriber(subscriber);

    let settings = get_configuration().expect("Failed to get settings");

    let db_pool = PgPoolOptions::new().connect_lazy_with(settings.database.with_db());

    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool)?.await?;

    Ok(())
}
