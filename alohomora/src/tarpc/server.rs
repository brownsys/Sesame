use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::Poll;

use aws_lc_rs::aead::{RandomizedNonceKey, AES_128_GCM};
use aws_lc_rs::agreement::{self, agree_ephemeral, PublicKey, UnparsedPublicKey};
use aws_lc_rs::error::Unspecified;
use aws_lc_rs::kdf::{self, get_sskdf_hmac_algorithm, sskdf_hmac, SskdfHmacAlgorithmId};

#[cfg(feature="tahini_server")]
use tahini_attest::server::get_key_for_client;

use crate::tarpc::{enums::TahiniSafeWrapper, traits::TahiniType};
use futures::{FutureExt, Sink, Stream};
use pin_project_lite::pin_project;
use tarpc::context::Context;
use tarpc::server::BaseChannel as TarpcBaseChannel;
use tarpc::server::Channel as TarpcChannel;
use tarpc::server::Serve as TarpcServe;
use tarpc::server::{Config, TrackedRequest};
use tarpc::{ChannelError, ClientMessage, Response, ServerError, Transport};

use super::transport::{KeyEngineState, TahiniTransportTrait};

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
        Trans: TahiniTransportTrait<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>
    {
        #[pin]
        channel: TarpcBaseChannel<Req, TahiniSafeWrapper<Resp>, Trans::InnerChannelType>,
        key_engine:  KeyEngineState
    }
}

impl<Req, Resp, Trans> TahiniBaseChannel<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: TahiniTransportTrait<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>,
{
    pub fn new(config: Config, transport: Trans) -> Self {
        let engine = transport.get_engine();
        Self {
            channel: TarpcBaseChannel::new(config, transport.get_inner()),
            key_engine: engine,
        }
    }
    pub fn with_defaults(transport: Trans) -> Self {
        let engine = transport.get_engine();
        Self {
            channel: TarpcBaseChannel::with_defaults(transport.get_inner()),
            key_engine: engine,
        }
    }
}

impl<Req, Resp, Trans> TahiniChannel for TahiniBaseChannel<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: TahiniTransportTrait<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>,
{
    type Req = Req;
    type Resp = Resp;
    type Transport = Trans::InnerChannelType;

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
        self.channel
            .execute(ServeAdapter::new(serve, self.key_engine))
    }
}

impl<Req, Resp, Trans, ChannelType> Sink<Response<TahiniSafeWrapper<Resp>>>
    for TahiniBaseChannel<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: TahiniTransportTrait<
        Response<TahiniSafeWrapper<Resp>>,
        ClientMessage<Req>,
        InnerChannelType = ChannelType,
    >,
    ChannelType: Transport<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>,
    ChannelType::Error: Error,
{
    type Error = ChannelError<ChannelType::TransportError>;
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
impl<Req, Resp, Trans, ChannelType> Stream for TahiniBaseChannel<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: TahiniTransportTrait<
        Response<TahiniSafeWrapper<Resp>>,
        ClientMessage<Req>,
        InnerChannelType = ChannelType,
    >,
    ChannelType: Transport<Response<TahiniSafeWrapper<Resp>>, ClientMessage<Req>>,
    ChannelType::Error: Error,
{
    type Item = Result<TrackedRequest<Req>, ChannelError<ChannelType::Error>>;
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

    async fn attest_serve(
        self,
        ctx: Context,
        req: Self::Req,
        lock_ref: Arc<OnceLock<RandomizedNonceKey>>,
    ) -> Result<Self::Resp, ServerError>
    where
        Self: Sized;
}

#[derive(Clone)]
struct ServeAdapter<T: TahiniServe> {
    tahini_serve: T,
    key_engine: KeyEngineState,
}
impl<T: TahiniServe> ServeAdapter<T> {
    fn new(tahini_serve: T, key_engine: KeyEngineState) -> Self {
        Self {
            tahini_serve,
            key_engine,
        }
    }

}

#[cfg(feature="tahini_server")]
pub fn get_session_key_for_client(client_id: usize) -> RandomizedNonceKey {
    let client_id = tahini_attest::types::ClientId::from(client_id);
    get_key_for_client(&client_id)
}

impl<T: TahiniServe> TarpcServe for ServeAdapter<T> {
    type Req = T::Req;
    type Resp = TahiniSafeWrapper<T::Resp>;

    async fn serve(self, ctx: Context, req: Self::Req) -> Result<Self::Resp, ServerError> {
        if self.key_engine.key.get().is_none() {
            self.tahini_serve
                .attest_serve(ctx, req, self.key_engine.key)
                .map(|res| res.map(|rsp| TahiniSafeWrapper(rsp)))
                .await
        } else {
            self.tahini_serve
                .serve(ctx, req)
                .map(|res| res.map(|rsp| TahiniSafeWrapper(rsp)))
                .await
        }
    }
    // fn method(&self, request: &Self::Req) -> Option<&'static str> {
    //     Some(T::Req::enum_name(request))
    // }
}
