use std::time::Instant;

use avtransport::av_transport_client::AvTransportClient;

use crate::avtransport::AvUri;

pub mod avtransport {
    tonic::include_proto!("avtransport");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AvTransportClient::connect("http://192.169.1.39:10021").await?;

    let request = tonic::Request::new(AvUri {
        uri: "https://media.w3.org/2010/05/sintel/trailer.mp4".to_string(),
        uri_meta_data: "".to_string(),
    });
    let now = Instant::now();
    let result = client.set_uri(request).await;
    println!("{:?}", result);
    println!("{:?}", now.elapsed());
    // let request = tonic::Request::new(HelloRequest {
    //     name: "Tonic".into(),
    // });

    // let response = client.say_hello(request).await?;

    // println!("RESPONSE={:?}", response);

    Ok(())
}
