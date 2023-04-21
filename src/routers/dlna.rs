// use std::process::Command;

use std::{
    net::{IpAddr, Ipv4Addr},
    sync::LazyLock,
};

use actix_web::{
    get,
    http::{header, Method},
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use tokio::sync::RwLock;

use crate::{
    actions::avtransport::{
        android::{AVTransportURI, EachAction, PositionInfo, SeekTarget, TransportState},
        AVTransportAction, AVTransportResponse, GetPositionInfoResponse, GetTransportInfoResponse,
    },
    net::tcp_client,
};

// const MAX_SIZE: usize = 262_144;

#[get("AVTransport.xml")]
async fn avtransport_xml() -> impl Responder {
    println!("read avtransport_xml");
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(include_str!("../xml/AVTransport.xml"))
}

#[get("ConnectionManager.xml")]
async fn connection_manager_xml() -> impl Responder {
    println!("read connection_manager_xml");
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(include_str!("../xml/ConnectionManager.xml"))
}

const _ACTION_RESPONSE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
    <s:Body>
        {body}
    </s:Body>
</s:Envelope>"#;

struct SingleAction {
    timestamp: chrono::NaiveDateTime,
    host: IpAddr,
    running: bool,
    current_uri: String,
    current_uri_meta_data: String,
}

static SIGNLE_ACTION: LazyLock<RwLock<SingleAction>> = LazyLock::new(|| {
    RwLock::new(SingleAction {
        timestamp: chrono::Local::now().naive_local(),
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        running: false,
        current_uri: "".to_string(),
        current_uri_meta_data: "".to_string(),
    })
});

#[post("/action")]
async fn avtransport_action(request: HttpRequest, bytes: web::Bytes) -> impl Responder {
    // println!("{:#?}", request.headers());
    let host = request
        .peer_addr()
        .map(|addr| addr.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let result = String::from_utf8_lossy(&bytes);
    println!("{host:?} avtransport_action = {result}\n\n");
    // let host = host.cloned().unwrap_or(HeaderValue::from_str("").unwrap());
    match AVTransportAction::from_xml_text(&result) {
        Ok(AVTransportAction::GetTransportInfo(_)) => {
            let resp = match tcp_client::send::<_, EachAction<TransportState>>(
                EachAction::only_action("GetTransportInfo"),
            )
            .await
            {
                Ok(rep) => {
                    let state = rep.data.unwrap().current_transport_state;
                    if state == "STOPPED" {
                        SIGNLE_ACTION.write().await.running = false;
                    }
                    let sa = SIGNLE_ACTION.read().await;
                    GetTransportInfoResponse {
                        current_speed: "1",
                        current_transport_state: if sa.running && sa.host == host {
                            state
                        } else {
                            "STOPPED".to_string()
                        },
                        current_transport_status: "OK",
                        ..Default::default()
                    }
                }
                Err(_) => {
                    let sa = SIGNLE_ACTION.read().await;
                    GetTransportInfoResponse {
                        current_speed: "1",
                        current_transport_state: if sa.running && sa.host == host {
                            "PLAYING".to_string()
                        } else {
                            "STOPPED".to_string()
                        },
                        current_transport_status: "OK",
                        ..Default::default()
                    }
                }
            };
            AVTransportResponse::ok(resp)
        }
        Ok(AVTransportAction::SetAVTransportURI(av_uri)) => {
            let mut sa = SIGNLE_ACTION.write().await;
            let now = chrono::Local::now().naive_local();
            if sa.running && sa.host != host && (now - sa.timestamp) < chrono::Duration::seconds(5)
            {
                return AVTransportResponse::err(401, "Invalid Action");
            }
            sa.host = host;
            sa.timestamp = now;
            sa.current_uri = av_uri.uri.clone();
            sa.current_uri_meta_data = av_uri.uri_meta_data.clone();
            drop(sa);
            println!("\nmeta xml = {}\n", av_uri.uri_meta_data);
            tcp_client::send::<_, EachAction>(EachAction::new(
                "SetAVTransportURI",
                AVTransportURI {
                    uri: av_uri.uri,
                    uri_meta: av_uri.uri_meta_data,
                },
            ))
            .await
            .ok();
            SIGNLE_ACTION.write().await.running = true;
            // Command::new("am")
            //     .args([
            //         "start",
            //         "-n",
            //         "com.ycsoft.smartbox/com.ycsoft.smartbox.ui.activity.TPActivity",
            //         "-e",
            //         "CurrentURI",
            //         &av_uri.uri,
            //     ])
            //     .status()
            //     .expect("错误...");
            AVTransportResponse::default_ok("SetAVTransportURI")
        }
        Ok(AVTransportAction::Play(_)) => {
            let _: Option<EachAction> =
                tcp_client::send(EachAction::only_action("Play")).await.ok();
            AVTransportResponse::default_ok("Play")
        }
        Ok(AVTransportAction::Stop(_)) => {
            tcp_client::send::<_, EachAction>(EachAction::only_action("Stop"))
                .await
                .ok();
            SIGNLE_ACTION.write().await.running = false;
            AVTransportResponse::default_ok("Stop")
        }
        Ok(AVTransportAction::GetPositionInfo) => {
            if let Ok(resp) = tcp_client::send::<_, EachAction<PositionInfo>>(
                EachAction::only_action("GetPositionInfo"),
            )
            .await
            {
                let data = resp.data.unwrap();
                let sa = SIGNLE_ACTION.read().await;
                AVTransportResponse::ok(GetPositionInfoResponse {
                    track: 1,
                    track_duration: &data.track_duration,
                    track_uri: Some(&sa.current_uri),
                    track_meta_data: Some(&sa.current_uri_meta_data),
                    abs_time: &data.rel_time,
                    rel_time: &data.rel_time,
                    abs_count: i32::MAX,
                    rel_count: i32::MAX,
                    ..Default::default()
                })
            } else {
                AVTransportResponse::default_ok("GetPositionInfo")
            }
        }
        Ok(AVTransportAction::Seek(seek)) => {
            tcp_client::send::<_, EachAction>(EachAction::new(
                "Seek",
                SeekTarget {
                    target: seek.target,
                },
            ))
            .await
            .ok();
            AVTransportResponse::default_ok("Seek")
        }
        Ok(AVTransportAction::Pause) => {
            tcp_client::send::<_, EachAction>(EachAction::only_action("Pause"))
                .await
                .ok();
            AVTransportResponse::default_ok("Pause")
        }
        // Ok(AVTransportResponse::GetMediaInfo) => {
        //     if let Ok(resp) = tcp_client::send::<_, EachAction<PositionInfo>>(
        //         EachAction::only_action("GetPositionInfo"),
        //     )
        //     .await
        //     {
        //         let data = resp.data.unwrap();
        //         let sa = SIGNLE_ACTION.read().await;
        //         AVTransportResponse::ok(GetMediaInfoResponse {
        //             current_uri: &sa.current_uri,
        //             current_uri_meta_data: &sa.current_uri_meta_data,
        //             nr_tracks:
        //         })
        //     } else {
        //         AVTransportResponse::default_ok("GetMediaInfo")
        //     }
        // }
        _ => AVTransportResponse::err(401, "Invalid Action"),
    }
}

async fn avtransport_event(request: HttpRequest) -> impl Responder {
    println!(
        "avtransport_event = {:?}\n\n",
        request.headers().get("CALLBACK")
    );
    HttpResponse::ExpectationFailed()
        .append_header(("Server", "OS/Version UPnP/1.1 product/version"))
        .append_header(("SID", "uuid:subscibe-UUID"))
        .finish()
}

#[post("/action")]
async fn connection_manager_action(bytes: web::Bytes) -> impl Responder {
    let result = String::from_utf8_lossy(&bytes);
    println!("connection_manager_action = \n{}", result);
    HttpResponse::InternalServerError()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(format!(
            include_str!("../actions/xml/invalid_action.xml"),
            code = 401,
            err_msg = "Invalid Action"
        ))
}

#[get("/RenderingControl.xml")]
async fn rendering_control_xml() -> impl Responder {
    println!("read rendering_control_xml");
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(include_str!("../xml/RenderingControl.xml"))
}

#[post("/action")]
async fn rendering_control_action(request: HttpRequest, bytes: web::Bytes) -> impl Responder {
    let result = String::from_utf8_lossy(&bytes);
    println!(
        "\nRenderingControl SOAPACTION = {:?}",
        request.headers().get("SOAPACTION")
    );
    println!("\nrendering_control_action = \n{}\n", result);
    HttpResponse::InternalServerError()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(format!(
            include_str!("../actions/xml/invalid_action.xml"),
            code = 401,
            err_msg = "Invalid Action"
        ))
}

pub fn config(configure: &mut ServiceConfig) {
    configure
        .service(
            web::scope("/dlna")
                .service(avtransport_xml)
                .service(rendering_control_xml)
                .service(connection_manager_xml),
        )
        .service(
            web::scope("/AVTransport")
                .service(avtransport_action)
                .route(
                    "/event",
                    web::method(Method::from_bytes(b"SUBSCRIBE").unwrap()).to(avtransport_event),
                ),
        )
        .service(web::scope("/ConnectionManager").service(connection_manager_action))
        .service(web::scope("/RenderingControl").service(rendering_control_action));
}
