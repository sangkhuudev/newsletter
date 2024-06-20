use std::net::TcpListener;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FormData {
    pub name: String, 
    pub email: String,
}


pub async fn subscribe(_form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}
