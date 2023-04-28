use actix_web::{http::header, HttpResponse};
use quick_xml::{de::from_str, events::Event, Error, Reader};
use serde::Deserialize;

const XML_ROOT: &str = r#"<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" xmlns:u="urn:schemas-upnp-org:service:RenderingControl:1"><s:Body>{body_content}</s:Body></s:Envelope>"#;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SetVolume {
    pub channel: String,
    pub desired_volume: i32,
}

#[derive(Debug, Deserialize)]
pub enum RenderingControlAction {
    GetVolume,
    SetVolume(SetVolume),
}

impl RenderingControlAction {
    pub fn from_xml_text(xml: &str) -> quick_xml::Result<Self> {
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

pub struct RenderingControlResponse;

impl RenderingControlResponse {
    pub fn default_ok(action: &str) -> HttpResponse {
        HttpResponse::Ok()
            .append_header((header::CONTENT_TYPE, "text/xml"))
            .body(XML_ROOT.replace(
                "{body_content}",
                &format!(
                    r#"<u:{action}Response xmlns:u="urn:schemas-upnp-org:service:RenderingControl:1"/>"#
                ),
            ))
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
