#![allow(dead_code)]
#![allow(unused_imports)]

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};

use hztp::{
    actions::avtransport::android::{EachAction, PositionInfo, SeekTarget},
    net::tcp_client,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct A {
    name: String,
    age: u32,
}

fn server() -> std::io::Result<()> {
    let tcp = TcpListener::bind("127.0.0.1:8080")?;
    for stream in tcp.incoming() {
        println!("有链接进入了....");
        let mut stream = stream?;
        // let mut result = String::new();
        // stream.read_to_string(&mut result)?;
        let result: EachAction<SeekTarget> = serde_json::from_reader(&mut stream).unwrap();
        // let mut buf = [0; 1024];
        // let mut result = Vec::new();
        // loop {
        //     let len = stream.read(&mut buf)?;
        //     println!("read len = {}", len);
        //     result.extend_from_slice(&buf[..len]);
        //     if len < buf.len() || buf[len - 1] == 0 {
        //         break;
        //     }
        // }
        println!("服务端读取到的数据：{:#?}", result);
        let to = EachAction::new(
            &result.action,
            Some(PositionInfo {
                track_duration: "00:10:11".to_string(),
                rel_time: "00:05:15".to_string(),
            }),
        );
        let to = serde_json::to_vec(&to).unwrap();
        stream.write_all(&to)?;
        // let mut result = String::new();
        // stream.read_to_string(&mut result)?;
        // println!("还能收到吗？？{}", result);
        // if true {
        // break;
        // }
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::thread::spawn(|| {
    //     server().unwrap();
    // });
    // std::thread::sleep(Duration::from_secs(3));
    // let mut t = TcpStream::connect("127.0.0.1:8080")?;
    // let mut t = TcpStream::connect("192.169.1.41:10021")?;
    // let now = Instant::now();
    // let a: EachAction<PositionInfo> = tcp_client::send(EachAction::new(
    //     "Seek",
    //     SeekTarget {
    //         target: "00:03:30".to_string(),
    //     },
    // ))
    // .await?;
    // let a: EachAction<PositionInfo> = tcp_client::send(EachAction::only_action("GetPositionInfo"))
    // .await?;
    // let a: EachAction<PositionInfo> = tcp_client::send(EachAction::only_action("Play"))
    // .await?;
    let a: EachAction<PositionInfo> = tcp_client::send(EachAction::only_action("Stop"))
    .await?;
    println!("客户端收到的数据111：{:#?}", a);
    // tokio::spawn(async {
    //     let a: EachAction<PositionInfo> = tcp_client::send(EachAction::new(
    //         "GetPositionInfo",
    //         SeekTarget {
    //             target: "00:10:50".to_string(),
    //         },
    //     ))
    //     .await
    //     .unwrap();
    //     println!("客户端收到的数据111：{:#?}", a);
    // });
    // tokio::spawn(async {
    //     let a: EachAction<PositionInfo> = tcp_client::send(EachAction::new(
    //         "Seek",
    //         SeekTarget {
    //             target: "00:00:50".to_string(),
    //         },
    //     ))
    //     .await
    //     .unwrap();
    //     println!("客户端收到的数据：{:#?}", a);
    // });
    // tokio::time::sleep(Duration::from_secs(10)).await;
    // println!("{:?}", now.elapsed());
    // std::thread::sleep(Duration::from_secs(3));
    // t.set_read_timeout(Some(Duration::from_secs(30)))?;
    // let msg = br#"SUBSCRIBE /AVTransport/event HTTP/1.1
    // HOST: 192.169.1.44:8080
    // USER-AGENT: iOS/9.2.1 UPnP/1.1 SCDLNA/1.0
    // CALLBACK: <http://192.168.1.100:5000/dlna/callback>
    // NT: upnp:event
    // TIMEOUT: Second-3600
    // 000
    // "#;
    // t.write_all(msg)?;
    // t.shutdown(std::net::Shutdown::Write)?;
    // println!("{:?}", msg);
    // // let buf = [];
    // // t.write_all(&buf)?;
    // // t.shutdown(std::net::Shutdown::Write)?;
    // // t.flush()?;
    // println!("消息写入完毕");
    // let mut result = Vec::new();
    // t.read_to_end(&mut result).unwrap();
    // println!("客户端收到的数据：{}", String::from_utf8_lossy(&result));

    // t.write_all(b"123123123123")?;
    Ok(())
}
