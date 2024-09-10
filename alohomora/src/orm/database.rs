use sea_orm::{ConnectionTrait, ConnectOptions, DatabaseBackend, DbBackend, DbErr};

pub use sea_orm::{Schema as BBoxSchema, Statement as BBoxStatement, ExecResult as BBoxExecResult, QueryResult as BBoxQueryResult};

// Use this to connect.
pub struct BBoxDatabase {}

impl BBoxDatabase {
    pub async fn connect<C: Into<ConnectOptions>>(opt: C) -> Result<BBoxDatabaseConnection, DbErr> {
        let conn = sea_orm::Database::connect(opt).await?;
        Ok(BBoxDatabaseConnection { conn })
    }
}

// ConnectionTrait interface.
#[rocket::async_trait]
pub trait BBoxConnectionTrait {
    fn get_database_backend(&self) -> DatabaseBackend;

    /// Execute a [Statement]
    async fn execute(&self, stmt: BBoxStatement) -> Result<BBoxExecResult, DbErr>;

    /// Execute a unprepared [Statement]
    async fn execute_unprepared(&self, sql: &str) -> Result<BBoxExecResult, DbErr>;

    /// Execute a [Statement] and return a query
    async fn query_one(&self, stmt: BBoxStatement) -> Result<Option<BBoxQueryResult>, DbErr>;

    /// Execute a [Statement] and return a collection Vec<[QueryResult]> on success
    async fn query_all(&self, stmt: BBoxStatement) -> Result<Vec<BBoxQueryResult>, DbErr>;

    /// Supports using RETURNING syntax.
    fn support_returning(&self) -> bool;
}

// A connection to DB for reading/writing.
pub struct BBoxDatabaseConnection {
    conn: sea_orm::DatabaseConnection,
}

impl BBoxDatabaseConnection {
    pub async fn ping(&self) -> Result<(), DbErr> {
        self.conn.ping().await
    }
    pub async fn close(self) -> Result<(), DbErr> {
        self.conn.close().await
    }
}

/*
#[rocket::async_trait]
impl BBoxConnectionTrait for BBoxDatabaseConnection {
    fn get_database_backend(&self) -> DatabaseBackend {
        self.conn.get_database_backend()
    }
    async fn execute(&self, stmt: BBoxStatement) -> Result<BBoxExecResult, DbErr> {
        self.conn.execute(stmt).await
    }
    async fn execute_unprepared(&self, sql: &str) -> Result<BBoxExecResult, DbErr> {
        self.conn.execute_unprepared(sql).await
    }
    async fn query_one(&self, stmt: BBoxStatement) -> Result<Option<BBoxQueryResult>, DbErr> {
        self.conn.query_one(stmt).await
    }
    async fn query_all(&self, stmt: BBoxStatement) -> Result<Vec<BBoxQueryResult>, DbErr> {
        self.conn.query_all(stmt).await
    }
    fn support_returning(&self) -> bool {
        self.conn.support_returning()
    }
}
 */

#[rocket::async_trait]
impl ConnectionTrait for BBoxDatabaseConnection {
    fn get_database_backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }
    async fn execute(&self, stmt: BBoxStatement) -> Result<BBoxExecResult, DbErr> {
        self.conn.execute(stmt).await
    }
    async fn execute_unprepared(&self, sql: &str) -> Result<BBoxExecResult, DbErr> {
        self.conn.execute_unprepared(sql).await
    }
    async fn query_one(&self, stmt: BBoxStatement) -> Result<Option<BBoxQueryResult>, DbErr> {
        self.conn.query_one(stmt).await
    }
    async fn query_all(&self, stmt: BBoxStatement) -> Result<Vec<BBoxQueryResult>, DbErr> {
        self.conn.query_all(stmt).await
    }
}

impl Default for BBoxDatabaseConnection {
    fn default() -> Self {
        BBoxDatabaseConnection { conn: sea_orm::DatabaseConnection::default() }
    }
}