extern crate indexmap;

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::option::Option;
use std::result::Result;
use std::sync::Arc;

use indexmap::IndexMap;

use rocket::http::uncased::AsUncased;
use rocket::Either;

use crate::bbox::BBox;
use crate::policy::FrontendPolicy;
use crate::rocket::data::BBoxData;
use crate::rocket::request::BBoxRequest;

pub type BBoxFormResult<'v, T> = Result<T, rocket::form::Errors<'v>>;

// BBoxForm is just a wrapper around types that satisfy FromBBoxForm.
pub struct BBoxForm<T>(pub(super) T);
impl<T> BBoxForm<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
impl<T> From<T> for BBoxForm<T> {
    #[inline]
    fn from(val: T) -> Self {
        Self(val)
    }
}
impl<T> Deref for BBoxForm<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for BBoxForm<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// For url encoded bodies.
pub struct BBoxValueField<'a> {
    pub name: rocket::form::name::NameView<'a>,
    pub(crate) value: &'a str, // Should be boxed when exposed.
}
impl<'a> BBoxValueField<'a> {
    pub fn shift(mut self) -> Self {
        self.name.shift();
        self
    }
    pub fn unexpected(&self) -> rocket::form::Error<'a> {
        rocket::form::Error::from(rocket::form::error::ErrorKind::Unexpected)
            .with_name(self.name.source())
            .with_value("<boxed>")
            .with_entity(rocket::form::error::Entity::ValueField)
    }
    pub fn missing(&self) -> rocket::form::Error<'a> {
        rocket::form::Error::from(rocket::form::error::ErrorKind::Missing)
            .with_name(self.name.source())
            .with_value("<boxed>")
            .with_entity(rocket::form::error::Entity::ValueField)
    }
    pub fn from_value(value: &'a str) -> Self {
        BBoxValueField {
            name: rocket::form::name::NameView::new(""),
            value
        }
    }
    pub(super) fn from_rocket(field: rocket::form::ValueField<'a>) -> Self {
        BBoxValueField {
            name: field.name,
            value: field.value,
        }
    }
    pub(super) fn to_rocket(self) -> rocket::form::ValueField<'a> {
        rocket::form::ValueField {
            name: self.name,
            value: self.value,
        }
    }
}

pub struct BBoxDataField<'a, 'r> {
    pub name: rocket::form::name::NameView<'a>,
    pub file_name: Option<&'a rocket::fs::FileName>,
    pub content_type: rocket::http::ContentType,
    pub request: BBoxRequest<'a, 'r>,
    pub data: BBoxData<'a>,
}
impl<'a, 'r> BBoxDataField<'a, 'r> {
    pub fn shift(mut self) -> Self {
        self.name.shift();
        self
    }
    pub fn unexpected(&self) -> rocket::form::Error<'a> {
        rocket::form::Error::from(rocket::form::error::ErrorKind::Unexpected)
            .with_name(self.name.source())
            .with_entity(rocket::form::error::Entity::DataField)
    }

    pub(super) fn from_rocket(field: rocket::form::DataField<'a, 'r>) -> Self {
        BBoxDataField {
            name: field.name,
            file_name: field.file_name,
            content_type: field.content_type,
            request: BBoxRequest::new(field.request),
            data: BBoxData::new(field.data)
        }
    }
    pub(super) fn to_rocket(self) -> rocket::form::DataField<'a, 'r> {
        rocket::form::DataField {
            name: self.name,
            file_name: self.file_name,
            content_type: self.content_type,
            request: self.request.get_request(),
            data: self.data.get_data(),
        }
    }
}

// Our version of FromFormField, this implies FromBBoxForm.
#[rocket::async_trait]
pub trait FromBBoxFormField<'a, 'r>: Send + Sized {
    fn from_bbox_value(field: BBoxValueField<'a>, _req: BBoxRequest<'a, 'r>) -> BBoxFormResult<'a, Self> {
        Err(field.unexpected())?
    }
    async fn from_bbox_data(field: BBoxDataField<'a, 'r>, _req: BBoxRequest<'a, 'r>) -> BBoxFormResult<'a, Self> {
        Err(field.unexpected())?
    }
    fn default() -> Option<Self> {
        None
    }
}

// Our own FromBBoxForm trait, mirror's rockets' FromForm trait.
// Do not use directly, derive instead.
#[rocket::async_trait]
pub trait FromBBoxForm<'a, 'r>: Send + Sized {
    type BBoxContext: Send;

    // Required methods
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext;
    fn bbox_push_value(
        ctxt: &mut Self::BBoxContext,
        field: BBoxValueField<'a>,
        request: BBoxRequest<'a, 'r>,
    );
    async fn bbox_push_data(
        ctxt: &mut Self::BBoxContext,
        field: BBoxDataField<'a, 'r>,
        request: BBoxRequest<'a, 'r>,
    );
    fn bbox_finalize(ctxt: Self::BBoxContext) -> BBoxFormResult<'a, Self>;

    // Provided methods
    fn bbox_push_error(_ctxt: &mut Self::BBoxContext, _error: rocket::form::Error<'a>) {}
    fn bbox_default(opts: rocket::form::Options) -> Option<Self> {
        Self::bbox_finalize(Self::bbox_init(opts)).ok()
    }
}

// Auto implement FromBBoxForm for everything that implements FromBBoxFormField.
pub struct FromBBoxFieldContext<'a, 'r, T: FromBBoxFormField<'a, 'r>> {
    field_name: Option<rocket::form::name::NameView<'a>>,
    opts: rocket::form::Options,
    value: Option<BBoxFormResult<'a, T>>,
    pushes: usize,
    _phantom: PhantomData<&'r ()>,
}
impl<'a, 'r, T: FromBBoxFormField<'a, 'r>> FromBBoxFieldContext<'a, 'r, T> {
    fn should_push(&mut self) -> bool {
        self.pushes += 1;
        self.value.is_none()
    }

    fn push(&mut self, name: rocket::form::name::NameView<'a>, result: BBoxFormResult<'a, T>) {
        fn is_unexpected(e: &rocket::form::Errors<'_>) -> bool {
            matches!(
                e.last().map(|e| &e.kind),
                Some(rocket::form::error::ErrorKind::Unexpected)
            )
        }

        self.field_name = Some(name);
        match result {
            Err(e) if !self.opts.strict && is_unexpected(&e) => { /* ok */ }
            result => self.value = Some(result),
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r, T: FromBBoxFormField<'a, 'r>> FromBBoxForm<'a, 'r> for T {
    type BBoxContext = FromBBoxFieldContext<'a, 'r, T>;

    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        FromBBoxFieldContext {
            opts,
            field_name: None,
            value: None,
            pushes: 0,
            _phantom: PhantomData,
        }
    }

    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>,) {
        if ctxt.should_push() {
            ctxt.push(field.name, Self::from_bbox_value(field, request))
        }
    }

    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>,) {
        if ctxt.should_push() {
            ctxt.push(field.name, Self::from_bbox_data(field, request).await);
        }
    }

    fn bbox_finalize(ctxt: Self::BBoxContext) -> BBoxFormResult<'a, Self> {
        let mut errors = match ctxt.value {
            Some(Ok(val)) if !ctxt.opts.strict || ctxt.pushes <= 1 => return Ok(val),
            Some(Ok(_)) => rocket::form::Errors::from(rocket::form::error::ErrorKind::Duplicate),
            Some(Err(errors)) => errors,
            None if !ctxt.opts.strict => match <T as FromBBoxFormField>::default() {
                Some(default) => return Ok(default),
                None => rocket::form::Errors::from(rocket::form::error::ErrorKind::Missing),
            },
            None => rocket::form::Errors::from(rocket::form::error::ErrorKind::Missing),
        };
        if let Some(name) = ctxt.field_name {
            errors.set_name(name);
            errors.set_value("<boxed>")
        }
        Err(errors)
    }
}

// Implement FromBBoxFormField for select types whose implementation of
// FromFormField is defined by rocket and is safe.
macro_rules! impl_form_via_rocket {
    ($($T:ident),+ $(,)?) => ($(
        #[rocket::async_trait]
        impl<'a, 'r, P: FrontendPolicy> FromBBoxFormField<'a, 'r> for BBox<$T, P> {
            #[inline(always)]
            fn from_bbox_value(field: BBoxValueField<'a>, req: BBoxRequest<'a, 'r>) -> BBoxFormResult<'a, Self> {
                use rocket::form::FromFormField;
                let pfield = rocket::form::ValueField{ name: field.name, value: field.value};
                let pvalue = $T::from_value(pfield)?;
                BBoxFormResult::Ok(BBox::new(pvalue, P::from_request(req.get_request())))
            }

            #[inline(always)]
            async fn from_bbox_data(field: BBoxDataField<'a, 'r>, req: BBoxRequest<'a, 'r>) -> BBoxFormResult<'a, Self> {
                use rocket::form::FromFormField;
                let pfield = rocket::form::DataField {
                    name: field.name,
                    file_name: field.file_name,
                    content_type: field.content_type,
                    request: field.request.get_request(),
                    data: field.data.get_data(),
                };
                let pvalue = $T::from_data(pfield).await?;
                BBoxFormResult::Ok(BBox::new(pvalue, P::from_request(req.get_request())))
            }
        }
    )+)
}

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use time::{Date, PrimitiveDateTime, Time};
impl_form_via_rocket!(
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
    bool
);

// Implement FromBBoxForm for Vec<T: FromBBoxForm>.
pub struct VecContext<'a, 'r, T: FromBBoxForm<'a, 'r>> {
    opts: rocket::form::Options,
    last_key: Option<&'a rocket::form::name::Key>,
    current: Option<T::BBoxContext>,
    errors: rocket::form::Errors<'a>,
    items: Vec<T>,
    _phantom: PhantomData<&'r ()>,
}
impl<'a, 'r, T: FromBBoxForm<'a, 'r>> VecContext<'a, 'r, T> {
    fn new(opts: rocket::form::Options) -> Self {
        VecContext {
            opts,
            last_key: None,
            current: None,
            items: vec![],
            errors: rocket::form::Errors::new(),
            _phantom: PhantomData,
        }
    }
    fn shift(&mut self) {
        if let Some(current) = self.current.take() {
            match T::bbox_finalize(current) {
                Ok(v) => self.items.push(v),
                Err(e) => self.errors.extend(e),
            }
        }
    }
    fn context(&mut self, name: &rocket::form::name::NameView<'a>) -> &mut T::BBoxContext {
        let this_key = name.key();
        let keys_match = match (self.last_key, this_key) {
            (Some(k1), Some(k2)) => k1 == k2,
            _ => false,
        };

        if !keys_match {
            self.shift();
            self.current = Some(T::bbox_init(self.opts));
        }

        self.last_key = name.key();
        self.current
            .as_mut()
            .expect("must have current if last == index")
    }
}

#[rocket::async_trait]
impl<'a, 'r: 'a, T: FromBBoxForm<'a, 'r> + 'r> FromBBoxForm<'a, 'r> for Vec<T> {
    type BBoxContext = VecContext<'a, 'r, T>;

    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        VecContext::new(opts)
    }

    fn bbox_push_value(this: &mut Self::BBoxContext, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>) {
        T::bbox_push_value(this.context(&field.name), field.shift(), request);
    }

    async fn bbox_push_data(this: &mut Self::BBoxContext, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>) {
        T::bbox_push_data(this.context(&field.name), field.shift(), request).await
    }

    fn bbox_finalize(mut this: Self::BBoxContext) -> BBoxFormResult<'a, Self> {
        this.shift();
        if !this.errors.is_empty() {
            Err(this.errors)
        } else if this.opts.strict && this.items.is_empty() {
            Err(rocket::form::Errors::from(
                rocket::form::error::ErrorKind::Missing,
            ))
        } else {
            Ok(this.items)
        }
    }
}

// Implement FromBBoxForm for HashMap and BTreeMap (provided keys and values
// also implement FromBBoxForm).
pub struct MapContext<'a, 'r, K, V>
where
    K: rocket::form::FromForm<'a>,
    V: FromBBoxForm<'a, 'r>,
{
    opts: rocket::form::Options,
    table: IndexMap<&'a str, usize>,
    entries: Vec<(K::Context, V::BBoxContext)>,
    metadata: Vec<rocket::form::name::NameView<'a>>,
    errors: rocket::form::Errors<'a>,
    _phantom: PhantomData<&'r ()>,
}
impl<'a, 'r, K, V> MapContext<'a, 'r, K, V>
where
    K: rocket::form::FromForm<'a>,
    V: FromBBoxForm<'a, 'r>,
{
    fn new(opts: rocket::form::Options) -> Self {
        MapContext {
            opts,
            table: IndexMap::new(),
            entries: vec![],
            metadata: vec![],
            errors: rocket::form::Errors::new(),
            _phantom: PhantomData,
        }
    }
    fn ctxt(
        &mut self,
        key: &'a str,
        name: rocket::form::name::NameView<'a>,
    ) -> &mut (K::Context, V::BBoxContext) {
        match self.table.get(key) {
            Some(i) => &mut self.entries[*i],
            None => {
                let i = self.entries.len();
                self.table.insert(key, i);
                self.entries.push((K::init(self.opts), V::bbox_init(self.opts)));
                self.metadata.push(name);
                &mut self.entries[i]
            }
        }
    }
    fn push(
        &mut self,
        name: rocket::form::name::NameView<'a>,
    ) -> Option<Either<&mut K::Context, &mut V::BBoxContext>> {
        let index_pair = name
            .key()
            .map(|k| k.indices())
            .map(|mut i| (i.next(), i.next()))
            .unwrap_or_default();

        match index_pair {
            (Some(key), None) => {
                let is_new_key = !self.table.contains_key(key);
                let (key_ctxt, val_ctxt) = self.ctxt(key, name);
                if is_new_key {
                    K::push_value(key_ctxt, rocket::form::ValueField::from_value(key));
                }

                return Some(Either::Right(val_ctxt));
            }
            (Some(kind), Some(key)) => {
                if kind.as_uncased().starts_with("k") {
                    return Some(Either::Left(&mut self.ctxt(key, name).0));
                } else if kind.as_uncased().starts_with("v") {
                    return Some(Either::Right(&mut self.ctxt(key, name).1));
                } else {
                    let error =
                        rocket::form::Error::from(&[Cow::Borrowed("k"), Cow::Borrowed("v")])
                            .with_entity(rocket::form::error::Entity::Index(0))
                            .with_name(name);

                    self.errors.push(error);
                }
            }
            _ => {
                let error = rocket::form::Error::from(rocket::form::error::ErrorKind::Missing)
                    .with_entity(rocket::form::error::Entity::Key)
                    .with_name(name);
                self.errors.push(error);
            }
        };

        None
    }
    fn push_value(&mut self, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>) {
        match self.push(field.name) {
            Some(Either::Left(ctxt)) => K::push_value(ctxt, field.shift().to_rocket()),
            Some(Either::Right(ctxt)) => V::bbox_push_value(ctxt, field.shift(), request),
            _ => {}
        }
    }
    async fn push_data(&mut self, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>) {
        match self.push(field.name) {
            Some(Either::Left(ctxt)) => K::push_data(ctxt, field.shift().to_rocket()).await,
            Some(Either::Right(ctxt)) => V::bbox_push_data(ctxt, field.shift(), request).await,
            _ => {}
        }
    }
    fn finalize<T: std::iter::FromIterator<(K, V)>>(mut self) -> BBoxFormResult<'a, T> {
        let errors = &mut self.errors;
        let map: T = self
            .entries
            .into_iter()
            .zip(self.metadata.iter())
            .zip(self.table.keys())
            .filter_map(|(((k_ctxt, v_ctxt), name), idx)| {
                let key = K::finalize(k_ctxt)
                    .map_err(|e| errors.extend(e.with_name((name.parent(), *idx))))
                    .ok();
                let val = V::bbox_finalize(v_ctxt)
                    .map_err(|e| errors.extend(e.with_name((name.parent(), *idx))))
                    .ok();
                Some((key?, val?))
            })
            .collect();
        if !errors.is_empty() {
            Err(self.errors)
        } else if self.opts.strict && self.table.is_empty() {
            Err(rocket::form::Errors::from(
                rocket::form::error::ErrorKind::Missing,
            ))
        } else {
            Ok(map)
        }
    }
}
#[rocket::async_trait]
impl<'a, 'r: 'a, K, V> FromBBoxForm<'a, 'r> for HashMap<K, V>
where
    K: rocket::form::FromForm<'a> + Eq + Hash,
    V: FromBBoxForm<'a, 'r>,
{
    type BBoxContext = MapContext<'a, 'r, K, V>;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        MapContext::new(opts)
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>) {
        ctxt.push_value(field, request);
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>) {
        ctxt.push_data(field, request).await;
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'a, Self> {
        this.finalize()
    }
}
#[rocket::async_trait]
impl<'a, 'r: 'a, K, V> FromBBoxForm<'a, 'r> for BTreeMap<K, V>
where
    K: rocket::form::FromForm<'a> + Ord,
    V: FromBBoxForm<'a, 'r>,
{
    type BBoxContext = MapContext<'a, 'r, K, V>;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        MapContext::new(opts)
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>) {
        ctxt.push_value(field, request);
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>) {
        ctxt.push_data(field, request).await;
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'a, Self> {
        this.finalize()
    }
}

// Implement FromBBoxForm for Option<T> (provided T also implement FromBBoxForm).
#[rocket::async_trait]
impl<'a, 'r, T: FromBBoxForm<'a, 'r>> FromBBoxForm<'a, 'r> for Option<T> {
    type BBoxContext = <T as FromBBoxForm<'a, 'r>>::BBoxContext;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        T::bbox_init(rocket::form::Options {
            strict: true,
            ..opts
        })
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>) {
        T::bbox_push_value(ctxt, field, request)
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>) {
        T::bbox_push_data(ctxt, field, request).await
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'a, Self> {
        Ok(T::bbox_finalize(this).ok())
    }
}

#[rocket::async_trait]
impl<'a, 'r, T: FromBBoxForm<'a, 'r>> FromBBoxForm<'a, 'r> for BBoxFormResult<'a, T> {
    type BBoxContext = <T as FromBBoxForm<'a, 'r>>::BBoxContext;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        T::bbox_init(opts)
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>) {
        T::bbox_push_value(ctxt, field, request)
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>) {
        T::bbox_push_data(ctxt, field, request).await
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'a, Self> {
        Ok(T::bbox_finalize(this))
    }
}

// Implement FromBBoxForm for pairs if inner types also implement FromBBoxForm.
pub struct PairContext<'a, 'r, A: FromBBoxForm<'a, 'r>, B: FromBBoxForm<'a, 'r>> {
    left: A::BBoxContext,
    right: B::BBoxContext,
    errors: rocket::form::Errors<'a>,
}
impl<'a, 'r, A: FromBBoxForm<'a, 'r>, B: FromBBoxForm<'a, 'r>> PairContext<'a, 'r, A, B> {
    fn context(
        &mut self,
        name: rocket::form::name::NameView<'a>,
    ) -> std::result::Result<
        Either<&mut A::BBoxContext, &mut B::BBoxContext>,
        rocket::form::Error<'a>,
    > {
        match name.key().map(|k| k.as_str()) {
            Some("0") => Ok(Either::Left(&mut self.left)),
            Some("1") => Ok(Either::Right(&mut self.right)),
            _ => Err(
                rocket::form::Error::from(&[Cow::Borrowed("0"), Cow::Borrowed("1")])
                    .with_entity(rocket::form::error::Entity::Index(0))
                    .with_name(name),
            ),
        }
    }
}
#[rocket::async_trait]
impl<'a, 'r: 'a, A: FromBBoxForm<'a, 'r>, B: FromBBoxForm<'a, 'r>> FromBBoxForm<'a, 'r> for (A, B) {
    type BBoxContext = PairContext<'a, 'r, A, B>;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        PairContext {
            left: A::bbox_init(opts),
            right: B::bbox_init(opts),
            errors: rocket::form::Errors::new(),
        }
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>) {
        match ctxt.context(field.name) {
            Ok(Either::Left(ctxt)) => A::bbox_push_value(ctxt, field.shift(), request),
            Ok(Either::Right(ctxt)) => B::bbox_push_value(ctxt, field.shift(), request),
            Err(e) => ctxt.errors.push(e),
        }
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>) {
        match ctxt.context(field.name) {
            Ok(Either::Left(ctxt)) => A::bbox_push_data(ctxt, field.shift(), request).await,
            Ok(Either::Right(ctxt)) => B::bbox_push_data(ctxt, field.shift(), request).await,
            Err(e) => ctxt.errors.push(e),
        }
    }
    fn bbox_finalize(mut ctxt: Self::BBoxContext) -> BBoxFormResult<'a, Self> {
        match (A::bbox_finalize(ctxt.left), B::bbox_finalize(ctxt.right)) {
            (Ok(key), Ok(val)) if ctxt.errors.is_empty() => Ok((key, val)),
            (Ok(_), Ok(_)) => Err(ctxt.errors)?,
            (left, right) => {
                if let Err(e) = left {
                    ctxt.errors.extend(e);
                }
                if let Err(e) = right {
                    ctxt.errors.extend(e);
                }
                Err(ctxt.errors)?
            }
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r: 'a, T: FromBBoxForm<'a, 'r> + Sync> FromBBoxForm<'a, 'r> for Arc<T> {
    type BBoxContext = <T as FromBBoxForm<'a, 'r>>::BBoxContext;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        T::bbox_init(opts)
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'a>, request: BBoxRequest<'a, 'r>) {
        T::bbox_push_value(ctxt, field, request)
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'a, 'r>, request: BBoxRequest<'a, 'r>) {
        T::bbox_push_data(ctxt, field, request).await
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'a, Self> {
        T::bbox_finalize(this).map(Arc::new)
    }
}
