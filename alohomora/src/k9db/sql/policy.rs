use crate::k9db::policies::{is_a_k9db_policy, order_k9db_policy_args};
use crate::k9db::schema::policy::{Policy, PolicyArgs};

impl PolicyArgs {
    pub fn is_k9db_compatible(&self) -> bool {
        is_a_k9db_policy(&self.name)
    }
    pub fn to_sql(&self) -> String {
        let args = order_k9db_policy_args(&self.name, self.args.clone()).join(";");
        if args.len() > 0 {
            format!("{};{}", self.name, args)
        } else {
            format!("{}", self.name)
        }
    }
}

impl Policy {
    fn to_sql_helper(&self) -> Option<String> {
        match self {
            Policy::None => None,
            Policy::Policy(p) => if p.is_k9db_compatible() {
                Some(format!("P {}", p.to_sql()))
            } else {
                None
            },
            Policy::And(args) => {
                let v: Vec<_> = args.iter()
                    .filter(|a| a.is_k9db_compatible())
                    .map(PolicyArgs::to_sql)
                    .collect();
                if v.len() == 0 {
                    None
                } else if v.len() == 1 {
                    Some(format!("P {}", v[0]))
                } else {
                    Some(format!("& {}", v.join(" ~ ")))
                }
            },
            Policy::Or(args) => {
                let v: Vec<_> = args.iter()
                    .filter(|a| a.is_k9db_compatible())
                    .map(PolicyArgs::to_sql)
                    .collect();
                if v.len() == 0 {
                    None
                } else if v.len() == 1 {
                    Some(format!("P {}", v[0]))
                } else {
                    Some(format!("| {}", v.join(" ~ ")))
                }
            }
        }
    }

    pub fn to_sql(&self, table: &str, column: &str) -> Option<String> {
        let policy = self.to_sql_helper()?;
        Some(format!("POLICY {}.{} {}", table, column, policy))
    }
}
