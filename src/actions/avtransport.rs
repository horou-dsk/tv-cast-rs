use actix_web::{http::header, HttpResponse};
use quick_xml::se::Serializer;
use quick_xml::{de::from_str, events::Event, Error, Reader, Result};
use serde::{Deserialize, Serialize};

use super::XmlToString;

const XML_ROOT: &str = r#"<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" xmlns:u="urn:schemas-upnp-org:service:AVTransport:1"><s:Body>{body_content}</s:Body></s:Envelope>"#;

#[derive(Debug, Deserialize)]
pub struct InstanceID {
    #[serde(rename = "$text")]
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetTransportInfo {
    #[serde(rename = "InstanceID")]
    pub id: InstanceID,
}

#[derive(Debug, Deserialize)]
pub struct SetAVTransportURI {
    #[serde(rename = "InstanceID")]
    pub id: InstanceID,
    #[serde(rename = "CurrentURI")]
    pub uri: String,
    #[serde(rename = "CurrentURIMetaData")]
    pub uri_meta_data: String,
}

#[derive(Debug, Deserialize)]
pub struct Play {
    #[serde(rename = "InstanceID")]
    pub id: InstanceID,
    #[serde(rename = "Speed")]
    pub speed: f32,
}

#[derive(Debug, Deserialize)]
pub struct Stop {
    #[serde(rename = "InstanceID")]
    pub id: InstanceID,
}

#[derive(Debug, Deserialize)]
pub struct Seek {
    #[serde(rename = "Target")]
    pub target: String,
}

#[derive(Debug, Deserialize)]
pub enum AVTransportAction {
    GetTransportInfo(GetTransportInfo),
    SetAVTransportURI(SetAVTransportURI),
    Play(Play),
    Stop(Stop),
    GetPositionInfo,
    Pause,
    Seek(Seek),
    GetMediaInfo,
}

impl AVTransportAction {
    pub fn from_xml_text(xml: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                // exits the loop when reaching end of file
                Ok(Event::Eof) => break,

                Ok(Event::Start(e)) if b"s:Body" == e.name().as_ref() => {
                    let result = reader.read_text(e.name())?;
                    let info = from_str(&result).map_err(|_| Error::TextNotFound);
                    return info;
                }

                // There are several other `Event`s we do not consider here
                _ => (),
            }
        }
        Err(Error::TextNotFound)
    }
}

xml_response! {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "PascalCase")]
    GetPositionInfoResponse<'a> {
        pub track: u8,
        pub track_duration: &'a str,
        pub track_meta_data: Option<&'a str>,
        #[serde(rename = "TrackURI")]
        pub track_uri: Option<&'a str>,
        pub rel_time: &'a str,
        pub abs_time: &'a str,
        pub rel_count: i32,
        pub abs_count: i32,
    }
}

xml_response! {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "PascalCase")]
    GetTransportInfoResponse<'a> {
        pub current_transport_state: String,
        pub current_transport_status: &'a str,
        pub current_speed: &'a str,
    }
}

xml_response! {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "PascalCase")]
    GetMediaInfoResponse<'a> {
        pub nr_tracks: &'a str,
        pub media_duration: &'a str,
        pub current_uri: &'a str,
        pub current_uri_meta_data: &'a str,
        pub next_uri: Option<&'a str>,
        pub next_uri_meta_data: Option<&'a str>,
        pub play_medium: &'a str,
        pub record_medium: &'a str,
        pub write_status: &'a str,
    }
}

pub struct AVTransportResponse;

impl AVTransportResponse {
    pub fn default_ok(action: &str) -> HttpResponse {
        HttpResponse::Ok()
            .append_header((header::CONTENT_TYPE, "text/xml"))
            .body(XML_ROOT.replace(
                "{body_content}",
                &format!(
                    r#"<u:{action}Response xmlns:u="urn:schemas-upnp-org:service:AVTransport:1"/>"#
                ),
            ))
    }

    pub fn ok(xml_body: impl XmlToString) -> HttpResponse {
        HttpResponse::Ok()
            .append_header((header::CONTENT_TYPE, "text/xml"))
            .body(XML_ROOT.replace("{body_content}", &xml_body.xml()))
    }

    pub fn err(code: u16, err_msg: &str) -> HttpResponse {
        HttpResponse::InternalServerError()
            .append_header((header::CONTENT_TYPE, "text/xml"))
            .body(format!(
                include_str!("./xml/invalid_action.xml"),
                code = code,
                err_msg = err_msg
            ))
    }
}

pub mod android {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct EachAction<T = u8> {
        pub action: String,
        pub data: Option<T>,
    }

    impl EachAction<u8> {
        pub fn only_action(action: &str) -> Self {
            Self {
                action: action.into(),
                data: None,
            }
        }
    }

    impl<T> EachAction<T> {
        pub fn new(action: &str, data: T) -> Self {
            Self {
                action: action.into(),
                data: Some(data),
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TransportState {
        pub current_transport_state: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PositionInfo {
        pub track_duration: String,
        pub rel_time: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SeekTarget {
        pub target: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AVTransportURI {
        pub uri: String,
        pub uri_meta: String,
    }
}

pub mod rpc {
    tonic::include_proto!("avtransport");
}

pub mod client {
    use std::{
        net::{IpAddr, Ipv4Addr},
        sync::{Arc, LazyLock},
    };

    use actix_web::{web, HttpRequest, HttpResponse};
    use tokio::sync::{Mutex, RwLock};
    use tonic::{transport::Channel, Status};

    use crate::actions::avtransport::GetPositionInfoResponse;

    use super::{
        rpc::{av_transport_client::AvTransportClient, AvUri, Empty, SeekPosition},
        AVTransportAction, AVTransportResponse, GetTransportInfoResponse,
    };

    pub type ClientData = web::Data<Arc<Mutex<AvTransportClient<Channel>>>>;

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

    fn log_rpc_err(status: &Status) {
        println!("{status:?}");
    }

    pub async fn on_action(
        xml_text: &str,
        request: HttpRequest,
        client: ClientData,
    ) -> HttpResponse {
        let host = request
            .peer_addr()
            .map(|addr| addr.ip())
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
        // println!("{host} avtransport_action = \n{xml_text}\n");
        let mut client = client.lock().await;
        match AVTransportAction::from_xml_text(xml_text) {
            Ok(AVTransportAction::GetTransportInfo(_)) => {
                let resp = match client.get_transport_info(Empty {}).await {
                    Ok(rep) => {
                        let state = rep.into_inner().current_transport_state;
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
                    Err(err) => {
                        log_rpc_err(&err);
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
                println!("{xml_text}");
                let mut sa = SIGNLE_ACTION.write().await;
                let now = chrono::Local::now().naive_local();
                if sa.running
                    && sa.host != host
                    && (now - sa.timestamp) < chrono::Duration::seconds(5)
                {
                    return AVTransportResponse::err(401, "Invalid Action");
                }
                sa.host = host;
                sa.timestamp = now;
                sa.current_uri = av_uri.uri.clone();
                sa.current_uri_meta_data = av_uri.uri_meta_data.clone();
                drop(sa);
                // println!("\nmeta xml = {}\n", av_uri.uri_meta_data);
                if client
                    .set_uri(AvUri {
                        uri: av_uri.uri,
                        uri_meta_data: av_uri.uri_meta_data,
                    })
                    .await
                    .is_ok()
                {
                    SIGNLE_ACTION.write().await.running = true;
                }
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
                println!("{xml_text}");
                client.play(Empty {}).await.unwrap();
                AVTransportResponse::default_ok("Play")
            }
            Ok(AVTransportAction::Stop(_)) => {
                println!("{xml_text}");
                let sa = SIGNLE_ACTION.read().await;
                if sa.host == host {
                    drop(sa);
                    client.stop(Empty {}).await.ok();
                    SIGNLE_ACTION.write().await.running = false;
                }
                AVTransportResponse::default_ok("Stop")
            }
            Ok(AVTransportAction::GetPositionInfo) => {
                if let Ok(resp) = client.get_position(Empty {}).await.inspect_err(log_rpc_err) {
                    let data = resp.into_inner();
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
                client
                    .seek(SeekPosition {
                        target: seek.target,
                    })
                    .await
                    .ok();
                AVTransportResponse::default_ok("Seek")
            }
            Ok(AVTransportAction::Pause) => {
                client.pause(Empty {}).await.ok();
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
}
