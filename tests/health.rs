use newsletter::configuration::{get_configuration, DatabaseSettings};
use reqwest::Client;
use sqlx::{Connection, Executor, PgPool, PgConnection};
use uuid::Uuid;
use std::net::TcpListener;


//------------------------------------------------------

pub struct TestApp {
    address: String,
    db_pool: PgPool
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let mut settings = get_configuration().expect("Failed to get settings");
    settings.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&settings.database).await;
    let server = newsletter::startup::run(listener, db_pool.clone()).expect("Failed to run the server");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);
    TestApp {address, db_pool}
}
// 1.Initialize a connection without a db 
// 2. Create db in that connnection 
// 3.Embedded that connection into a db pool
async fn configure_database(db_setting: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(
        &db_setting.connection_string_without_db()
    )
        .await
        .expect("Failed to establish new connection");

    connection.execute(format!(r#"CREATE DATABASE "{}";"#, db_setting.database_name).as_str())
        .await
        .expect("Failed to create database");
    // Migrate database
    let connection_pool = PgPool::connect(
        &db_setting.connection_string()
    )
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");
    connection_pool

}
//----------------------------------------------------
#[tokio::test]
async fn health_works() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();
    // Act
    let response = client
        .get(&format!("{}/health", &app.address))
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
    let app = spawn_app().await;
    let client = Client::new();
   // Act
    let body = "name=sang%20khuu&email=rustdev%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute the request.");
    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "rustdev@gmail.com");
    assert_eq!(saved.name, "sang khuu");
}

#[tokio::test]
async fn subscribe_returns_a_400_for_missing_form() {
    // Arrange
    let app = spawn_app().await;
    let client = Client::new();
    let test_cases = vec![
        ("name=sang%20khuu", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
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
