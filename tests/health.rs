use newsletter::configuration::get_configuration;
use reqwest::Client;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let server = newsletter::startup::run(listener).expect("Failed to run the server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
#[tokio::test]
async fn health_works() {
    // Arrange
    let address = spawn_app();
    let client = Client::new();
    // Act
    let response = client
        .get(&format!("{}/health", &address))
        .send()
        .await
        .expect("Failed to execute the request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form() {
    // Arrange
    let address = spawn_app();
    let client = Client::new();
    let settings = get_configuration().expect("Failed to read configuration");
    let connection_string = settings.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    // Act
    let body = "name=sang%20khuu&email=rustdev%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute the request.");
    // Assert
    assert_eq!(200, response.status().as_u16());

    // let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
    //     .fetch_one(&mut connection)
    //     .await
    //     .expect("Failed to fetch saved subscription");

    // assert_eq!(saved.email, "rustdev@gmail.com");
    // assert_eq!(saved.name, "sang khuu");
}

#[tokio::test]
async fn subscribe_returns_a_400_for_missing_form() {
    // Arrange
    let address = spawn_app();
    let client = Client::new();
    let test_cases = vec![
        ("name=sang%20khuu", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute the request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            // Custom message for error
            "API dit not fail with 400 Bad request when the payload was {}",
            error_message
        )
    }
}
