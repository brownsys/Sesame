extern crate indexmap;

use std::borrow::Cow;
use std::boxed::Box;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::option::Option;
use std::result::Result;
use std::sync::Arc;

use indexmap::IndexMap;

use rocket::http::uncased::AsUncased;
use rocket::Either;

use crate::bbox::BBox;
use crate::rocket::data::BBoxData;
use crate::rocket::request::BBoxRequest;

// TODO(babman): make sure errors do not leak bboxed info.
pub type BBoxFormResult<'v, T> = Result<T, rocket::form::Errors<'v>>;

// BBoxForm is just a wrapper around types that satisify FromBBoxForm.
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
pub struct BBoxValueField<'r> {
    pub name: rocket::form::name::NameView<'r>,
    pub value: BBox<String>,
    pub(crate) plain_value: &'r str,
}
impl<'r> BBoxValueField<'r> {
    pub fn shift(mut self) -> Self {
        self.name.shift();
        self
    }
    pub fn unexpected(&self) -> rocket::form::Error<'r> {
        rocket::form::Error::from(rocket::form::error::ErrorKind::Unexpected)
            .with_name(self.name.source())
            .with_value("<boxed>")
            .with_entity(rocket::form::error::Entity::ValueField)
    }
    pub fn missing(&self) -> rocket::form::Error<'r> {
        rocket::form::Error::from(rocket::form::error::ErrorKind::Missing)
            .with_name(self.name.source())
            .with_value("<boxed>")
            .with_entity(rocket::form::error::Entity::ValueField)
    }
    pub fn from_value(value: &'r str) -> Self {
        BBoxValueField {
            name: rocket::form::name::NameView::new(""),
            value: BBox::new(value.to_string(), vec![]),
            plain_value: value,
        }
    }
}

pub struct BBoxDataField<'r, 'i> {
    pub name: rocket::form::name::NameView<'r>,
    pub content_type: rocket::http::ContentType,
    pub request: BBoxRequest<'r, 'i>,
    pub data: BBoxData<'r>,
}
impl<'r, 'i> BBoxDataField<'r, 'i> {
    pub fn shift(mut self) -> Self {
        self.name.shift();
        self
    }
    pub fn unexpected(&self) -> rocket::form::Error<'r> {
        rocket::form::Error::from(rocket::form::error::ErrorKind::Unexpected)
            .with_name(self.name.source())
            .with_entity(rocket::form::error::Entity::DataField)
    }
}

// Our version of FromFormField, this implies FromBBoxForm.
#[rocket::async_trait]
pub trait FromBBoxFormField<'r>: Send + Sized {
    fn from_bbox_value(field: BBoxValueField<'r>) -> BBoxFormResult<'r, Self> {
        Result::Err(field.unexpected())?
    }
    async fn from_bbox_data<'i>(field: BBoxDataField<'r, 'i>) -> BBoxFormResult<'r, Self> {
        Result::Err(field.unexpected())?
    }
    fn default() -> Option<Self> {
        Option::None
    }
}

// Our own FromBBoxForm trait, mirror's rockets' FromForm trait.
// Do not use directly, derive instead.
#[rocket::async_trait]
pub trait FromBBoxForm<'r>: Send + Sized {
    type BBoxContext: Send;

    // Required methods
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext;
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'r>);

    async fn bbox_push_data<'life0, 'life1>(
        ctxt: &'life0 mut Self::BBoxContext,
        field: BBoxDataField<'r, 'life1>,
    );
    fn bbox_finalize(ctxt: Self::BBoxContext) -> BBoxFormResult<'r, Self>;

    // Provided methods
    fn bbox_push_error(_ctxt: &mut Self::BBoxContext, _error: rocket::form::Error<'r>) {}
    fn bbox_default(opts: rocket::form::Options) -> Option<Self> {
        Self::bbox_finalize(Self::bbox_init(opts)).ok()
    }
}

// Auto implement FromBBoxForm for everything that implements FromBBoxFormField.
pub struct FromBBoxFieldContext<'r, T: FromBBoxFormField<'r>> {
    field_name: Option<rocket::form::name::NameView<'r>>,
    field_value: Option<BBox<String>>,
    opts: rocket::form::Options,
    value: Option<BBoxFormResult<'r, T>>,
    pushes: usize,
}
impl<'r, T: FromBBoxFormField<'r>> FromBBoxFieldContext<'r, T> {
    fn should_push(&mut self) -> bool {
        self.pushes += 1;
        self.value.is_none()
    }

    fn push(&mut self, name: rocket::form::name::NameView<'r>, result: BBoxFormResult<'r, T>) {
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
impl<'r, T: FromBBoxFormField<'r>> FromBBoxForm<'r> for T {
    type BBoxContext = FromBBoxFieldContext<'r, T>;

    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        FromBBoxFieldContext {
            opts,
            field_name: Option::None,
            field_value: Option::None,
            value: None,
            pushes: 0,
        }
    }

    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'r>) {
        if ctxt.should_push() {
            ctxt.field_value = Option::Some(field.value.clone());
            ctxt.push(field.name, Self::from_bbox_value(field))
        }
    }

    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'r, '_>) {
        if ctxt.should_push() {
            ctxt.push(field.name, Self::from_bbox_data(field).await);
        }
    }

    fn bbox_finalize(ctxt: Self::BBoxContext) -> BBoxFormResult<'r, Self> {
        let mut errors = match ctxt.value {
            Option::Some(BBoxFormResult::Ok(val)) if !ctxt.opts.strict || ctxt.pushes <= 1 => {
                return BBoxFormResult::Ok(val)
            }
            Option::Some(BBoxFormResult::Ok(_)) => {
                rocket::form::Errors::from(rocket::form::error::ErrorKind::Duplicate)
            }
            Option::Some(BBoxFormResult::Err(errors)) => errors,
            Option::None if !ctxt.opts.strict => match <T as FromBBoxFormField>::default() {
                Option::Some(default) => return Ok(default),
                Option::None => rocket::form::Errors::from(rocket::form::error::ErrorKind::Missing),
            },
            Option::None => rocket::form::Errors::from(rocket::form::error::ErrorKind::Missing),
        };
        if let Option::Some(name) = ctxt.field_name {
            errors.set_name(name);
        }
        if let Option::Some(_value) = ctxt.field_value {
            errors.set_value("<boxed>");
        }
        BBoxFormResult::Err(errors)
    }
}

// Implement FromBBoxFormField for select types whose implementation of
// FromFormField is defined by rocket and is safe.
macro_rules! impl_form_via_rocket {
  ($($T:ident),+ $(,)?) => ($(
      impl<'r> FromBBoxFormField<'r> for BBox<$T> {
          #[inline(always)]
          fn from_bbox_value(field: BBoxValueField<'r>) -> BBoxFormResult<'r, Self> {
              use rocket::form::FromFormField;
              let pfield = rocket::form::ValueField{ name: field.name, value: field.plain_value};
              let pvalue = $T::from_value(pfield)?;
              BBoxFormResult::Ok(field.value.map(|_| pvalue))
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
pub struct VecContext<'v, T: FromBBoxForm<'v>> {
    opts: rocket::form::Options,
    last_key: Option<&'v rocket::form::name::Key>,
    current: Option<T::BBoxContext>,
    errors: rocket::form::Errors<'v>,
    items: Vec<T>,
}
impl<'v, T: FromBBoxForm<'v>> VecContext<'v, T> {
    fn new(opts: rocket::form::Options) -> Self {
        VecContext {
            opts,
            last_key: None,
            current: None,
            items: vec![],
            errors: rocket::form::Errors::new(),
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
    fn context(&mut self, name: &rocket::form::name::NameView<'v>) -> &mut T::BBoxContext {
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
impl<'v, T: FromBBoxForm<'v> + 'v> FromBBoxForm<'v> for Vec<T> {
    type BBoxContext = VecContext<'v, T>;

    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        VecContext::new(opts)
    }

    fn bbox_push_value(this: &mut Self::BBoxContext, field: BBoxValueField<'v>) {
        T::bbox_push_value(this.context(&field.name), field.shift());
    }

    async fn bbox_push_data(this: &mut Self::BBoxContext, field: BBoxDataField<'v, '_>) {
        T::bbox_push_data(this.context(&field.name), field.shift()).await
    }

    fn bbox_finalize(mut this: Self::BBoxContext) -> BBoxFormResult<'v, Self> {
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
pub struct MapContext<'v, K, V>
where
    K: FromBBoxForm<'v>,
    V: FromBBoxForm<'v>,
{
    opts: rocket::form::Options,
    table: IndexMap<&'v str, usize>,
    entries: Vec<(K::BBoxContext, V::BBoxContext)>,
    metadata: Vec<rocket::form::name::NameView<'v>>,
    errors: rocket::form::Errors<'v>,
}
impl<'v, K, V> MapContext<'v, K, V>
where
    K: FromBBoxForm<'v>,
    V: FromBBoxForm<'v>,
{
    fn new(opts: rocket::form::Options) -> Self {
        MapContext {
            opts,
            table: IndexMap::new(),
            entries: vec![],
            metadata: vec![],
            errors: rocket::form::Errors::new(),
        }
    }
    fn ctxt(
        &mut self,
        key: &'v str,
        name: rocket::form::name::NameView<'v>,
    ) -> &mut (K::BBoxContext, V::BBoxContext) {
        match self.table.get(key) {
            Some(i) => &mut self.entries[*i],
            None => {
                let i = self.entries.len();
                self.table.insert(key, i);
                self.entries
                    .push((K::bbox_init(self.opts), V::bbox_init(self.opts)));
                self.metadata.push(name);
                &mut self.entries[i]
            }
        }
    }
    fn push(
        &mut self,
        name: rocket::form::name::NameView<'v>,
    ) -> Option<Either<&mut K::BBoxContext, &mut V::BBoxContext>> {
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
                    K::bbox_push_value(key_ctxt, BBoxValueField::from_value(key));
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
    fn push_value(&mut self, field: BBoxValueField<'v>) {
        match self.push(field.name) {
            Some(Either::Left(ctxt)) => K::bbox_push_value(ctxt, field.shift()),
            Some(Either::Right(ctxt)) => V::bbox_push_value(ctxt, field.shift()),
            _ => {}
        }
    }
    async fn push_data(&mut self, field: BBoxDataField<'v, '_>) {
        match self.push(field.name) {
            Some(Either::Left(ctxt)) => K::bbox_push_data(ctxt, field.shift()).await,
            Some(Either::Right(ctxt)) => V::bbox_push_data(ctxt, field.shift()).await,
            _ => {}
        }
    }
    fn finalize<T: std::iter::FromIterator<(K, V)>>(mut self) -> BBoxFormResult<'v, T> {
        let errors = &mut self.errors;
        let map: T = self
            .entries
            .into_iter()
            .zip(self.metadata.iter())
            .zip(self.table.keys())
            .filter_map(|(((k_ctxt, v_ctxt), name), idx)| {
                let key = K::bbox_finalize(k_ctxt)
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
impl<'v, K, V> FromBBoxForm<'v> for HashMap<K, V>
where
    K: FromBBoxForm<'v> + Eq + Hash,
    V: FromBBoxForm<'v>,
{
    type BBoxContext = MapContext<'v, K, V>;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        MapContext::new(opts)
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'v>) {
        ctxt.push_value(field);
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'v, '_>) {
        ctxt.push_data(field).await;
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'v, Self> {
        this.finalize()
    }
}
#[rocket::async_trait]
impl<'v, K, V> FromBBoxForm<'v> for BTreeMap<K, V>
where
    K: FromBBoxForm<'v> + Ord,
    V: FromBBoxForm<'v>,
{
    type BBoxContext = MapContext<'v, K, V>;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        MapContext::new(opts)
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'v>) {
        ctxt.push_value(field);
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'v, '_>) {
        ctxt.push_data(field).await;
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'v, Self> {
        this.finalize()
    }
}

// Implement FromBBoxForm for Option<T> (provided T also implement FromBBoxForm).
#[rocket::async_trait]
impl<'v, T: FromBBoxForm<'v>> FromBBoxForm<'v> for Option<T> {
    type BBoxContext = <T as FromBBoxForm<'v>>::BBoxContext;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        T::bbox_init(rocket::form::Options {
            strict: true,
            ..opts
        })
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'v>) {
        T::bbox_push_value(ctxt, field)
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'v, '_>) {
        T::bbox_push_data(ctxt, field).await
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'v, Self> {
        Ok(T::bbox_finalize(this).ok())
    }
}

#[rocket::async_trait]
impl<'v, T: FromBBoxForm<'v>> FromBBoxForm<'v> for BBoxFormResult<'v, T> {
    type BBoxContext = <T as FromBBoxForm<'v>>::BBoxContext;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        T::bbox_init(opts)
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'v>) {
        T::bbox_push_value(ctxt, field)
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'v, '_>) {
        T::bbox_push_data(ctxt, field).await
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'v, Self> {
        Ok(T::bbox_finalize(this))
    }
}

// Implement FromBBoxForm for pairs if inner types also implement FromBBoxForm.
pub struct PairContext<'v, A: FromBBoxForm<'v>, B: FromBBoxForm<'v>> {
    left: A::BBoxContext,
    right: B::BBoxContext,
    errors: rocket::form::Errors<'v>,
}
impl<'v, A: FromBBoxForm<'v>, B: FromBBoxForm<'v>> PairContext<'v, A, B> {
    fn context(
        &mut self,
        name: rocket::form::name::NameView<'v>,
    ) -> std::result::Result<
        Either<&mut A::BBoxContext, &mut B::BBoxContext>,
        rocket::form::Error<'v>,
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
impl<'v, A: FromBBoxForm<'v>, B: FromBBoxForm<'v>> FromBBoxForm<'v> for (A, B) {
    type BBoxContext = PairContext<'v, A, B>;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        PairContext {
            left: A::bbox_init(opts),
            right: B::bbox_init(opts),
            errors: rocket::form::Errors::new(),
        }
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'v>) {
        match ctxt.context(field.name) {
            Ok(Either::Left(ctxt)) => A::bbox_push_value(ctxt, field.shift()),
            Ok(Either::Right(ctxt)) => B::bbox_push_value(ctxt, field.shift()),
            Err(e) => ctxt.errors.push(e),
        }
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'v, '_>) {
        match ctxt.context(field.name) {
            Ok(Either::Left(ctxt)) => A::bbox_push_data(ctxt, field.shift()).await,
            Ok(Either::Right(ctxt)) => B::bbox_push_data(ctxt, field.shift()).await,
            Err(e) => ctxt.errors.push(e),
        }
    }
    fn bbox_finalize(mut ctxt: Self::BBoxContext) -> BBoxFormResult<'v, Self> {
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
impl<'v, T: FromBBoxForm<'v> + Sync> FromBBoxForm<'v> for Arc<T> {
    type BBoxContext = <T as FromBBoxForm<'v>>::BBoxContext;
    fn bbox_init(opts: rocket::form::Options) -> Self::BBoxContext {
        T::bbox_init(opts)
    }
    fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: BBoxValueField<'v>) {
        T::bbox_push_value(ctxt, field)
    }
    async fn bbox_push_data(ctxt: &mut Self::BBoxContext, field: BBoxDataField<'v, '_>) {
        T::bbox_push_data(ctxt, field).await
    }
    fn bbox_finalize(this: Self::BBoxContext) -> BBoxFormResult<'v, Self> {
        T::bbox_finalize(this).map(Arc::new)
    }
}
