use actix_web::{get, http::header, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use hztp::{
    actions::rpc_action::rpc::av_transport_client::AvTransportClient,
    constant::{ANDROID_ADDR, SERVER_PORT},
    dlna_init, ip_online_check,
    protocol::DLNAHandler,
    routers,
    ssdp::ALLOW_IP,
};
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[get("/")]
async fn hello() -> impl Responder {
    // Command::new("am")
    //     .args([
    //         "start",
    //         "-n",
    //         "com.droidlogic.mboxlauncher/com.droidlogic.mboxlauncher.Launcher",
    //     ])
    //     .status()
    //     .expect("错误...");
    HttpResponse::Found()
        .append_header((header::LOCATION, "https://niconico-ni.com"))
        .finish()
    // HttpResponse::Ok().body("Hello world!")
}

async fn bind_ip(req: HttpRequest) -> impl Responder {
    let info = req.connection_info();
    if let Some(ip) = info.peer_addr() {
        let device_ip = ip.parse::<Ipv4Addr>().unwrap();
        let mut allow_ip = ALLOW_IP.write().unwrap();
        if !allow_ip.contains(&device_ip) {
            println!("{device_ip} 绑定！");
            allow_ip.push(device_ip);
        }
        HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .body(format!("{ip} 绑定成功！"))
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

// #[post("/sh")]
// async fn sh(data: web::Json<Sh>) -> impl Responder {
//     let output = Command::new("sh")
//         .arg("-c")
//         .arg(&data.sh)
//         .output()
//         .expect("错误...");
//     let stdout = output.stdout;
//     let output = String::from_utf8_lossy(&stdout);
//     HttpResponse::Ok().body(output.to_string())
// }

#[get("/description.xml")]
async fn description(dlna: web::Data<Arc<DLNAHandler>>, req: HttpRequest) -> impl Responder {
    let info = req.connection_info();
    match info.peer_addr() {
        Some(ip)
            if ALLOW_IP
                .read()
                .unwrap()
                .contains(&ip.parse::<Ipv4Addr>().unwrap()) =>
        {
            HttpResponse::Ok()
                .append_header((header::CONTENT_TYPE, "text/xml"))
                .body(dlna.description().to_string())
        }
        _ => HttpResponse::Unauthorized().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // if !cfg!(windows) {
    //     tokio::time::sleep(Duration::from_secs(5)).await;
    // }
    let mut args = std::env::args();
    args.next();
    let name = args.next().expect("缺少投屏名称参数！");
    tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Err(err) = ip_online_check().await {
                println!("检查ip地址是否在线失败！ error = {:?}", err);
            }
        }
    });
    let dlna = Arc::new(dlna_init(name).unwrap());
    let connect_uri = if cfg!(windows) {
        "http://192.169.1.19:10021".to_string()
    } else {
        format!("http://{}", ANDROID_ADDR)
    };
    let rpc_client = loop {
        if let Ok(client) = AvTransportClient::connect(connect_uri.clone()).await {
            break client;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    };
    let rpc_client = Arc::new(Mutex::new(rpc_client));

    println!("Starting Server...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(dlna.clone()))
            .app_data(web::Data::new(rpc_client.clone()))
            .service(hello)
            .route("/ip", web::get().to(bind_ip))
            // .service(sh)
            .service(description)
            .configure(routers::dlna::config)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", SERVER_PORT))?
    .run()
    .await
}
