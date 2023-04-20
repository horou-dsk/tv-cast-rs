use std::{
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
    S: Serialize,
    R: DeserializeOwned,
{
    let mut stream = timeout(Duration::from_secs(1), TcpStream::connect(ANDROID_ADDR)).await??;
    // 该方法为同步独有
    // stream
    //     .set_read_timeout(Some(Duration::from_secs(10)))
    //     .await?;
    let v = serde_json::to_vec(&data).unwrap();
    stream.write_all(&v).await?;
    // serde_json::to_writer(&mut stream.rea, &data).unwrap();
    stream.shutdown().await?;
    println!("发送完毕....");
    let mut result = Vec::new();
    // let mut buf = [0; 1024];
    // stream.read_exact(&mut buf).await?;
    timeout(Duration::from_secs(2), stream.read_to_end(&mut result)).await??;
    // stream.read_to_string(&mut result)?;
    println!(
        "接收到的字符串数据：\n{}\n",
        String::from_utf8_lossy(&result)
    );
    if result.is_empty() {
        Err(Error::from(ErrorKind::InvalidData))
    } else {
        Ok(serde_json::from_slice(&result).expect("json解析失败，格式错误！"))
    }
    // Ok(serde_json::from_reader(stream).unwrap())
}
