// use std::time::Duration;

use std::time::Duration;

use rand::Rng;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[actix_web::main]
async fn main() -> tokio::io::Result<()> {
    let server = TcpListener::bind("127.0.0.1:8081").await?;
    let mut rng = rand::thread_rng();
    while let Ok((mut stream, _addr)) = server.accept().await {
        let millis = rng.gen_range(100..500);
        tokio::spawn(async move {
            let mut result = String::new();
            stream.read_to_string(&mut result).await.unwrap();
            println!("服务端收到的数据：{}", result);
            tokio::time::sleep(Duration::from_millis(millis)).await;
            stream.write_all(result.as_bytes()).await.unwrap();
            //             stream
            //                 .write_all(
            //                     br#"HTTP/1.1 200 OK
            // Date: Fri, 22 May 2009 06:07:21 GMT
            // Content-Type: text/html; charset=UTF-8

            // <html>
            //         <head></head>
            //         <body>
            //             Success...
            //         </body>
            // </html>"#,
            //                 )
            //                 .await
            //                 .unwrap();
            // tokio::time::sleep(Duration::from_secs(10)).await;
        });
    }
    // let msg = [0; 1024];
    // println!("{}", String::from_utf8_lossy(&msg));

    // let buf = "好玩好玩水电费卡圣诞福利温热12300021401阿萨德        ".as_bytes();
    // println!("{:?}", buf);
    Ok(())
}
