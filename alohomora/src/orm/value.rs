use sea_orm::{ColIdx, ColumnType, DbErr, QueryResult, TryFromU64, TryGetable, TryGetError, Value};
use sea_orm::sea_query::{ArrayType, Nullable, ValueType, ValueTypeErr};

use crate::bbox::BBox;
use crate::policy::{NoPolicy, Policy};
use crate::orm::ORMPolicy;

// Now, BBox<T, Policy> can be used in models.
impl<T: TryGetable, P: Policy + ORMPolicy> TryGetable for BBox<T, P> {
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let value = T::try_get_by(res, index)?;
        Ok(BBox::new(value, P::from_result(res)))
    }
}

// Now BBox<T, Policy> can be used in conditions and building ORM queries.
impl<T: Into<Value>, P: Policy> From<BBox<T, P>> for Value {
    fn from(value: BBox<T, P>) -> Self {
        value.consume().0.into()
    }
}

impl<T: ValueType, P: ORMPolicy> ValueType for BBox<T, P> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        Ok(BBox::new(T::try_from(v)?, P::empty()))
    }

    fn type_name() -> String {
        T::type_name()
    }

    fn array_type() -> ArrayType {
        T::array_type()
    }

    fn column_type() -> ColumnType {
        T::column_type()
    }
}

impl<T: TryFromU64, P: ORMPolicy> TryFromU64 for BBox<T, P> {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        Ok(BBox::new(T::try_from_u64(n)?, P::empty()))
    }
}

impl<T: Nullable, P: Policy> Nullable for BBox<T, P> {
    fn null() -> Value {
        T::null()
    }
}
