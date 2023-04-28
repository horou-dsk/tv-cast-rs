// use std::process::Command;

use actix_web::{
    get,
    http::{header, Method},
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};

use crate::actions::rpc_action::{self, ClientData};

// const MAX_SIZE: usize = 262_144;

#[get("AVTransport.xml")]
async fn avtransport_xml() -> impl Responder {
    // println!("read avtransport_xml");
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(include_str!("../xml/AVTransport.xml"))
}

#[get("ConnectionManager.xml")]
async fn connection_manager_xml() -> impl Responder {
    // println!("read connection_manager_xml");
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

#[post("/action")]
async fn avtransport_action(
    request: HttpRequest,
    bytes: web::Bytes,
    client: ClientData,
) -> impl Responder {
    let result = String::from_utf8_lossy(&bytes);
    rpc_action::on_action(&result, request, client).await
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
async fn connection_manager_action(_bytes: web::Bytes) -> impl Responder {
    // let result = String::from_utf8_lossy(&bytes);
    // println!("connection_manager_action = \n{}", result);
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
    // println!("read rendering_control_xml");
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/xml"))
        .body(include_str!("../xml/RenderingControl.xml"))
}

#[post("/action")]
async fn rendering_control_action(bytes: web::Bytes, client: ClientData) -> impl Responder {
    let result = String::from_utf8_lossy(&bytes);
    rpc_action::on_render_control_action(&result, client).await
    // HttpResponse::InternalServerError()
    //     .append_header((header::CONTENT_TYPE, "text/xml"))
    //     .body(format!(
    //         include_str!("../actions/xml/invalid_action.xml"),
    //         code = 401,
    //         err_msg = "Invalid Action"
    //     ))
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
