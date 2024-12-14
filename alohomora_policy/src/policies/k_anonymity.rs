#[macro_export]
macro_rules! k_anonymity_policy {
    ($policy_name:ident, $min_k:expr, [ $( ( table: $table:expr, column: $column:expr ) ),+ ] $(, $schema_policy_impl:tt )?) => {
        #[derive(Clone)]
        pub struct $policy_name {
            count: u64,
            schema: std::collections::HashMap<String, usize>,
        }

        impl $policy_name {
            pub fn new() -> Self {
                let mut schema = std::collections::HashMap::new();
                $(
                    schema.insert($table.to_string(), $column);
                )+
                $policy_name {
                    count: 0,
                    schema,
                }
            }

            pub fn validate_schema(&self, table: &str, column: usize) -> bool {
                self.schema.get(table).map_or(false, |&col| col == column)
            }

            pub fn initialize_from_row(&mut self, table: &str, row: &Vec<mysql::Value>) -> Result<(), String> {
                if let Some(&col) = self.schema.get(table) {
                    if col >= row.len() {
                        return Err(format!(
                            "Column index {} out of bounds for table '{}'",
                            col, table
                        ));
                    }
                    self.count = mysql::from_value(row[col].clone());
                    Ok(())
                } else {
                    Err(format!("Table '{}' is not defined in the schema", table))
                }
            }
        }

        impl alohomora::policy::Policy for $policy_name {
            fn name(&self) -> String {
                stringify!($policy_name).to_string()
            }

            fn check(&self, _context: &alohomora::context::UnprotectedContext, _reason: alohomora::policy::Reason) -> bool {
                self.count >= $min_k
            }

            fn join(&self, other: alohomora::policy::AnyPolicy) -> Result<alohomora::policy::AnyPolicy, ()> {
                if other.is::<$policy_name>() {
                    let other = other.specialize::<$policy_name>().unwrap();
                    self.join_logic(other)
                        .map(|joined| alohomora::policy::AnyPolicy::new(joined))
                        .map_err(|_| ())
                } else {
                    Ok(alohomora::policy::AnyPolicy::new(alohomora::policy::PolicyAnd::new(
                        alohomora::policy::AnyPolicy::new(self.clone()),
                        other,
                    )))
                }
            }

            fn join_logic(&self, p2: Self) -> Result<Self, ()> {
                let mut merged_schema = self.schema.clone();
                for (table, column) in p2.schema {
                    if let Some(&existing_col) = merged_schema.get(&table) {
                        if existing_col != column {
                            return Err(());
                        }
                    } else {
                        merged_schema.insert(table, column);
                    }
                }

                Ok($policy_name {
                    count: std::cmp::min(self.count, p2.count),
                    schema: merged_schema,
                })
            }
        }

        k_anonymity_policy!(@parse_schema_policy $policy_name $(, $schema_policy_impl)? );
    };

    // If no custom schema policy block is provided, define a default one
    (@parse_schema_policy $policy_name:ident) => {
        impl alohomora::policy::SchemaPolicy for $policy_name {
            fn from_row(table: &str, row: &Vec<mysql::Value>) -> Self {
                let mut policy = $policy_name::new();
                policy
                    .initialize_from_row(table, row)
                    .expect("Failed to initialize policy from row");
                policy
            }
        }
    };

    // If a custom schema policy block is provided, use it as-is
    (@parse_schema_policy $policy_name:ident, { $($body:tt)* }) => {
        impl alohomora::policy::SchemaPolicy for $policy_name {
            $($body)*
        }
    };
}