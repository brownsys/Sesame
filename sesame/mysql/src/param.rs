use sesame::pcon::{EitherPCon, PCon};
use sesame::policy::{AnyPolicy, AnyPolicyable};

// Our params may be pcons or clear.
pub trait PConParam {
    fn get(self) -> EitherPCon<mysql::Value, AnyPolicy>;
}

// Implement for basic types.
macro_rules! pcon_param_impl {
  ($($T:ty,)+) => (
    $(
    impl PConParam for $T {
        fn get(self) -> EitherPCon<mysql::Value, AnyPolicy> {
            EitherPCon::Left(self.into())
        }
    }
    )+
  );
}
pcon_param_impl!(String, &str,);
pcon_param_impl!(u8, u16, u32, u64, u128, usize,);
pcon_param_impl!(i8, i16, i32, i64, i128, isize,);
pcon_param_impl!(bool, f32, f64,);

impl<T: Into<mysql::Value>, P: AnyPolicyable> PConParam for PCon<T, P> {
    fn get(self) -> EitherPCon<mysql::Value, AnyPolicy> {
        EitherPCon::Right(self.into_any_policy_no_clone().into_pcon())
    }
}

/*
impl<T: Into<mysql::Value>> PConParam for T where T: From<mysql::Value> {
    fn get(self) -> EitherPCon<mysql::Value, AnyPolicy> {
        EitherPCon::<mysql::Value, AnyPolicy>::Value(self.into())
    }
}
 */

impl<T: Into<mysql::Value>, P: AnyPolicyable> PConParam for EitherPCon<T, P> {
    fn get(self) -> EitherPCon<mysql::Value, AnyPolicy> {
        match self {
            EitherPCon::Left(t) => EitherPCon::Left(t.into()),
            EitherPCon::Right(pcon) => {
                EitherPCon::Right(pcon.into_any_policy_no_clone().into_pcon())
            }
        }
    }
}
