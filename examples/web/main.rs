#![feature(lazy_cell)]

use std::{cell::LazyCell, sync::Arc};

use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ld = LazyCell::new(|| {
        println!("初始化...");
        Arc::new(String::from("123123123"))
    });
    let ld = ld.clone();
    HttpServer::new(move || App::new().app_data(ld.clone()).service(hello))
        .bind(("127.0.0.1", 22339))?
        .run()
        .await
}
