use hztp::actions::{avtransport::{AVTransportAction, GetPositionInfoResponse}, XmlToString};
use quick_xml::se::Serializer;
use serde::Serialize;

const XML_TEXT: &str = r#"<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" xmlns:u="urn:schemas-upnp-org:service:AVTransport:1"><s:Body><u:SetAVTransportURI xmlns:u="urn:schemas-upnp-org:service:AVTransport:1"><InstanceID>0</InstanceID><CurrentURI>http://tx-safety-video.acfun.cn/mediacloud/acfun/acfun_video/c627f9e9f776b4d8-e1f7ff7766dce1c3e81f9a41abf34a63-hls_1080p_2.m3u8?pkey=ABAb8Ozr5YBbXnxbQ5SrFBUaAsy1W-2ErRyIbxcvHBhqtogZpSauyNDT3A5_RmFCxNw_9yctOE9vCV_3GKxfdaPkM_o4Uy_jvBYzVJQAG6k-MDs_NrFcHeMpdySY1c1DuDLXh-3xjgFwmsOnOfP8qwslhG_IQav0lfrfDllDQuwys1wHutnqrbCNAcLqkDuiSoQGD5truFYgZSNSJKMXhOy3TH5lWnAb9vmvFTbZmeDKZP_EEBYIgRSVAPcRdvS0aDrqr3vKx0p5Ax04UFwbxwSxjXSUMSnddagDY4TMArI5SQ</CurrentURI><CurrentURIMetaData>&lt;DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/"&gt;&lt;item id="123" parentID="-1" restricted="1"&gt;&lt;res protocolInfo="http-get:*:video/*:*;DLNA.ORG_OP=01;DLNA.ORG_FLAGS=01700000000000000000000000000000"&gt;http://tx-safety-video.acfun.cn/mediacloud/acfun/acfun_video/c627f9e9f776b4d8-e1f7ff7766dce1c3e81f9a41abf34a63-hls_1080p_2.m3u8?pkey=ABAb8Ozr5YBbXnxbQ5SrFBUaAsy1W-2ErRyIbxcvHBhqtogZpSauyNDT3A5_RmFCxNw_9yctOE9vCV_3GKxfdaPkM_o4Uy_jvBYzVJQAG6k-MDs_NrFcHeMpdySY1c1DuDLXh-3xjgFwmsOnOfP8qwslhG_IQav0lfrfDllDQuwys1wHutnqrbCNAcLqkDuiSoQGD5truFYgZSNSJKMXhOy3TH5lWnAb9vmvFTbZmeDKZP_EEBYIgRSVAPcRdvS0aDrqr3vKx0p5Ax04UFwbxwSxjXSUMSnddagDY4TMArI5SQ&lt;/res&gt;&lt;upnp:storageMedium&gt;UNKNOWN&lt;/upnp:storageMedium&gt;&lt;upnp:writeStatus&gt;UNKNOWN&lt;/upnp:writeStatus&gt;&lt;dc:title&gt;01&lt;/dc:title&gt;&lt;upnp:class&gt;object.item.videoItem&lt;/upnp:class&gt;&lt;/item&gt;&lt;/DIDL-Lite&gt;</CurrentURIMetaData></u:SetAVTransportURI></s:Body></s:Envelope>"#;

fn main() {
    let body = AVTransportAction::from_xml_text(XML_TEXT).unwrap();
    let mut buf = String::new();
    let ser = Serializer::new(&mut buf);
    let resp = GetPositionInfoResponse {
        track: 0,
        track_duration: "00:04:32",
        track_meta_data: None,
        track_uri: None,
        rel_time: "00:10:00",
        abs_time: "00:10:00",
        rel_count: 2147483647,
        abs_count: 2147483647,
        ..Default::default()
    };
    resp.serialize(ser).unwrap();
    println!("{}", resp.xml());
    // println!("{:?}", body);
}
