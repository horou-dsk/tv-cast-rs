use std::time::Instant;

use avtransport::{av_transport_client::AvTransportClient, Empty};

pub mod avtransport {
    tonic::include_proto!("avtransport");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AvTransportClient::connect("http://192.169.1.19:50051").await?;

    let request = tonic::Request::new(Empty {});
    let now = Instant::now();
    let result = client.play(request).await;
    println!("{:?}", result);
    println!("{:?}", now.elapsed());
    // let request = tonic::Request::new(HelloRequest {
    //     name: "Tonic".into(),
    // });

    // let response = client.say_hello(request).await?;

    // println!("RESPONSE={:?}", response);

    Ok(())
}
