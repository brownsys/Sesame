use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use futures::{FutureExt, Sink, Stream};
use pin_project_lite::pin_project;
use tarpc::context::Context;
use tarpc::server::BaseChannel as TarpcBaseChannel;
use tarpc::server::Channel as TarpcChannel;
use tarpc::server::Serve as TarpcServe;
use tarpc::server::{Config, TrackedRequest};
use tarpc::{ChannelError, ClientMessage, Response, ServerError, Transport};
use crate::tarpc::{enums::TahiniSafeWrapper, traits::TahiniType};

pub trait TahiniChannel
where
    Self: Sized,
{
    type Req: TahiniType;
    type Resp: TahiniType;
    type Transport: Transport<Response<TahiniSafeWrapper<Self::Resp>>, ClientMessage<Self::Req>>;

    fn config(&self) -> &Config;
    fn in_flight_requests(&self) -> usize;

    fn transport(&self) -> &Self::Transport;

    fn execute<S>(self, serve: S) -> impl Stream<Item = impl Future<Output = ()>>
    where
        S: TahiniServe<Req = Self::Req, Resp = Self::Resp> + Clone;
}

pin_project! {
    pub struct TahiniBaseChannel<Req, Resp, Trans>
    where
        Req: TahiniType,
        Resp: TahiniType,
        Trans: Transport<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>
    {
        #[pin]
        channel: TarpcBaseChannel<Req, TahiniSafeWrapper<Resp>, Trans>
    }
}

impl<Req, Resp, Trans> TahiniBaseChannel<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: Transport<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>,
{
    pub fn new(config: Config, transport: Trans) -> Self {
        Self {
            channel: TarpcBaseChannel::new(config, transport),
        }
    }
    pub fn with_defaults(transport: Trans) -> Self {
        Self {
            channel: TarpcBaseChannel::with_defaults(transport),
        }
    }
}

impl<Req, Resp, Trans> TahiniChannel for TahiniBaseChannel<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: Transport<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>,
{
    type Req = Req;
    type Resp = Resp;
    type Transport = Trans;

    fn config(&self) -> &Config {
        self.channel.config()
    }

    fn in_flight_requests(&self) -> usize {
        self.channel.in_flight_requests()
    }

    fn transport(&self) -> &Self::Transport {
        self.channel.transport()
    }

    fn execute<S>(self, serve: S) -> impl Stream<Item = impl Future<Output = ()>>
    where
        Self: Sized,
        S: TahiniServe<Req = Self::Req, Resp = Self::Resp> + Clone,
    {
        self.channel.execute(ServeAdapter::new(serve))
    }
}

impl<Req, Resp, Trans> Sink<Response<TahiniSafeWrapper<Resp>>> for TahiniBaseChannel<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: Transport<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>,
    Trans::Error: Error,
{
    type Error = ChannelError<Trans::Error>;
    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        TarpcBaseChannel::poll_ready(self.project().channel, cx)
    }
    fn start_send(
        self: Pin<&mut Self>,
        item: Response<TahiniSafeWrapper<Resp>>,
    ) -> Result<(), Self::Error> {
        TarpcBaseChannel::start_send(self.project().channel, item)
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        TarpcBaseChannel::poll_flush(self.project().channel, cx)
    }
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        TarpcBaseChannel::poll_close(self.project().channel, cx)
    }
}

//
impl<Req, Resp, Trans> Stream for TahiniBaseChannel<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: Transport<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>,
{
    type Item = Result<TrackedRequest<Req>, ChannelError<Trans::Error>>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        TarpcBaseChannel::poll_next(self.project().channel, cx)
    }
}
// // mimics `tarpc::server::Serve`
#[allow(async_fn_in_trait)]
pub trait TahiniServe {
    /// Type of request.
    type Req: TahiniType;

    /// Type of response.
    type Resp: TahiniType + Send;

    /// Extracts a method name from the request.
    fn method(&self, _request: &Self::Req) -> Option<&'static str> {
        None
    }

    /// Responds to a single request.
    async fn serve(self, ctx: Context, req: Self::Req) -> Result<Self::Resp, ServerError>;
}
//
// // Private struct for us!
#[derive(Clone)]
struct ServeAdapter<T: TahiniServe> {
    tahini_serve: T,
}
impl<T: TahiniServe> ServeAdapter<T> {
    pub fn new(tahini_serve: T) -> Self {
        Self { tahini_serve }
    }
}

impl<T: TahiniServe> TarpcServe for ServeAdapter<T> {
    type Req = T::Req;
    type Resp = TahiniSafeWrapper<T::Resp>;

    async fn serve(self, ctx: Context, req: Self::Req) -> Result<Self::Resp, ServerError> {
        self.tahini_serve
            .serve(ctx, req)
            .map(|res| res.map(|rsp| TahiniSafeWrapper(rsp)))
            .await
    }
    // fn method(&self, request: &Self::Req) -> Option<&'static str> {
    //     Some(T::Req::enum_name(request))
    // }
}
