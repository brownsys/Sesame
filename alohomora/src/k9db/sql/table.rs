use futures::StreamExt;
use crate::k9db::schema::column::Column;
use crate::k9db::schema::table::Table;

impl Table {
    fn sql_schema(&self) -> String {
        let cols: Vec<_> = self.columns.iter().map(Column::to_sql).collect();
        let cols = cols.join(",\n  ");

        if self.data_subject {
            format!("CREATE DATA_SUBJECT TABLE {} (\n  {}\n);", self.name, cols)
        } else {
            format!("CREATE TABLE {} (\n  {}\n);", self.name, cols)
        }
    }

    pub fn to_sql(&self) -> Vec<String> {
        let mut vec = vec![self.sql_schema()];
        vec.extend(self.columns.iter()
            .map(|column| column.policy.to_sql(&self.name, &column.name))
            .filter(Option::is_some)
            .map(Option::unwrap)
        );
        vec
    }
}