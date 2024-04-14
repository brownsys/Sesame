use std::collections::HashMap;
// use std::hash::Hash;

// use mysql::prelude::FromValue;

pub enum JoinIdx {
    Left(usize),
    Right(usize),
}

pub fn left_join(
    left: Vec<Vec<mysql::Value>>,
    right: Vec<Vec<mysql::Value>>,
    lid: usize,
    rid: usize,
    idx: Vec<JoinIdx>,
) -> Vec<Vec<mysql::Value>> {
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

// pub type AvgIdx = usize;

// // Compute the average of the given column grouped by the value of the group_by column.
// // Result is on the form:
// // [
// //    [<group1>, <avg1>],
// //    [<group2>, <avg2>],
// //    ...
// // ]
// pub fn average<GroupType>(
//     column: AvgIdx,
//     group_by: AvgIdx,
//     data: Vec<Vec<mysql::Value>>,
// ) -> Vec<Vec<mysql::Value>>
// where
//     GroupType: Eq + Hash + FromValue + Into<mysql::Value>,
// {
//     let map: HashMap<GroupType, (u64, u64)> = HashMap::new();
//     data.into_iter()
//         .fold(map, |mut map, row| {
//             let group: GroupType = mysql::from_value(row[group_by].clone());
//             let value: u64 = mysql::from_value(row[column].clone());
//             let tup: &mut (u64, u64) = map.entry(group).or_default();
//             tup.0 += value;
//             tup.1 += 1;
//             map
//         })
//         .into_iter()
//         .map(|(group, (sum, count))| vec![group.into(), (sum / count).into()])
//         .collect()
// }
