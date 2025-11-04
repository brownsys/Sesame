use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseBackend, DbErr, QueryOrder, Set};

use sesame::context::UnprotectedContext;
use sesame::pcon::PCon;
use sesame::policy::{NoPolicy, Reason, SimplePolicy};

use sesame_orm::{ORMPCon, ORMPolicy, PConDatabase, PConDatabaseConnection, PConSchema};

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
    use sea_orm::entity::prelude::*;
    use sesame::policy::NoPolicy;
    use sesame_orm::ORMPCon;
    use std::convert::TryInto;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: ORMPCon<i32, NoPolicy>,
        pub name: ORMPCon<String, NoPolicy>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

async fn setup_schema(db: &PConDatabaseConnection) {
    // Setup Schema helper
    let schema = PConSchema::new(DatabaseBackend::Sqlite);

    // Derive from Entity
    let stmt = schema.create_table_from_entity(grade::Entity);

    // Execute create table statement
    let result = db.execute(db.get_database_backend().build(&stmt)).await;

    assert!(result.is_ok());
}

fn pcon<T>(t: T) -> ORMPCon<T, NoPolicy> {
    PCon::new(t, NoPolicy {}).into()
}

async fn test() -> Result<(), DbErr> {
    // Connecting SQLite
    let db = PConDatabase::connect("sqlite::memory:").await?;

    // Setup database schema
    setup_schema(&db).await;

    // Performing tests
    let grade1 = grade::ActiveModel {
        id: Set(pcon(1)),
        name: Set(pcon("Kinan".to_owned())),
    };
    let grade2 = grade::ActiveModel {
        id: Set(pcon(2)),
        name: Set(pcon("Artem".to_owned())),
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
                id: pcon(2),
                name: pcon("Artem".to_owned())
            },
            grade::Model {
                id: pcon(1),
                name: pcon("Kinan".to_owned())
            },
        ]
    );

    let result = grade::Entity::find_by_id(pcon(2)).one(&db).await.unwrap();
    assert_eq!(
        result.unwrap(),
        grade::Model {
            id: pcon(2),
            name: pcon("Artem".to_owned())
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
            id: pcon(1),
            name: pcon("Kinan".to_owned())
        }]
    );

    Ok(())
}

#[test]
fn orm_test() {
    let x = tokio_test::block_on(test());
    x.unwrap();
}
