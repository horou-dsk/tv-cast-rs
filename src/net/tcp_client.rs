use std::{
    fmt::Debug,
    io::{Error, ErrorKind},
    time::Duration,
};

use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

use crate::constant::ANDROID_ADDR;

#[inline]
pub async fn send<S, R>(data: S) -> std::io::Result<R>
where
    S: Serialize + Debug,
    R: DeserializeOwned,
{
    let mut stream = match timeout(Duration::from_secs(1), TcpStream::connect(ANDROID_ADDR)).await {
        Ok(stream) => match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("{data:?} 连接失败...{err:?}");
                return Err(err);
            }
        },
        Err(err) => {
            eprintln!("{data:?} 连接超时...{err:?}");
            return Err(std::io::ErrorKind::TimedOut.into());
        }
    };
    // 该方法为同步IO独有
    // stream
    //     .set_read_timeout(Some(Duration::from_secs(10)))
    //     .await?;
    let v = serde_json::to_vec(&data).unwrap();
    stream.write_all(&v).await?;
    // serde_json::to_writer(&mut stream.rea, &data).unwrap();
    stream.shutdown().await?;
    let mut result = Vec::new();
    // let mut buf = [0; 1024];
    // stream.read_exact(&mut buf).await?;
    match timeout(Duration::from_secs(3), stream.read_to_end(&mut result)).await {
        Ok(r) => {
            if let Err(err) = r {
                eprintln!("{data:?} 读取失败... {err:?}");
            }
        }
        Err(err) => {
            eprintln!("{data:?} 读取超时... {err:?}");
        }
    }
    // stream.read_to_string(&mut result)?;
    // println!(
    //     "接收到的字符串数据：\n{}\n",
    //     String::from_utf8_lossy(&result)
    // );
    if result.is_empty() {
        Err(Error::from(ErrorKind::InvalidData))
    } else {
        Ok(serde_json::from_slice(&result).expect("json解析失败，格式错误！"))
    }
    // Ok(serde_json::from_reader(stream).unwrap())
}
