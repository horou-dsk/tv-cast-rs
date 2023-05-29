use std::{
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, LazyLock},
};

use actix_web::{web, HttpRequest, HttpResponse};
use tokio::sync::{Mutex, RwLock};

use crate::actions::avtransport::GetPositionInfoResponse;

use super::{
    avtransport::{AVTransportAction, AVTransportResponse, GetTransportInfoResponse},
    jni_action,
    renderingcontrol::{GetVolumeResponse, RenderingControlAction, RenderingControlResponse},
};

pub type ClientData = web::Data<Arc<Mutex<jni_action::AVTransportAction>>>;

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

// fn log_rpc_err(status: &Status) {
//     println!("{status:?}");
// }

pub async fn on_action(xml_text: &str, request: HttpRequest, client: ClientData) -> HttpResponse {
    let host = request
        .peer_addr()
        .map(|addr| addr.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    // println!("{host} avtransport_action = \n{xml_text}\n");
    let mut client = client.lock().await;
    match AVTransportAction::from_xml_text(xml_text) {
        Ok(AVTransportAction::GetTransportInfo(_)) => {
            let resp = match client.get_transport_info() {
                Ok(rep) => {
                    let state = rep.current_transport_state;
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
                    log::error!("{err:?}");
                    // log_rpc_err(&err);
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
            if sa.running && sa.host != host && (now - sa.timestamp) < chrono::Duration::seconds(5)
            {
                return AVTransportResponse::err(401, "Invalid Action");
            }
            sa.host = host;
            sa.timestamp = now;
            sa.current_uri = av_uri.uri.clone();
            sa.current_uri_meta_data = av_uri.uri_meta_data.clone();
            drop(sa);
            // println!("\nmeta xml = {}\n", av_uri.uri_meta_data);
            if client.set_uri(av_uri).is_ok() {
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
            client.play().unwrap();
            AVTransportResponse::default_ok("Play")
        }
        Ok(AVTransportAction::Stop(_)) => {
            println!("{xml_text}");
            let sa = SIGNLE_ACTION.read().await;
            if sa.host == host {
                drop(sa);
                client.stop().ok();
                SIGNLE_ACTION.write().await.running = false;
            }
            AVTransportResponse::default_ok("Stop")
        }
        Ok(AVTransportAction::GetPositionInfo) => {
            if let Ok(resp) = client
                .get_position()
                .inspect_err(|err| log::error!("{err:?}"))
            {
                let data = resp;
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
            client.seek(seek.target).ok();
            AVTransportResponse::default_ok("Seek")
        }
        Ok(AVTransportAction::Pause) => {
            client.pause().ok();
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
        _ => {
            println!("未实现Action = \n{}\n", xml_text);
            AVTransportResponse::err(401, "Invalid Action")
        }
    }
}

pub async fn on_render_control_action(xml_text: &str, client: ClientData) -> HttpResponse {
    let mut client = client.lock().await;
    match RenderingControlAction::from_xml_text(xml_text) {
        Ok(RenderingControlAction::SetVolume(volume)) => {
            client.set_volume(volume).unwrap();
            RenderingControlResponse::default_ok("SetVolume")
        }
        Ok(RenderingControlAction::GetVolume) => {
            let vol = client.get_volume().unwrap();
            RenderingControlResponse::ok(GetVolumeResponse {
                current_volume: vol.current_volume,
                ..Default::default()
            })
        }
        Ok(RenderingControlAction::SetMute(set_mute)) => {
            client.set_mute(set_mute.desired_mute).ok();
            RenderingControlResponse::default_ok("SetMute")
        }
        _ => {
            println!("未实现Action = \n{}\n", xml_text);
            RenderingControlResponse::err(401, "Invalid Action")
        }
    }
}
