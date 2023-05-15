use avtransport::{
    av_transport_server::{AvTransport, AvTransportServer},
    AvUri, Empty, PositionInfo, SeekPosition, TransportInfo, Volume, VolumeInfo, VolumeMute,
};
use tonic::{transport::Server, Request, Response, Status};

pub mod avtransport {
    tonic::include_proto!("avtransport");
}

#[derive(Default)]
struct MyAvTransport {}

#[tonic::async_trait]
impl AvTransport for MyAvTransport {
    async fn set_uri(&self, _request: Request<AvUri>) -> Result<Response<Empty>, Status> {
        Err(Status::aborted(""))
    }

    async fn get_position(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PositionInfo>, Status> {
        Err(Status::aborted(""))
    }

    async fn play(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }

    async fn stop(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }

    async fn pause(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }

    async fn seek(&self, _request: Request<SeekPosition>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }

    async fn get_transport_info(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<TransportInfo>, Status> {
        Err(Status::aborted(""))
    }

    async fn set_volume(&self, _request: Request<Volume>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }

    async fn get_volume(&self, _request: Request<Empty>) -> Result<Response<VolumeInfo>, Status> {
        Err(Status::aborted(""))
    }

    async fn set_mute(&self, _request: Request<VolumeMute>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:10023".parse().unwrap();

    let my_at = MyAvTransport::default();

    Server::builder()
        .add_service(AvTransportServer::new(my_at))
        .serve(addr)
        .await?;
    Ok(())
}
