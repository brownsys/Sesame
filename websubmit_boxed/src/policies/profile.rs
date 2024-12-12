use crate::policies::ContextData;
use crate::policies::User;

// Access control policy.
// #[schema_policy(table = "users", column = 5)] // gender
// #[schema_policy(table = "users", column = 6)] // age
// #[schema_policy(table = "users", column = 7)] // ethnicity

alohomora_policy::access_control_policy!(UserProfilePolicy, ContextData, User,
    [is_not_authenticated, alohomora_policy::never_leaked!()],
    [is_owner || is_admin, alohomora_policy::anything!()]
    (alohomora_policy::never_leaked!());
    User::combine);