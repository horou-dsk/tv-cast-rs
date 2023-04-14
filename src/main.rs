use actix_web::{
    get, http::header, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use hztp::{dlna_init, protocol::DLNAHandler, routers};
use serde::{Deserialize, Serialize};
use std::{process::Command, sync::Arc};

#[get("/")]
async fn hello() -> impl Responder {
    Command::new("am")
        .args([
            "start",
            "-n",
            "com.droidlogic.mboxlauncher/com.droidlogic.mboxlauncher.Launcher",
        ])
        .status()
        .expect("错误...");
    HttpResponse::Found()
        .append_header((header::LOCATION, "https://niconico-ni.com"))
        .finish()
    // HttpResponse::Ok().body("Hello world!")
}

#[get("/ip")]
async fn echo(req: HttpRequest) -> impl Responder {
    let info = req.connection_info();
    let ip = info.peer_addr().unwrap_or("没获取到IP");
    HttpResponse::Ok().body(ip.to_string())
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[derive(Debug, Serialize, Deserialize)]
struct Sh {
    sh: String,
}

#[post("/sh")]
async fn sh(data: web::Json<Sh>) -> impl Responder {
    let output = Command::new("sh")
        .arg("-c")
        .arg(&data.sh)
        .output()
        .expect("错误...");
    let stdout = output.stdout;
    let output = String::from_utf8_lossy(&stdout);
    HttpResponse::Ok().body(output.to_string())
}

#[get("/description.xml")]
async fn description(dlna: web::Data<Arc<DLNAHandler>>, req: HttpRequest) -> impl Responder {
    let info = req.connection_info();
    let ip = info.peer_addr().unwrap_or("没获取到IP");
    println!("read description.xml from ip = {}", ip);
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(dlna.description().to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Server...");
    let dlna = Arc::new(dlna_init().unwrap());
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(dlna.clone()))
            .service(hello)
            .service(echo)
            .service(sh)
            .service(description)
            .configure(routers::dlna::config)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
