use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize};
use time::{Date, PrimitiveDateTime, Time};
use crate::rocket::{BBoxDataField, BBoxFormResult, BBoxRequest, BBoxValueField, FromBBoxFormField};

// Forms
macro_rules! impl_form_via_rocket_prim {
    ($($T:ident),+ $(,)?) => ($(
        #[rocket::async_trait]
        impl<'a, 'r> FromBBoxFormField<'a, 'r> for $T {
        #[inline(always)]
        fn from_bbox_value(field: BBoxValueField<'a>, _req: BBoxRequest<'a, 'r>) -> BBoxFormResult<'a, Self> {
            use rocket::form::FromFormField;
            let pfield = rocket::form::ValueField{ name: field.name, value: field.value};
            let pvalue = $T::from_value(pfield)?;
            BBoxFormResult::Ok(pvalue)
        }

        #[inline(always)]
        async fn from_bbox_data(field: BBoxDataField<'a, 'r>, _req: BBoxRequest<'a, 'r>) -> BBoxFormResult<'a, Self> {
            use rocket::form::FromFormField;
            let pfield = rocket::form::DataField {
                name: field.name,
                file_name: field.file_name,
                content_type: field.content_type,
                request: field.request.get_request(),
                data: field.data.get_data(),
            };
            let pvalue = $T::from_data(pfield).await?;
            BBoxFormResult::Ok(pvalue)
        }
    }
    )+)
}

impl_form_via_rocket_prim!(
    f32,
    f64,
    isize,
    i8,
    i16,
    i32,
    i64,
    i128,
    usize,
    u8,
    u16,
    u32,
    u64,
    u128,
    NonZeroIsize,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroUsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    Ipv4Addr,
    IpAddr,
    Ipv6Addr,
    SocketAddrV4,
    SocketAddrV6,
    SocketAddr,
    Date,
    Time,
    PrimitiveDateTime,
    String,
    bool,
);