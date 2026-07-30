use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use sesame::policy::{AnyPolicy, AnyPolicyable, NoPolicy, Policy, PolicyAnd, PolicyOr};

#[cfg(feature = "derive")]
pub use sesame_derive::schema_policy;

// Schema policies can be constructed from DB rows.
pub trait SchemaPolicy: Policy {
    fn from_row(table_name: &str, row: &mysql::Row) -> Self
    where
        Self: Sized;
}

// Impl SchemaPolicy for some policy containers.
impl SchemaPolicy for NoPolicy {
    fn from_row(_table_name: &str, _row: &mysql::Row) -> Self {
        NoPolicy {}
    }
}
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyAnd<P1, P2> {
    fn from_row(table_name: &str, row: &mysql::Row) -> Self {
        PolicyAnd::new(P1::from_row(table_name, row), P2::from_row(table_name, row))
    }
}
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyOr<P1, P2> {
    fn from_row(table_name: &str, row: &mysql::Row) -> Self {
        PolicyOr::new(P1::from_row(table_name, row), P2::from_row(table_name, row))
    }
}

// Global static singleton. Factories are individually reference
// counted so a query can snapshot the ones its columns need once and
// share them across all of its rows without holding the lock.
type SchemaPolicyFactory = dyn (Fn(&mysql::Row) -> AnyPolicy) + Send + Sync;
type SchemaPolicyMap = HashMap<(String, usize), Vec<Arc<SchemaPolicyFactory>>>;
lazy_static! {
    static ref SCHEMA_POLICIES: RwLock<SchemaPolicyMap> = RwLock::new(SchemaPolicyMap::new());
}

// Helper to fold an iterator of policies into an AndPolicy.
fn fold_policies<I: Iterator<Item = AnyPolicy>>(mut policies: I) -> AnyPolicy {
    match policies.next() {
        None => AnyPolicy::new(NoPolicy {}),
        Some(mut policy) => {
            for next in policies {
                policy = AnyPolicy::new(PolicyAnd::new(policy, next));
            }
            policy
        }
    }
}

// The registered factories for every column of a result set, resolved
// once per query and shared by all of its rows. Indexed by column
// position; columns with no registered policy hold an empty Vec.
pub(crate) struct ColumnPolicies {
    factories: Vec<Vec<Arc<SchemaPolicyFactory>>>,
}

impl ColumnPolicies {
    // Resolve the factories for a result set's columns. One lock
    // acquisition and one map lookup per column, instead of one per
    // cell. Registration runs in `ctor` functions before main (the
    // #[schema_policy] macro is the only sanctioned caller of
    // add_schema_policy), so the registry cannot change between this
    // snapshot and the rows it serves.
    pub(crate) fn resolve(columns: &[mysql::Column]) -> Arc<Self> {
        let map = SCHEMA_POLICIES.read().unwrap();
        let factories = columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                map.get(&(col.table_str().into_owned(), idx))
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();
        Arc::new(ColumnPolicies { factories })
    }

    // Build the policy for one cell, exactly as get_schema_policies
    // does: the fold of every registered factory applied to the row,
    // or NoPolicy when none are registered.
    pub(crate) fn for_cell(&self, column: usize, row: &mysql::Row) -> AnyPolicy {
        match self.factories.get(column) {
            None => AnyPolicy::new(NoPolicy {}),
            Some(factories) if factories.is_empty() => AnyPolicy::new(NoPolicy {}),
            Some(factories) => fold_policies(factories.iter().map(|factory| factory(row))),
        }
    }
}

// Create policies for a cell given its entire row and the name of its table.
// Query execution resolves policies through ColumnPolicies instead; this is
// the manual path into the registry, kept (hidden) so registration itself can
// be tested without a live DB.
#[doc(hidden)]
pub fn get_schema_policies(table_name: String, column: usize, row: &mysql::Row) -> AnyPolicy {
    let map = SCHEMA_POLICIES.read().unwrap();
    match map.get(&(table_name, column)) {
        None => AnyPolicy::new(NoPolicy {}),
        Some(factories) => fold_policies(factories.iter().map(|factory| factory(row))),
    }
}

// Register Policy T as a schema policy associated with the table and column.
// Never use this function directly, instead use the #[schema_policy(...)] macro.
extern crate small_ctor;
pub use small_ctor::ctor as register;
pub fn add_schema_policy<T: SchemaPolicy + AnyPolicyable>(table_name: String, column: usize) {
    let mut map = SCHEMA_POLICIES.write().unwrap();
    map.entry((table_name.clone(), column))
        .or_default()
        .push(Arc::new(move |row: &mysql::Row| {
            AnyPolicy::new(T::from_row(&table_name, row))
        }));
}
