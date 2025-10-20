use sea_orm::sea_query::{ArrayType, Nullable, ValueType, ValueTypeErr};
use sea_orm::{ColIdx, ColumnType, DbErr, QueryResult, TryFromU64, TryGetError, TryGetable, Value};

use crate::{ORMBBox, ORMPolicy};

// Now, ORMBBox<T, Policy> can be used in models.
impl<T: TryGetable, P: ORMPolicy> TryGetable for ORMBBox<T, P> {
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let value = T::try_get_by(res, index)?;
        Ok(ORMBBox {
            t: value,
            p: P::from_result(res),
        })
    }
}

// Now ORMBBox<T, Policy> can be used in conditions and building ORM queries.
impl<T: Into<Value>, P: ORMPolicy> From<ORMBBox<T, P>> for Value {
    fn from(value: ORMBBox<T, P>) -> Self {
        value.t.into()
    }
}

impl<T: ValueType, P: ORMPolicy> ValueType for ORMBBox<T, P> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        Ok(ORMBBox {
            t: T::try_from(v)?,
            p: P::empty(),
        })
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

impl<T: TryFromU64, P: ORMPolicy> TryFromU64 for ORMBBox<T, P> {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        Ok(ORMBBox {
            t: T::try_from_u64(n)?,
            p: P::empty(),
        })
    }
}

impl<T: Nullable, P: ORMPolicy> Nullable for ORMBBox<T, P> {
    fn null() -> Value {
        T::null()
    }
}
