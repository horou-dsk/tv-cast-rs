use std::net::TcpStream;

use serde::{de::DeserializeOwned, Serialize};

pub fn send<S, T>(data: S) -> std::io::Result<T>
where
    S: Serialize,
    T: DeserializeOwned,
{
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    serde_json::to_writer(&mut stream, &data).unwrap();
    stream.shutdown(std::net::Shutdown::Write)?;
    Ok(serde_json::from_reader(stream).unwrap())
}
