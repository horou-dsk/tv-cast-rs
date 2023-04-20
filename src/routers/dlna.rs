// use std::process::Command;

use actix_web::{
    get,
    http::{header, Method},
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};

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

static mut RUNNING: bool = false;

#[post("/action")]
async fn avtransport_action(request: HttpRequest, bytes: web::Bytes) -> impl Responder {
    let host = request.headers().get("Host");
    let result = String::from_utf8_lossy(&bytes);
    println!("{host:?} avtransport_action = {result}\n\n");
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
                        unsafe { RUNNING = false };
                    }
                    GetTransportInfoResponse {
                        current_speed: "1",
                        current_transport_state: state,
                        current_transport_status: "OK",
                        ..Default::default()
                    }
                }
                Err(_) => GetTransportInfoResponse {
                    current_speed: "1",
                    current_transport_state: if unsafe { RUNNING } {
                        "PLAYING".to_string()
                    } else {
                        "STOPPED".to_string()
                    },
                    current_transport_status: "OK",
                    ..Default::default()
                },
            };
            AVTransportResponse::ok(resp)
        }
        Ok(AVTransportAction::SetAVTransportURI(av_uri)) => {
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
            unsafe { RUNNING = true };
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
            unsafe { RUNNING = false };
            AVTransportResponse::default_ok("Stop")
        }
        Ok(AVTransportAction::GetPositionInfo) => {
            if let Ok(resp) = tcp_client::send::<_, EachAction<PositionInfo>>(
                EachAction::only_action("GetPositionInfo"),
            )
            .await
            {
                let data = resp.data.unwrap();
                AVTransportResponse::ok(GetPositionInfoResponse {
                    track: 0,
                    track_duration: &data.track_duration,
                    abs_time: &data.rel_time,
                    rel_time: &data.rel_time,
                    abs_count: 2147483646,
                    rel_count: 2147483646,
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
