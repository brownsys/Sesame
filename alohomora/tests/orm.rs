use alohomora::bbox::BBox;
use alohomora::context::UnprotectedContext;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseBackend, DbErr, QueryOrder, Set};

use alohomora::orm::{BBoxDatabase, BBoxDatabaseConnection, BBoxSchema, ORMPolicy};
use alohomora::policy::{NoPolicy, Reason, SimplePolicy};

#[derive(Clone)]
pub struct MyPolicy {
    pub name: String,
}
impl SimplePolicy for MyPolicy {
    fn simple_name(&self) -> String {
        todo!()
    }
    fn simple_check(&self, _context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
        todo!()
    }
    fn simple_join_direct(&mut self, _other: &mut Self) {
        todo!()
    }
}
impl ORMPolicy for MyPolicy {
    fn from_result(result: &QueryResult) -> Self {
        MyPolicy {
            name: result.try_get("", "name").unwrap(),
        }
    }
    fn empty() -> Self {
        MyPolicy {
            name: String::from(""),
        }
    }
}

mod grade {
    use alohomora::bbox::BBox;
    use alohomora::policy::NoPolicy;
    use sea_orm::entity::prelude::*;
    use std::convert::TryInto;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: BBox<i32, NoPolicy>,
        pub name: BBox<String, NoPolicy>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

async fn setup_schema(db: &BBoxDatabaseConnection) {
    // Setup Schema helper
    let schema = BBoxSchema::new(DatabaseBackend::Sqlite);

    // Derive from Entity
    let stmt = schema.create_table_from_entity(grade::Entity);

    // Execute create table statement
    let result = db.execute(db.get_database_backend().build(&stmt)).await;

    assert!(result.is_ok());
}

fn bbox<T>(t: T) -> BBox<T, NoPolicy> {
    BBox::new(t, NoPolicy {})
}

async fn test() -> Result<(), DbErr> {
    // Connecting SQLite
    let db = BBoxDatabase::connect("sqlite::memory:").await?;

    // Setup database schema
    setup_schema(&db).await;

    // Performing tests
    let grade1 = grade::ActiveModel {
        id: Set(bbox(1)),
        name: Set(BBox::new("Kinan".to_owned(), NoPolicy {})),
    };
    let grade2 = grade::ActiveModel {
        id: Set(bbox(2)),
        name: Set(BBox::new("Artem".to_owned(), NoPolicy {})),
    };

    // Insert them.
    let result = grade::Entity::insert_many([grade1, grade2]).exec(&db).await;
    assert!(result.is_ok());

    // Select them
    let result = grade::Entity::find()
        .order_by_desc(grade::Column::Id)
        .all(&db)
        .await
        .unwrap();
    assert_eq!(
        result,
        vec![
            grade::Model {
                id: bbox(2),
                name: BBox::new("Artem".to_owned(), NoPolicy {})
            },
            grade::Model {
                id: bbox(1),
                name: BBox::new("Kinan".to_owned(), NoPolicy {})
            },
        ]
    );

    let result = grade::Entity::find_by_id(bbox(2)).one(&db).await.unwrap();
    assert_eq!(
        result.unwrap(),
        grade::Model {
            id: bbox(2),
            name: BBox::new("Artem".to_owned(), NoPolicy {})
        }
    );

    let result = grade::Entity::find()
        .filter(grade::Column::Name.eq("Kinan"))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(
        result,
        vec![grade::Model {
            id: bbox(1),
            name: BBox::new("Kinan".to_owned(), NoPolicy {})
        }]
    );

    Ok(())
}

#[test]
fn mytest() {
    let x = tokio_test::block_on(test());
    x.unwrap();
}
