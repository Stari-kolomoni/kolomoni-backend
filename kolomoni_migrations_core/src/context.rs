use sqlx::PgConnection;


pub struct MigrationContext<'c> {
    database_connection: &'c mut PgConnection,
}

impl<'c> MigrationContext<'c> {
    #[inline]
    pub(crate) fn new(database_connection: &'c mut PgConnection) -> Self {
        Self {
            database_connection,
        }
    }
}

impl<'c> MigrationContext<'c> {
    pub fn database_connection(&'c mut self) -> &'c mut PgConnection {
        self.database_connection
    }
}
