use crate::tarpc::context::TahiniContext;
use crate::tarpc::enums::TahiniSafeWrapper;
// use crate::tarpc::traits::{
//     deserialize_tahini_type, serialize_tahini_type, TahiniType, TahiniType2,
// };
use crate::bbox::BBox;
use crate::policy::{Policy, PolicyInto, TahiniPolicy};
use crate::tarpc::traits::{Fromable, TahiniTransformFrom, TahiniTransformInto, TahiniType};
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

    async fn transform_with_fromable<
        'a,
        InputRemoteType: TahiniType,
        InputLocalType: TahiniTransformInto<InputRemoteType> + 'static,
        EgressTransform: FnOnce(InputRemoteType) -> Self::Req,
        OutputRemoteType: TahiniType,
        IngressTransform: FnOnce(Self::Resp) -> Fromable<OutputRemoteType>,
    >(
        &'a self,
        ctx: context::Context,
        request_name: &'static str,
        tahini_context_builder: &'static str,
        local_input: InputLocalType,
        input_transform: EgressTransform,
        output_transform: IngressTransform,
    ) -> Result<Fromable<OutputRemoteType>, RpcError>;

    async fn transform_both_ways<
        'a,
        InputRemoteType: TahiniType,
        InputLocalType: TahiniTransformInto<InputRemoteType> + 'static,
        EgressTransform: FnOnce(InputRemoteType) -> Self::Req,
        OutputRemoteType: TahiniType,
        OutputLocalType: TahiniTransformFrom<OutputRemoteType> + 'static,
        IngressTransform: FnOnce(Self::Resp) -> OutputRemoteType,
    >(
        &'a self,
        ctx: context::Context,
        request_name: &'static str,
        tahini_context_builder: &'static str,
        local_input: InputLocalType,
        input_transform: EgressTransform,
        output_transform: IngressTransform,
    ) -> Result<OutputLocalType, RpcError>;

    async fn transform_and_call<
        'a,
        T: 'static,
        TargetPolicy: Policy + serde::Serialize + 'static,
        SourcePolicy: PolicyInto<TargetPolicy> + 'static,
        F: FnOnce(BBox<T, TargetPolicy>) -> Self::Req,
    >(
        &'a self,
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
        'a,
        T: 'static,
        TargetPolicy: Policy + serde::Serialize + 'static,
        SourcePolicy: PolicyInto<TargetPolicy> + 'static,
        F: FnOnce(BBox<T, TargetPolicy>) -> Self::Req,
    >(
        &'a self,
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
        let req = request_builder(
            before_processed
                .transform_into(&tahini_context)
                .expect("Couldn't transform policy"),
        );
        self.call(ctx, request_name, req).await
    }

    async fn transform_both_ways<
        'a,
        InputRemoteType: TahiniType,
        InputLocalType: TahiniTransformInto<InputRemoteType> + 'static,
        EgressTransform: FnOnce(InputRemoteType) -> Self::Req,
        OutputRemoteType: TahiniType,
        OutputLocalType: TahiniTransformFrom<OutputRemoteType> + 'static,
        IngressTransform: FnOnce(Self::Resp) -> OutputRemoteType,
    >(
        &'a self,
        ctx: context::Context,
        request_name: &'static str,
        tahini_context_builder: &'static str,
        local_input: InputLocalType,
        input_transform: EgressTransform,
        output_transform: IngressTransform,
    ) -> Result<OutputLocalType, RpcError> {
        let splitted: Vec<_> = tahini_context_builder.split(".").collect();
        assert_eq!(splitted.len(), 2, "Checking if request_name is of length 2");
        let (service, rpc) = (splitted[0], splitted[1]);
        let tahini_context = TahiniContext::new(service, rpc);
        let remote_input: InputRemoteType = local_input
            .transform_into(&tahini_context)
            .expect("Policy didn't allow to transform the input for this RPC");
        let wrapped = input_transform(remote_input);
        let resp = self.call(ctx, request_name, wrapped).await?;
        let unwrapped: OutputRemoteType = output_transform(resp);
        Ok(OutputLocalType::transform_from(unwrapped, &tahini_context)
            .expect("Policy didn't allow to parse remote type into local type"))
    }
    async fn transform_with_fromable<
        'a,
        InputRemoteType: TahiniType,
        InputLocalType: TahiniTransformInto<InputRemoteType> + 'static,
        EgressTransform: FnOnce(InputRemoteType) -> Self::Req,
        OutputRemoteType: TahiniType,
        IngressTransform: FnOnce(Self::Resp) -> Fromable<OutputRemoteType>,
    >(
        &'a self,
        ctx: context::Context,
        request_name: &'static str,
        tahini_context_builder: &'static str,
        local_input: InputLocalType,
        input_transform: EgressTransform,
        output_transform: IngressTransform,
    ) -> Result<Fromable<OutputRemoteType>, RpcError> {
        let splitted: Vec<_> = tahini_context_builder.split(".").collect();
        assert_eq!(splitted.len(), 2, "Checking if request_name is of length 2");
        let (service, rpc) = (splitted[0], splitted[1]);
        let tahini_context = TahiniContext::new(service, rpc);
        let remote_input: InputRemoteType = local_input
            .transform_into(&tahini_context)
            .expect("Policy didn't allow to transform the input for this RPC");
        let wrapped = input_transform(remote_input);
        let resp = self.call(ctx, request_name, wrapped).await?;
        let mut unwrapped: Fromable<OutputRemoteType> = output_transform(resp);
        unwrapped.add_context(tahini_context);
        Ok(unwrapped)
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
