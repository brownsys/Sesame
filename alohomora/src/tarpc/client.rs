use crate::tarpc::context::TahiniContext;
use crate::tarpc::enums::TahiniSafeWrapper;
// use crate::tarpc::traits::{
//     deserialize_tahini_type, serialize_tahini_type, TahiniType, TahiniType2,
// };
use crate::tarpc::traits::{Fromable, TahiniTransformInto, TahiniType};
use pin_project_lite::pin_project;
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;
use std::future::Future;
use std::path::Path;
use tahini_attest::client::DynamicAttestationVerifier;
use tahini_attest::types::DynamicAttestationData;
use tarpc::client::Channel as TarpcChannel;
use tarpc::client::NewClient as TarpcNewClient;
use tarpc::client::RequestDispatch as TarpcRequestDispatch;
use tarpc::client::{Config, RpcError};
use tarpc::{context, ChannelError, ClientMessage, Response, Transport};

use super::transport::{KeyEngineState, TahiniTransportTrait};

#[derive(Clone)]
pub struct TahiniChannel<Req: TahiniType, Resp: TahiniType> {
    channel: TarpcChannel<TahiniSafeWrapper<Req>, Resp>,
    engine: KeyEngineState,
}

impl<'a, Req: TahiniType, Resp: TahiniType> TahiniChannel<Req, Resp> {
    pub(crate) fn new(
        channel: TarpcChannel<TahiniSafeWrapper<Req>, Resp>,
        engine: KeyEngineState,
    ) -> Self {
        Self { channel, engine }
    }
}

#[allow(async_fn_in_trait)]
pub trait TahiniStubWrapper {
    type Channel: TahiniStub;

    async fn attest_on_launch(&self);
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

    async fn transform_only_egress<
        'a,
        InputRemoteType: TahiniType,
        InputLocalType: TahiniTransformInto<InputRemoteType> + 'static,
        EgressTransform: FnOnce(InputRemoteType) -> Self::Req,
    >(
        &'a self,
        ctx: context::Context,
        request_name: &'static str,
        tahini_context_builder: &'static str,
        local_input: InputLocalType,
        input_transform: EgressTransform,
    ) -> Result<Self::Resp, RpcError>;

    async fn attest_to_remote<KeyShareWrapClosure: FnOnce(usize) -> Self::Req>(
        &self,
        ctx: context::Context,
        service_name: &'static str,
        wrap_closure: KeyShareWrapClosure,
    ) -> Result<bool, RpcError>;
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
        //Ensure the key has been initialized to avoid subsequent requests from doubling the key
        //exchange
        //FIXME: Even if Sesame compiles with the below line, applications that try to use it
        //can't.
        //I even removed the "once_wait" feature from lib.rs, and it still compiles.
        //So for now, spin lock, unless we can bump the whole toolchain to 1.85 nightly
        // self.engine.key.wait();
        if self.engine.key.get().is_none() {
            println!("Engine is none");
            sleep(Duration::from_secs(2));
        }
        let request = TahiniSafeWrapper(request);
        let response = self.channel.call(ctx, request_name, request).await?;
        Ok(response)
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
    async fn transform_only_egress<
        'a,
        InputRemoteType: TahiniType,
        InputLocalType: TahiniTransformInto<InputRemoteType> + 'static,
        EgressTransform: FnOnce(InputRemoteType) -> Self::Req,
    >(
        &'a self,
        ctx: context::Context,
        request_name: &'static str,
        tahini_context_builder: &'static str,
        local_input: InputLocalType,
        input_transform: EgressTransform,
    ) -> Result<Self::Resp, RpcError> {
        let splitted: Vec<_> = tahini_context_builder.split(".").collect();
        assert_eq!(splitted.len(), 2, "Checking if request_name is of length 2");
        let (service, rpc) = (splitted[0], splitted[1]);
        let tahini_context = TahiniContext::new(service, rpc);
        let remote_input: InputRemoteType = local_input
            .transform_into(&tahini_context)
            .expect("Policy didn't allow to transform the input for this RPC");
        let wrapped = input_transform(remote_input);
        self.call(ctx, request_name, wrapped).await
    }

    async fn attest_to_remote<KeyShareWrapClosure: FnOnce(usize) -> Self::Req>(
        &self,
        ctx: context::Context,
        service_name: &'static str,
        wrap_closure: KeyShareWrapClosure,
    ) -> Result<bool, RpcError> {
        let client_attest_config = Path::new("./client_attestation_config.toml");
        let attest_verifier = DynamicAttestationVerifier::from_config(client_attest_config)
            .expect("Couldn't load config");
        match attest_verifier
            .verify_binary(service_name.to_string().into())
            .await
        {
            Ok((client_id, aes_key)) => {
                let request_name = "tahini_attest";
                let req = wrap_closure(client_id.into());
                let res = self
                    .channel
                    .call(ctx, request_name, TahiniSafeWrapper(req))
                    .await?;
                self.engine
                    .key
                    .set(aes_key)
                    .expect("Key is already initialized for this session");
                self.engine
                    .passthrough
                    .set(true)
                    .expect("Attestation should be running only once");
                Ok(true)
            }
            Err(e) => {
                println!("Got error {:?}", e);
                Err(RpcError::Shutdown)
            }
        }
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
    C: TahiniStubWrapper,
{
    pub async fn spawn(self) -> C {
        let client = TarpcNewClient {
            client: self.client,
            dispatch: self.dispatch,
        };
        let client = client.spawn();
        client.attest_on_launch().await;
        client
    }
}

//TODO(douk): Generated client-side code basically extracts the channel from within
//this new client, and then recomponses itself into a TahiniNewClient.
//Might be a more elegant way to do that with a nice API with generics over a trait.
pub fn new<Req, Resp, Trans>(
    config: Config,
    transport: Trans,
) -> TahiniNewClient<
    TahiniChannel<Req, Resp>,
    TahiniRequestDispatch<Req, Resp, Trans::InnerChannelType>,
>
where
    Req: TahiniType,
    Resp: TahiniType,
    Trans: TahiniTransportTrait<ClientMessage<TahiniSafeWrapper<Req>>, Response<Resp>>,
{
    let engine = transport.get_engine();
    let client = tarpc::client::new(config, transport.get_inner());
    TahiniNewClient {
        client: TahiniChannel::new(client.client, engine),
        dispatch: TahiniRequestDispatch::new(client.dispatch),
    }
}
