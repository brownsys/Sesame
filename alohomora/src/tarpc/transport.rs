// TODO(babman): we can probably do some work with this to ensure transport/codec are safe, maybe
//               using scrutinizer.
use aws_lc_rs::aead::{Aad, Nonce};
pub use aws_lc_rs::aead::RandomizedNonceKey as TahiniChannelKey;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, OnceLock};
use tarpc::serde_transport::Transport;
use tarpc::Transport as TransportTrait;
use tokio_serde::{Deserializer, Serializer};
use tokio_util::bytes::Bytes;
use tokio_util::{
    bytes::BytesMut,
    codec::{Framed, LengthDelimitedCodec},
};

#[derive(Clone)]
pub(crate) struct KeyEngineState {
    pub key: Arc<OnceLock<TahiniChannelKey>>,
    pub passthrough: Arc<OnceLock<bool>>,
    // pub session_id: Arc<OnceLock<u64>>
}


impl KeyEngineState {
    pub(crate) fn new() -> Self {
        Self {
            key: Arc::new(OnceLock::new()),
            passthrough: Arc::new(OnceLock::new()),
        }
    }
}

pin_project! {
pub struct TahiniTransport<S, Item, SinkItem, Codec> {
        #[pin]
        pub(crate) transport:
            Transport<S, Item, SinkItem, TahiniEncryptionLayer<Item, SinkItem, Codec>>,
        pub key_engine: KeyEngineState,
    }
}

pub trait TahiniTransportTrait<SinkItem, Item> {
    type InnerChannelType: TransportTrait<SinkItem, Item>;

    fn get_engine(&self) -> KeyEngineState;

    fn get_inner(self) -> Self::InnerChannelType;
}

impl<S, Item, SinkItem, Codec> TahiniTransportTrait<SinkItem, Item>
    for TahiniTransport<S, Item, SinkItem, Codec>
where
    S: rocket::tokio::io::AsyncWrite + rocket::tokio::io::AsyncRead,
    Codec: Serializer<SinkItem> + Deserializer<Item>,
    Item: for<'de> Deserialize<'de> + Debug,
    SinkItem: Serialize,
{
    type InnerChannelType =
        Transport<S, Item, SinkItem, TahiniEncryptionLayer<Item, SinkItem, Codec>>;

    fn get_engine(&self) -> KeyEngineState {
        self.key_engine.clone()
    }

    fn get_inner(self) -> Self::InnerChannelType {
        self.transport
    }
}

pub fn new_tahini_transport<'a, S, Item, SinkItem, Codec>(
    framed_io: Framed<S, LengthDelimitedCodec>,
    codec: Codec,
) -> TahiniTransport<S, Item, SinkItem, Codec>
where
    S: rocket::tokio::io::AsyncWrite + rocket::tokio::io::AsyncRead,
    Item: for<'de> Deserialize<'de> + Debug,
    SinkItem: Serialize,
    Codec: Serializer<SinkItem> + Deserializer<Item>, // + Serializer<SerializedCipher>
                                                      // + Deserializer<SerializedCipher>,
{
    // let oncelock: Arc<OnceLock<KeyEngineState>> = Arc::new(OnceLock::new());
    let engine = KeyEngineState::new();
    let key_engine = engine.clone();
    let encryption_layer: TahiniEncryptionLayer<Item, SinkItem, Codec> =
        TahiniEncryptionLayer::new(codec, engine);

    TahiniTransport {
        transport: tarpc::serde_transport::new(framed_io, encryption_layer),
        key_engine,
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedCipher {
    bytes: Vec<u8>,
    nonce: [u8; 12],
}

pin_project! {
    pub struct TahiniEncryptionLayer<Item, SinkItem, C>
    {
        #[pin]
        inner_channel: C,
        item: PhantomData<Item>,
        sink: PhantomData<SinkItem>,
        key_state: KeyEngineState,
    }
}

impl<C, Item, SinkItem> TahiniEncryptionLayer<Item, SinkItem, C>
where
    C: Serializer<SinkItem> + Deserializer<Item>,
{
    pub(crate) fn new(codec: C, engine: KeyEngineState) -> Self {
        Self {
            inner_channel: codec,
            item: PhantomData,
            sink: PhantomData,
            key_state: engine,
        }
    }
}

pub enum TahiniChannelLayerError {
    InnerError(std::io::Error),
    WrapperError(std::io::Error),
}

impl TahiniChannelLayerError {
    pub fn wrapper_err(s: &str) -> Self {
        TahiniChannelLayerError::WrapperError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            s.to_string(),
        ))
    }
}

impl From<TahiniChannelLayerError> for std::io::Error {
    fn from(value: TahiniChannelLayerError) -> Self {
        match value {
            TahiniChannelLayerError::InnerError(inner) => inner,
            TahiniChannelLayerError::WrapperError(wrapper) => {
                std::io::Error::new(std::io::ErrorKind::InvalidData, wrapper)
            }
        }
    }
}

impl<C, Item, SinkItem> Serializer<SinkItem> for TahiniEncryptionLayer<Item, SinkItem, C>
where
    C: Serializer<SinkItem>,
    SinkItem: Serialize,
{
    type Error = TahiniChannelLayerError;

    fn serialize(
        self: std::pin::Pin<&mut Self>,
        item: &SinkItem,
    ) -> Result<tokio_util::bytes::Bytes, Self::Error> {
        let key_engine = self.key_state.clone();
        let key_opt = key_engine.key.get();
        match key_opt {
            None => C::serialize(self.project().inner_channel, item)
                .map_err(|_| TahiniChannelLayerError::wrapper_err("Keyshare ser error")),
            Some(key) => match key_engine.passthrough.get() {
                None => {
                    let _ = key_engine.passthrough.set(true);
                    return C::serialize(self.project().inner_channel, item)
                        .map_err(|_| TahiniChannelLayerError::wrapper_err("Keyshare ser error"));
                }
                Some(_) => {
                    let mut inner_codec = self.project().inner_channel;
                    let res = C::serialize(inner_codec.as_mut(), item).map_err(|_| {
                        TahiniChannelLayerError::wrapper_err("Payload serialization error")
                    })?;
                    let mut cipher_buf = Vec::from(res);
                    println!("Key is : {:?}", key);
                    let nonce = key
                        .seal_in_place_append_tag(Aad::empty(), &mut cipher_buf)
                        .map_err(|_| TahiniChannelLayerError::wrapper_err("Encryption error"))?;
                    let nonce = nonce.as_ref();

                    let encrypted_struct = SerializedCipher {
                        bytes: cipher_buf,
                        nonce: *nonce,
                    };
                    let encrypted_serialized =
                        serde_json::to_vec(&encrypted_struct).map_err(|_| {
                            TahiniChannelLayerError::wrapper_err("Ciphertext serialization error")
                        })?;
                    Ok(Bytes::from(encrypted_serialized))
                }
            },
        }
    }
}

impl<'de, C, Item, SinkItem> Deserializer<Item> for TahiniEncryptionLayer<Item, SinkItem, C>
where
    C: Deserializer<Item>,
    Item: Deserialize<'de> + Debug,
{
    type Error = TahiniChannelLayerError;
    fn deserialize(
        self: std::pin::Pin<&mut Self>,
        src: &tokio_util::bytes::BytesMut,
    ) -> Result<Item, Self::Error> {
        let arc_cloned = self.key_state.key.clone();
        let key_opt = arc_cloned.get();
        match key_opt {
            None => {
                println!("Assessing that we are without a key");
                let a = C::deserialize(self.project().inner_channel, src).map_err(|_| {
                    TahiniChannelLayerError::wrapper_err("Remote Keyshare deserialization error");
                });
                match a {
                    Ok(_) => println!("We successfully deserialize key share"),
                    Err(_) => println!("We fail at deserializing the keyshare"),
                };
                a.map_err(|_| {
                    TahiniChannelLayerError::wrapper_err(
                        "Failed at deserializing the remote key share",
                    )
                })
            }
            Some(key) => {
                println!("Assessing we have a key for decryption!");
                let ciphertext: SerializedCipher = serde_json::from_slice(src).map_err(|_| {
                    TahiniChannelLayerError::wrapper_err("Ciphertext deserialization error")
                })?;
                println!("Successfully got the ciphertext struct");
                let nonce = Nonce::from(&ciphertext.nonce);
                let mut cipher = ciphertext.bytes;
                println!("Key is : {:?}", key);
                let plaintext_slice: &[u8] = key
                    .open_in_place(nonce, Aad::empty(), &mut cipher)
                    .map_err(|_| TahiniChannelLayerError::wrapper_err("Decryption error"))?;
                println!("Successfully deciphered");
                let plaintext_bytes = BytesMut::from(plaintext_slice);
                let res = C::deserialize(self.project().inner_channel, &plaintext_bytes).map_err(|_| {
                    TahiniChannelLayerError::wrapper_err("Plaintext payload deserialization error")
                })?;
                println!("We got struct {:?}", res);
                Ok(res)
            }
        }
    }
}
