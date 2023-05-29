use crate::{
    actions::jni_action::AVTransportAction, android, constant::SERVER_PORT, dlna_init,
    ip_online_check, protocol::DLNAHandler, routers, ssdp::ALLOW_IP,
};
use actix_web::{
    get, http::header, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, path::Path, sync::Arc};
use tokio::sync::Mutex;

#[get("/")]
async fn hello() -> impl Responder {
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
            log::info!("{device_ip} 绑定！");
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

pub fn run_main(
    name: String,
    uuid_path: &Path,
    av_action: AVTransportAction,
) -> std::io::Result<()> {
    actix_web::rt::System::new().block_on(async move {
        android::remove_file(&uuid_path.join("out.log")).await?;
        android::remove_file(&uuid_path.join("hztp")).await?;
        android::remove_file(&uuid_path.join("hztp_uuid.txt")).await?;
        if let Err(err) = android::uninstall_package("com.waxrain.airplaydmr").await {
            eprintln!("卸载失败 {err:?}");
        }
        let mut args = std::env::args();
        args.next();
        let name = args.next().unwrap_or(name);
        tokio::spawn(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                if let Err(err) = ip_online_check().await {
                    log::error!("检查ip地址是否在线失败！ error = {:?}", err);
                }
            }
        });
        // let mut air_play_bonjour = AirPlayBonjour::new(name.clone());
        // air_play_bonjour.start();
        let dlna = match dlna_init(name, uuid_path) {
            Ok(dlna) => dlna,
            Err(err) => {
                log::error!("dlna init error {err:?}");
                return Err(err);
            }
        };
        let dlna = Arc::new(dlna);
        // let connect_uri = if cfg!(windows) {
        //     "http://192.169.1.19:10021".to_string()
        // } else {
        //     format!("http://{}", ANDROID_ADDR)
        // };
        // let rpc_client = loop {
        //     if let Ok(client) = AvTransportClient::connect(connect_uri.clone()).await {
        //         break client;
        //     }
        //     tokio::time::sleep(Duration::from_secs(2)).await;
        // };
        let rpc_client = Arc::new(Mutex::new(av_action));

        log::info!("Starting Server...");

        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
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
    })
}
