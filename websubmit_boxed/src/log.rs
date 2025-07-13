use serde::Serialize;
use serde_remote_converter::remote_converter;
use serde_logger::SerializesFor;

use alohomora::policy::Policy;

#[remote_converter]
#[derive(SerializesFor)]
#[derive(Serialize)]
#[serde(remote = "alohomora::bbox::BBox")]
pub struct BBoxDef<T: Serialize, P: Policy + Serialize> {
    t: T,
    p: P,
}
