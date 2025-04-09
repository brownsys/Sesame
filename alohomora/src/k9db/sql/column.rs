use crate::k9db::schema::column::Column;
use crate::k9db::schema::constraint::Constraint;

impl Column {
    pub fn to_sql(&self) -> String {
        let mut constraints = Vec::new();
        for (constraint, value) in &self.constraints {
            if constraint == "primary key" {
                constraints.push(String::from("PRIMARY KEY"));
            } else if constraint == "unique" {
                constraints.push(String::from("UNIQUE"));
            } else if constraint == "auto increment" {
                constraints.push(String::from("AUTO_INCREMENT"));
            } else if constraint == "foreign key" {
                constraints.push(format!("REFERENCES {}", value.target_column()));
            } else if constraint == "owned by" {
                constraints.push(format!("OWNED_BY {}", value.target_column()));
            } else {
                panic!("unknown constraint {}", constraint);
            }
        }
        let mut constraints = constraints.join(" ");
        if !constraints.is_empty() {
            constraints = format!(" {}", constraints);
        }
        format!("{} {}{}", self.name, self.ty, constraints)
    }
}