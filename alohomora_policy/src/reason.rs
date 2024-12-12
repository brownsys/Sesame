

#[macro_export]
macro_rules! never_leaked { () => { $crate::allowed_reasons!() } }
#[macro_export]
macro_rules! only_to_db { () => { $crate::allowed_reasons!([alohomora::policy::Reason::DB(_, _)]) } }
#[macro_export]
macro_rules! only_to_pcr { () => { $crate::allowed_reasons!([alohomora::policy::Reason::Custom(_)]) } }
#[macro_export]
macro_rules! anything { () => { $crate::allowed_reasons!(, true) } }


// #[macro_export]
// macro_rules! to_pcr {
//     () => {
//         $crate::match_reasons!([alohomora::policy::Reason::Custom(_)])
//     }
// }

#[macro_export]
/// Returns a lambda that can check a reason
/// and only return true if it fully matches the condition for a reason
macro_rules! allowed_reasons {
    ($([$reason: pat $(, $cond: expr)?])*) => {
        $crate::allowed_reasons!($([$reason $(, $cond)?])*, false);
    };
    ($([$reason: pat $(, $cond: expr)?])*, $default: expr) => {
        |given_reason: alohomora::policy::Reason| {
            match given_reason {
                $($reason => $($cond &&)? true,)*
                _ => $default,
            }
        }
    };
}