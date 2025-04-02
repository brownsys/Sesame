use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;
use tarpc::client::{
    Channel, RpcError};
use tarpc::client::stub::Stub;

// Creation of this must be checked for purity.
#[derive(Clone)]
pub struct TahiniPrivacyPureRegion<Req, Resp> {
    channel: Channel<Req, Resp>
}


///Provide a service, create the PPR:
///Create an enum that implements a Tahini trait
///This Tahini trait defines a mapping between enum variants and method name
///Creates the appropriate request (hard part?), calls "channel.call"
///Return a PCon
///
///In application, we therefore have:   

///Does the developer provide 
///Tahini Pure Region
///What is the intended objective here?
///
///The developer provides a "functor" 
///Where does the netcode lie? 
///Does the developer supply the closure?
///Or is the netcode implemented here?
///As such, the developer would only supply the channel
///
///The generated code is responsible for constructing the 
///We can lock in this API only for Tahini services.
///
///How does one do that?
///Place the channel into a wrapper type, and put a trait bound only for the Tahini-defined
///wrappers.
impl<Req, Resp> TahiniPrivacyPureRegion<Req, Resp> {
    pub const fn new(channel: Channel<Req, Resp>) -> Self {
        TahiniPrivacyPureRegion { channel }
    }
    pub fn get_channel(self) -> Channel<Req, Resp> {
        self.channel
    }
}


pub async fn execute_tahini<Req, Resp>(
    functor: TahiniPrivacyPureRegion<Req, Resp>,
    context: tarpc::context::Context,
    endpoint: &'static str,
    data: BBox<Req, AnyPolicy>) -> Result<BBox<Resp, AnyPolicy>, RpcError> {
    let (t, p) = data.consume();
    let channel = functor.get_channel();
    let resp = Stub::call(&channel,
        context,
        endpoint,
        t
        ).await?;
    Ok(BBox::new(resp, p))
}



//On est pas loin de l'idee, il faut juste savoir quelles pieces vont ou
//Genre quel est le role de la PPR?
//Dans quelle limite est-ce qu'elle bind aux wrapper enums? Comment on recupere
//la structure de base? Actuellement on est en full fold out.
//J'imagine que c'est quelque chose qui a ete pris en consideration dans Sesame?
//
//La PPR retourne juste un PCon? 
//
