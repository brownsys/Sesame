use std::marker::PhantomData;

pub struct TahiniContext<'a> {
    pub service: &'a str,
    pub rpc: &'a str,
    priv_marker: PhantomData<()>,
}

impl<'a> TahiniContext<'a> {
    pub(crate) fn new(service: &'a str, rpc: &'a str) -> Self {
        TahiniContext {
            service,
            rpc,
            priv_marker: Default::default(),
        }
    }
}
