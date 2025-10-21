use diesel::r2d2::{ConnectionManager, PoolError};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::error::Error;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = diesel::r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        if self.enable_wal {
            use diesel::connection::SimpleConnection;
            conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")
                .map_err(|e| diesel::r2d2::Error::QueryError(e))?;
        }
        Ok(())
    }
}

pub fn create_pool(db_url: &str) -> Result<DbPool, PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);

    diesel::r2d2::Pool::builder()
        .connection_customizer(Box::new(ConnectionOptions { enable_wal: true }))
        .build(manager)
}

pub fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| e.into())
        .map(|_| ())
}
