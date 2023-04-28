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
