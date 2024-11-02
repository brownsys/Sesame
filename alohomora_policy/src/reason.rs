

#[macro_export]
macro_rules! never_leaked { () => { $crate::match_reasons!() } }
#[macro_export]
macro_rules! to_db { () => { $crate::match_reasons!([alohomora::policy::Reason::DB(_, _)]) } }
#[macro_export]
macro_rules! to_pcr { () => { $crate::match_reasons!([alohomora::policy::Reason::Custom(_)]) } }

// #[macro_export]
// macro_rules! to_pcr {
//     () => {
//         $crate::match_reasons!([alohomora::policy::Reason::Custom(_)])
//     }
// }

#[macro_export]
/// Returns a lambda that can check a reason
/// and only return true if it fully matches the condition for a reason
macro_rules! match_reasons {
    ($([$reason: pat $(, $cond: expr)?])*) => {
        |given_reason: alohomora::policy::Reason| {
            match given_reason {
                $($reason => $($cond &&)? true,)*
                _ => false
            }
        }
    }
}