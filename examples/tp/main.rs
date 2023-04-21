use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[actix_web::main]
async fn main() -> tokio::io::Result<()> {
    let mut t = TcpStream::connect("192.169.1.19:49153").await?;
    let b = r#"POST /upnp/control/rendertransport1 HTTP/1.0
HOST: 192.169.1.19:49153
Content-Type: text/xml; charset="utf-8"
Content-Length: 2900
SOAPACTION: "urn:schemas-upnp-org:service:AVTransport:1#SetAVTransportURI"
User-Agent: Linux/3.0.0 UPnP/1.0 Platinum/1.0.5.13

<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"><s:Body><u:SetAVTransportURI xmlns:u="urn:schemas-upnp-org:service:AVTransport:1"><InstanceID>0</InstanceID><CurrentURI>http://cn-scya-ct-01-01.bilivideo.com/upgcxcode/04/67/1086236704/1086236704-1-208.mp4?e=ig8euxZM2rNcNbRjnWdVhwdlhWTHhwdVhoNvNC8BqJIzNbfqXBvEuENvNC8aNEVEtEvE9IMvXBvE2ENvNCImNEVEIj0Y2J_aug859r1qXg8gNEVE5XREto8z5JZC2X2gkX5L5F1eTX1jkXlsTXHeux_f2o859IMvNC8xNbLEkF6MuwLStj8fqJ0EkX1ftx7Sqr_aio8_&amp;ua=tvproj&amp;uipk=5&amp;nbs=1&amp;deadline=1682084268&amp;gen=playurlv2&amp;os=bcache&amp;oi=3738422470&amp;trid=000026efa8df521e4f2f87923760d582431eT&amp;mid=1986490403&amp;upsig=4e38df49f82dfbce9a99275ac2a4d8df&amp;uparams=e,ua,uipk,nbs,deadline,gen,os,oi,trid,mid&amp;cdnid=62701&amp;bvc=vod&amp;nettype=0&amp;bw=169952&amp;orderid=0,3&amp;buvid=XX45C9B48BAD14D905F5B61034AC86220F7BF&amp;build=6400400&amp;mobi_app=android&amp;logo=80000000&amp;_nva_ext_=</CurrentURI><CurrentURIMetaData>&lt;DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/" xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/"&gt;&lt;item id="0" parentID="-1" restricted="1"&gt;&lt;dc:title&gt;第1话 - 新世界地图&lt;/dc:title&gt;&lt;upnp:longDescription&gt;HqUwMkMlPDavpnmv8JTHHxAtf2nFRxYf99FcRjLYMe-qjDIcDQxpje6jhBIIzqP5pSPkt-aF6IEtwxkkwSDYEnXAFI2V49ZZpWA3sMBidVFJY0uB93S0sjq4JlMA_9zKY-s25r4klmgYdiW4OOIKHTQRzwAhtDAI2Wgcqw2PLRalHmByoTlSB_xwch2YIuAeEySN1K-4yID3u0KMIfylvOgLzFsymM-2BgEadFaPqDWVga7haz8mKXTiE-dMv5KnRMwGDOaR1opW_eHwgkTRq_fiDLH3SqtHLa_hf0DZKAw&lt;/upnp:longDescription&gt;&lt;res protocolInfo="http-get:*:video/mp4:DLNA.ORG_PN=MPEG4_P2_SP_AAC;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01500000000000000000000000000000"&gt;http://cn-scya-ct-01-01.bilivideo.com/upgcxcode/04/67/1086236704/1086236704-1-208.mp4?e=ig8euxZM2rNcNbRjnWdVhwdlhWTHhwdVhoNvNC8BqJIzNbfqXBvEuENvNC8aNEVEtEvE9IMvXBvE2ENvNCImNEVEIj0Y2J_aug859r1qXg8gNEVE5XREto8z5JZC2X2gkX5L5F1eTX1jkXlsTXHeux_f2o859IMvNC8xNbLEkF6MuwLStj8fqJ0EkX1ftx7Sqr_aio8_&amp;amp;ua=tvproj&amp;amp;uipk=5&amp;amp;nbs=1&amp;amp;deadline=1682084268&amp;amp;gen=playurlv2&amp;amp;os=bcache&amp;amp;oi=3738422470&amp;amp;trid=000026efa8df521e4f2f87923760d582431eT&amp;amp;mid=1986490403&amp;amp;upsig=4e38df49f82dfbce9a99275ac2a4d8df&amp;amp;uparams=e,ua,uipk,nbs,deadline,gen,os,oi,trid,mid&amp;amp;cdnid=62701&amp;amp;bvc=vod&amp;amp;nettype=0&amp;amp;bw=169952&amp;amp;orderid=0,3&amp;amp;buvid=XX45C9B48BAD14D905F5B61034AC86220F7BF&amp;amp;build=6400400&amp;amp;mobi_app=android&amp;amp;logo=80000000&amp;amp;_nva_ext_=&lt;/res&gt;&lt;upnp:class&gt;object.item.videoItem&lt;/upnp:class&gt;&lt;/item&gt;&lt;/DIDL-Lite&gt;</CurrentURIMetaData></u:SetAVTransportURI></s:Body></s:Envelope>"#;
    t.write_all(b.as_bytes()).await?;
    t.shutdown().await?;
    let mut result = String::new();
    t.read_to_string(&mut result).await?;
    println!("{}", result);
    Ok(())
}
