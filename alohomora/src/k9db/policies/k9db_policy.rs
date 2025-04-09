use std::collections::HashMap;
use std::sync::RwLock;
use crate::policy::{AnyPolicy, Policy};

// A K9db policy must implement this trait (and register itself with add_k9db_policy(..)).
pub trait K9dbPolicy: Policy + Clone + 'static {
    fn from_row(metadata: Vec<String>) -> Self;
    fn order_args(args: HashMap<String, String>) -> Vec<String>;
    fn only_k9db() -> bool { false }
    // Make a factory for this policy.
    fn factory() -> K9dbPolicyFactory {
        K9dbPolicyFactory {
            from_row: Box::new(|metadata| AnyPolicy::new(Self::from_row(metadata))),
            order_args: Box::new(|args| Self::order_args(args)),
            only_k9db: Self::only_k9db(),
        }
    }
}

// Factory struct
pub struct K9dbPolicyFactory {
    pub from_row: Box<dyn (Fn(Vec<String>) -> AnyPolicy) + Send + Sync>,
    pub order_args: Box<dyn (Fn(HashMap<String, String>) -> Vec<String>) + Send + Sync>,
    pub only_k9db: bool,
}

// Global static singleton.
// policy_name -> factory
type K9dbPolicyMap = HashMap<String, K9dbPolicyFactory>;
lazy_static! {
    static ref K9DB_POLICIES: RwLock<K9dbPolicyMap> = RwLock::new(K9dbPolicyMap::new());
}

// Register Policy T as a schema policy associated with the table and column.
// Never use this function directly, instead use the #[schema_policy(...)] macro.
pub use small_ctor::ctor as register;
pub fn add_k9db_policy<T: K9dbPolicy>(name: String) {
    let mut map = K9DB_POLICIES.write().unwrap();
    assert!(!map.contains_key(&name));
    map.insert(name, T::factory());
}
pub fn is_a_k9db_policy(name: &String) -> bool {
    let map = K9DB_POLICIES.read().unwrap();
    map.contains_key(name)
}
pub fn create_k9db_policy(policy_name: String, metadata: Vec<String>) -> AnyPolicy {
    let map = K9DB_POLICIES.read().unwrap();
    let entry = map.get(&policy_name).unwrap();
    let from_row = &entry.from_row;
    from_row(metadata)
}
pub fn order_k9db_policy_args(name: &String, args: HashMap<String, String>) -> Vec<String> {
    let map = K9DB_POLICIES.read().unwrap();
    let order_args = &map.get(name).unwrap().order_args;
    order_args(args)
}