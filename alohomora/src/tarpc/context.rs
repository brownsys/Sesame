use std::{
    collections::{hash_map, HashMap},
    marker::PhantomData,
};

use serde::{Deserialize, Serialize};

use super::{TahiniEnum, TahiniType};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Clone)]
pub struct TahiniContext {
    pub service: String,
    pub rpc: String,
    priv_marker: PhantomData<()>,
}

impl TahiniContext {
    pub(crate) fn new<'a>(service: &'a str, rpc: &'a str) -> Self {
        TahiniContext {
            service: service.to_string(),
            rpc: rpc.to_string(),
            priv_marker: Default::default(),
        }
    }
}

impl TahiniType for TahiniContext {
    fn to_tahini_enum(&self) -> super::TahiniEnum {
        let mut hash_map = HashMap::new();
        hash_map.insert(
            "service",
            TahiniEnum::Value(Box::new(self.service.to_string())),
        );
        hash_map.insert("rpc", TahiniEnum::Value(Box::new(self.rpc.to_string())));
        super::TahiniEnum::Struct("TahiniContext", hash_map)
    }
}
