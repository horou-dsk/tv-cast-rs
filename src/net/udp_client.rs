use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::{fmt::Debug, net::SocketAddr, time::Duration};
use tokio::sync::oneshot::Sender;
use tokio::sync::RwLock;
use tokio::{net::UdpSocket, time::timeout};
#[allow(dead_code)]
struct Msg {
    action: String,
    tx: Sender<String>,
}

pub struct UdpClient {
    socket: Arc<UdpSocket>,
    to_addr: SocketAddr,
    messages: Arc<RwLock<Vec<Msg>>>,
}

// pub struct UdpResult<T> {
//     each_action: EachAction<T>,
//     rx: Receiver<T>,
// }

// impl<T> std::future::Future for UdpResult<T> {
//     type Output = T;

//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         pin!(self);
//         self.rx.poll(cx)
//     }
// }

impl UdpClient {
    pub async fn start(&self) {
        let _msg = self.messages.clone();
        let socket = self.socket.clone();
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                if let Ok((size, _addr)) = socket.recv_from(&mut buf).await {
                    let _result = String::from_utf8_lossy(&buf[..size]);
                    // 后续消息类型判断和处理...
                }
            }
        });
    }

    pub async fn new(to: SocketAddr) -> Self {
        let socket = UdpSocket::bind("127.0.0.1:22336")
            .await
            .expect("建立udp失败");
        Self {
            socket: Arc::new(socket),
            to_addr: to,
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn send<S, R>(
        &self,
        data: S,
    ) -> std::io::Result<impl Future<Output = std::io::Result<R>>>
    where
        S: Serialize + Debug,
        R: DeserializeOwned,
    {
        let v = serde_json::to_vec(&data).unwrap();
        self.socket.send_to(&v, self.to_addr).await?;
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();
        let r = async move {
            let r = match timeout(Duration::from_secs(3), rx).await {
                Ok(r) => r,
                Err(err) => {
                    eprintln!("{data:?} 读取超时... {err:?}");
                    return Err(Error::from(ErrorKind::InvalidData));
                }
            };
            if let Ok(result) = r {
                Ok(serde_json::from_str(&result).unwrap())
            } else {
                eprintln!("{data:?} 读取失败...");
                Err(Error::from(ErrorKind::InvalidData))
            }
        };
        self.messages.write().await.push(Msg {
            action: "".to_string(),
            tx,
        });
        Ok(r)
    }
}
