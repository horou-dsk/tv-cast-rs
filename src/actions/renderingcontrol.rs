use quick_xml::{de::from_str, events::Event, Error, Reader};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum RenderingControlAction {
    GetVolume,
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
