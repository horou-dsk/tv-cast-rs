use hztp::actions::avtransport::android::EachAction;
use pin_project_lite::pin_project;
use tokio::sync::oneshot::{error::RecvError, Receiver};

pin_project! {
    pub struct UdpResult<T> {
        each_action: EachAction<T>,
        #[pin]
        rx: Receiver<T>,
    }
}

impl<T> std::future::Future for UdpResult<T> {
    type Output = Result<T, RecvError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let s = self.project();
        s.rx.poll(cx)
    }
}

fn main() {}
