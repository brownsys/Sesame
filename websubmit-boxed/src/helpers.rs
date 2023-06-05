use std::collections::HashMap;

pub enum JoinIdx {
  Left(usize),
  Right(usize),
}

pub fn left_join(left: Vec<Vec<mysql::Value>>, right: Vec<Vec<mysql::Value>>, lid: usize, rid: usize, idx: Vec<JoinIdx>) -> Vec<Vec<mysql::Value>> {
  let mut rmap = HashMap::new();
  for (i, r) in right.iter().enumerate() {
      let id: u64 = mysql::from_value(r[rid].clone());
      rmap.insert(id, i);
  }

  left.into_iter()
      .map(|r| {
          let id: u64 = mysql::from_value(r[lid].clone());
          let other = rmap.get(&id);

          let mut vec = Vec::new();
          for i in idx.iter() {
            match i {
              JoinIdx::Left(i) => vec.push(r[*i].clone()),
              JoinIdx::Right(i) => match other {
                None => vec.push(mysql::Value::NULL),
                Some(oidx) => vec.push(right[*oidx][*i].clone()),
              },
            }
          }
          vec
      })
      .collect()
}
