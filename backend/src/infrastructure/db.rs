use sea_orm::Database as OtherDatabase;
pub use sea_orm::DatabaseConnection;
use std::env;

#[derive(Debug, Clone)]
pub struct Database {
    pub connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Self {
        let connection = OtherDatabase::connect(
            env::var("DATABASE_URL").expect("[Error] Could not load environment variables"),
        )
        .await
        .expect("[Error] Could not connect to database");

        Database { connection }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}
