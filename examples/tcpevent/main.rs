use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

fn server() -> std::io::Result<()> {
    let tcp = TcpListener::bind("127.0.0.1:8080")?;
    loop {
        for stream in tcp.incoming() {
            println!("有链接进入了....");
            let mut stream = stream?;
            let mut buf = [0; 1024];
            let mut result = Vec::new();
            loop {
                let len = stream.read(&mut buf)?;
                println!("read len = {}", len);
                result.extend_from_slice(&buf[..len]);
                if len < buf.len() || buf[len - 1] == 0 {
                    break;
                }
            }
            println!("服务端读取到的数据：{}", String::from_utf8_lossy(&result));
            stream.write_all("yy1111".as_bytes())?;
            // let mut result = String::new();
            // stream.read_to_string(&mut result)?;
            // println!("还能收到吗？？{}", result);
        }
    }
}

fn main() -> std::io::Result<()> {
    std::thread::spawn(|| {
        server().unwrap();
    });
    std::thread::sleep(Duration::from_secs(1));
    let mut t = TcpStream::connect("127.0.0.1:8080")?;
    // let mut t = TcpStream::connect("192.169.1.44:10021")?;
    t.set_read_timeout(Some(Duration::from_secs(30)))?;
    let msg = br#"SUBSCRIBE /AVTransport/event HTTP/1.1
    HOST: 192.169.1.44:8080
    USER-AGENT: iOS/9.2.1 UPnP/1.1 SCDLNA/1.0
    CALLBACK: <http://192.168.1.100:5000/dlna/callback>
    NT: upnp:event
    TIMEOUT: Second-3600
    000
    "#;
    t.write_all(msg)?;
    println!("{:?}", msg);
    // let buf = [];
    // t.write_all(&buf)?;
    // t.shutdown(std::net::Shutdown::Write)?;
    // t.flush()?;
    println!("消息写入完毕");
    let mut result = Vec::new();
    t.read_to_end(&mut result).unwrap();
    println!("客户端收到的数据：{}", String::from_utf8_lossy(&result));

    // t.write_all(b"123123123123")?;
    // t.shutdown(std::net::Shutdown::Write)?;
    Ok(())
}
