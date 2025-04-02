use crate::tarpc::context::TahiniContext;
use crate::tarpc::enums::TahiniSafeWrapper;
// use crate::tarpc::traits::{
//     deserialize_tahini_type, serialize_tahini_type, TahiniType, TahiniType2,
// };
use crate::bbox::BBox;
use crate::policy::{Policy, PolicyTransformable, TahiniPolicy};
use crate::tarpc::traits::TahiniType;
use pin_project_lite::pin_project;
use std::future::Future;
use tarpc::client::Channel as TarpcChannel;
use tarpc::client::NewClient as TarpcNewClient;
use tarpc::client::RequestDispatch as TarpcRequestDispatch;
use tarpc::client::{Config, RpcError};
use tarpc::{context, ChannelError, ClientMessage, Response, Transport};

#[derive(Clone)]
pub struct TahiniChannel<Req: TahiniType, Resp: TahiniType> {
    channel: TarpcChannel<TahiniSafeWrapper<Req>, Resp>,
    // phantom_req: PhantomData<Req>,
    // phantom_resp: PhantomData<Resp>,
}

impl<'a, Req: TahiniType, Resp: TahiniType> TahiniChannel<Req, Resp> {
    pub(crate) fn new(channel: TarpcChannel<TahiniSafeWrapper<Req>, Resp>) -> Self {
        Self { channel }
    }
}

// mimics `tarpc::client::Stub`.
#[allow(async_fn_in_trait)]
pub trait TahiniStub {
    type Req: TahiniType;

    /// The service response type.
    type Resp: TahiniType;

    /// Calls a remote service.
    async fn call(
        &self,
        ctx: context::Context,
        request_name: &'static str,
        request: Self::Req,
    ) -> Result<Self::Resp, RpcError>;

    async fn transform_and_call<
        T,
        TargetPolicy: Policy + serde::Serialize,
        SourcePolicy: PolicyTransformable<TargetPolicy>,
        F: FnOnce(BBox<T, TargetPolicy>) -> Self::Req,
    >(
        &self,
        ctx: context::Context,
        request_name: &'static str,
        tahini_context_builder: &'static str,
        request_builder: F,
        before_processed: BBox<T, SourcePolicy>,
    ) -> Result<Self::Resp, RpcError>;
}

impl<Req: TahiniType + Clone, Resp: TahiniType> TahiniStub for TahiniChannel<Req, Resp> {
    type Req = Req;
    type Resp = Resp;

    async fn call(
        &self,
        ctx: context::Context,
        request_name: &'static str,
        request: Req,
    ) -> Result<Self::Resp, RpcError> {
        let request = TahiniSafeWrapper(request);
        let response = self.channel.call(ctx, request_name, request).await?;
        Ok(response)
    }

    async fn transform_and_call<
        T,
        TargetPolicy: Policy + serde::Serialize,
        SourcePolicy: PolicyTransformable<TargetPolicy>,
        F: FnOnce(BBox<T, TargetPolicy>) -> Self::Req,
    >(
        &self,
        ctx: context::Context,
        request_name: &'static str,
        tahini_context_builder: &'static str,
        request_builder: F,
        before_processed: BBox<T, SourcePolicy>,
    ) -> Result<Self::Resp, RpcError> {
        let splitted: Vec<_> = tahini_context_builder.split(".").collect();
        assert_eq!(splitted.len(), 2, "Checking if request_name is of length 2");
        let (service, rpc) = (splitted[0], splitted[1]);
        let tahini_context = TahiniContext::new(service, rpc);
        let req = request_builder(before_processed.transform_policy(tahini_context).expect("Couldn't transform policy"));
        self.call(ctx, request_name, req).await
    }
}

pin_project! {
    pub struct TahiniRequestDispatch<Req: TahiniType, Resp: TahiniType, Trans>
        where
            Trans: tarpc::Transport<ClientMessage<TahiniSafeWrapper<Req>>, Response<Resp>>
    {
        #[pin]
        pub(super) dispatch: TarpcRequestDispatch<TahiniSafeWrapper<Req>, Resp, Trans>,
    }

}
//What is the current issue? The dispatch leaks the types.

impl<Req: TahiniType, Resp: TahiniType, Trans> TahiniRequestDispatch<Req, Resp, Trans>
where
    Trans: tarpc::Transport<ClientMessage<TahiniSafeWrapper<Req>>, Response<Resp>>,
{
    pub(crate) fn new(dispatch: TarpcRequestDispatch<TahiniSafeWrapper<Req>, Resp, Trans>) -> Self {
        Self { dispatch }
    }
}

impl<Req, Resp, Trans> Future for TahiniRequestDispatch<Req, Resp, Trans>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: Transport<ClientMessage<TahiniSafeWrapper<Req>>, Response<Resp>> + Send,
{
    type Output = Result<(), ChannelError<Trans::TransportError>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        this.dispatch.poll(cx)
    }
}

pub struct TahiniNewClient<C, D> {
    pub client: C,
    pub dispatch: D,
}
impl<E, C, Req, Resp, Trans> TahiniNewClient<C, TahiniRequestDispatch<Req, Resp, Trans>>
where
    Req: TahiniType + 'static,
    Resp: TahiniType + 'static,
    Trans: Transport<ClientMessage<TahiniSafeWrapper<Req>>, Response<Resp>> + Send + 'static,
    // Trans: TahiniTransport<Req, Resp> + 'static,
    TahiniRequestDispatch<Req, Resp, Trans>: Future<Output = Result<(), E>> + Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn spawn(self) -> C {
        let client = TarpcNewClient {
            client: self.client,
            dispatch: self.dispatch,
        };
        client.spawn()
    }
}

pub fn new<Req, Resp, Trans>(
    config: Config,
    transport: Trans,
) -> TahiniNewClient<TahiniChannel<Req, Resp>, TahiniRequestDispatch<Req, Resp, Trans>>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: Transport<ClientMessage<TahiniSafeWrapper<Req>>, Response<Resp>>,
{
    let client = tarpc::client::new(config, transport);
    TahiniNewClient {
        client: TahiniChannel::new(client.client),
        dispatch: TahiniRequestDispatch::new(client.dispatch),
    }
}
