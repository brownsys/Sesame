use sea_orm::{ConnectOptions, ConnectionTrait, DatabaseBackend, DbBackend, DbErr};

pub use sea_orm::{
    ExecResult as PConExecResult, QueryResult as PConQueryResult, Schema as PConSchema,
    Statement as PConStatement,
};

// Use this to connect.
pub struct PConDatabase {}

impl PConDatabase {
    pub async fn connect<C: Into<ConnectOptions>>(opt: C) -> Result<PConDatabaseConnection, DbErr> {
        let conn = sea_orm::Database::connect(opt).await?;
        Ok(PConDatabaseConnection { conn })
    }
}

// ConnectionTrait interface.
#[async_trait::async_trait]
pub trait PConConnectionTrait {
    fn get_database_backend(&self) -> DatabaseBackend;

    /// Execute a [Statement]
    async fn execute(&self, stmt: PConStatement) -> Result<PConExecResult, DbErr>;

    /// Execute a unprepared [Statement]
    async fn execute_unprepared(&self, sql: &str) -> Result<PConExecResult, DbErr>;

    /// Execute a [Statement] and return a query
    async fn query_one(&self, stmt: PConStatement) -> Result<Option<PConQueryResult>, DbErr>;

    /// Execute a [Statement] and return a collection Vec<[QueryResult]> on success
    async fn query_all(&self, stmt: PConStatement) -> Result<Vec<PConQueryResult>, DbErr>;

    /// Supports using RETURNING syntax.
    fn support_returning(&self) -> bool;
}

// A connection to DB for reading/writing.
pub struct PConDatabaseConnection {
    conn: sea_orm::DatabaseConnection,
}

impl PConDatabaseConnection {
    pub async fn ping(&self) -> Result<(), DbErr> {
        self.conn.ping().await
    }
    pub async fn close(self) -> Result<(), DbErr> {
        self.conn.close().await
    }
}

/*
#[async_trait::async_trait]
impl PConConnectionTrait for PConDatabaseConnection {
    fn get_database_backend(&self) -> DatabaseBackend {
        self.conn.get_database_backend()
    }
    async fn execute(&self, stmt: PConStatement) -> Result<PConExecResult, DbErr> {
        self.conn.execute(stmt).await
    }
    async fn execute_unprepared(&self, sql: &str) -> Result<PConExecResult, DbErr> {
        self.conn.execute_unprepared(sql).await
    }
    async fn query_one(&self, stmt: PConStatement) -> Result<Option<PConQueryResult>, DbErr> {
        self.conn.query_one(stmt).await
    }
    async fn query_all(&self, stmt: PConStatement) -> Result<Vec<PConQueryResult>, DbErr> {
        self.conn.query_all(stmt).await
    }
    fn support_returning(&self) -> bool {
        self.conn.support_returning()
    }
}
 */

#[async_trait::async_trait]
impl ConnectionTrait for PConDatabaseConnection {
    fn get_database_backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }
    async fn execute(&self, stmt: PConStatement) -> Result<PConExecResult, DbErr> {
        self.conn.execute(stmt).await
    }
    async fn execute_unprepared(&self, sql: &str) -> Result<PConExecResult, DbErr> {
        self.conn.execute_unprepared(sql).await
    }
    async fn query_one(&self, stmt: PConStatement) -> Result<Option<PConQueryResult>, DbErr> {
        self.conn.query_one(stmt).await
    }
    async fn query_all(&self, stmt: PConStatement) -> Result<Vec<PConQueryResult>, DbErr> {
        self.conn.query_all(stmt).await
    }
}

impl Default for PConDatabaseConnection {
    fn default() -> Self {
        PConDatabaseConnection {
            conn: sea_orm::DatabaseConnection::default(),
        }
    }
}
