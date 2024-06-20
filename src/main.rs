use newsletter::{configuration::get_configuration, startup::run};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = get_configuration().expect("Failed to get settings");

    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}
