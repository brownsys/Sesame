use std::slice::Iter;
use std::iter::{Peekable, Iterator};

use crate::bbox::BBox;
use crate::db::BBoxValue;
use crate::k9db::db::BBoxK9dbColumnSet;
use crate::k9db::policies::create_k9db_policy;
use crate::policy::{AnyPolicy, NoPolicy, PolicyAnd, PolicyOr};

// Policy Column name parsing.
pub fn is_policy_column(column_name: &str) -> bool {
    column_name.starts_with("$_")
}
pub fn parse_policy_column_name(column_name: &str) -> (char, String, String) {
    // returns (op, column name, policy name), where
    //   op: one of P, &, |, or ).
    //   column name: the name of the column this policy applies to.
    //   policy name: policy type name as registered by K9dbPolicy trait.
    let split: Vec<_> = column_name[2..].split("__").collect();
    let op = split[0].as_bytes()[0] as char;
    let column_name = split[1].to_owned();
    let policy_name = split[2].to_owned();
    (op, column_name, policy_name)
}
pub fn parse_single_policy(row: &Vec<mysql::Value>, policy_name: &String, index: &usize) -> PolicyArgs {
    let value: String = mysql::from_value(row[*index].clone());
    let split = value.split(';').map(|v| v.to_owned()).collect();
    PolicyArgs::Single(policy_name.clone(), split)
}

// Parsing of policies provided by the database on query result.
pub enum PolicyArgs {
    None,
    // Policy name, vec of args.
    Single(String, Vec<String>),
    // And or Or (recursive).
    And(Vec<PolicyArgs>),
    Or(Vec<PolicyArgs>),
}
impl PolicyArgs {
    pub fn parse(
        row: &Vec<mysql::Value>,
        policy_cols: &mut Peekable<Iter<(usize, char, String)>>,
    ) -> Self {
        let mut vec = Vec::new();
        let mut global_op = 'P';
        // 1st policy column is always consumed.
        match policy_cols.next() {
            None => { return PolicyArgs::None; },
            Some((index, op, policy_name)) => {
                if *op == 'P' {
                    return parse_single_policy(row, policy_name, index);
                } else if *op == '&' || *op == '|' {
                    global_op = *op;
                    vec.push(parse_single_policy(row, policy_name, index));
                } else {
                    panic!("Malformed policy columns schemas");
                }
            }
        }
        // if 1st policy column is & or |, then we have more columns to parse.
        while let Some((index, op, policy_name)) = policy_cols.peek() {
            if *op == 'P' {
                // another brick in the wall.
                vec.push(parse_single_policy(row, policy_name, index));
                policy_cols.next();
            } else if *op == ')' {
                // we are done parsing this layer, stop iterating and return.
                vec.push(parse_single_policy(row, policy_name, index));
                policy_cols.next();
                if global_op == '&' {
                    return PolicyArgs::And(vec);
                } else {
                    return PolicyArgs::Or(vec);
                }
            } else if *op == '&' || *op == '|' {
                // a nested layer, recurse and continue!
                vec.push(PolicyArgs::parse(row, policy_cols));
            } else {
                panic!("Malformed policy columns schemas");
            }
        }
        panic!("Malformed policy columns schemas");
    }
    pub fn construct_policy(self) -> AnyPolicy {
        match self {
            PolicyArgs::None => AnyPolicy::new(NoPolicy {}),
            PolicyArgs::Single(policy_name, policy_args) => create_k9db_policy(policy_name, policy_args),
            PolicyArgs::And(vec) => {
                vec.into_iter()
                    .map(PolicyArgs::construct_policy)
                    .reduce(|left, right| {
                        AnyPolicy::new(PolicyAnd::new(left, right))
                    })
                    .unwrap()
            },
            PolicyArgs::Or(vec) => {
                vec.into_iter()
                    .map(PolicyArgs::construct_policy)
                    .reduce(|left, right| {
                        AnyPolicy::new(PolicyOr::new(left, right))
                    })
                    .unwrap()
            },
        }
    }
}

pub fn attach_policies_to_values(row: Vec<mysql::Value>, columns: &BBoxK9dbColumnSet) -> Vec<BBoxValue> {
    let mut result = Vec::new();
    for i in 0..columns.columns.len() {
        // Must attach policies of value at column i.
        let value = row.get(i).unwrap().clone();
        let column = &columns.columns[i];
        let name = column.name_str();
        match columns.policy_cols.get(&name.into_owned()) {
            None => result.push(BBox::new(value, AnyPolicy::new(NoPolicy {}))),
            Some(policy_cols) => {
                let args = PolicyArgs::parse(&row, &mut policy_cols.iter().peekable());
                result.push(BBox::new(value, args.construct_policy()));
            }
        }
    }
    result
}