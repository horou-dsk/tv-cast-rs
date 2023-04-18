use actix_web::{
    get, http::header, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use hztp::{
    constant::SERVER_PORT, dlna_init, ip_online_check, protocol::DLNAHandler, routers,
    ssdp::ALLOW_IP,
};
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, process::Command, sync::Arc};

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
    if let Some(ip) = info.peer_addr() {
        ALLOW_IP
            .write()
            .unwrap()
            .push(ip.parse::<Ipv4Addr>().unwrap());
        HttpResponse::Ok().body(format!("{ip} 绑定成功！"))
    } else {
        HttpResponse::Ok().body(format!("没有获取到ip地址！"))
    }
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
    if let Ok(ipv4) = ip.parse::<Ipv4Addr>() {
        if !ALLOW_IP.read().unwrap().contains(&ipv4) {
            return HttpResponse::InternalServerError().body("permission denied");
        }
    } else {
        return HttpResponse::InternalServerError().body("permission denied");
    }
    println!("read description.xml from ip = {}", ip);
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(dlna.description().to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Server...");
    tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            if let Err(err) = ip_online_check().await {
                println!("检查ip地址是否在线失败！ error = {:?}", err);
            }
        }
    });
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
    .bind(("0.0.0.0", SERVER_PORT))?
    .run()
    .await
}
