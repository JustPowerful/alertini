use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn main() {
    dotenvy::dotenv().ok(); // initialize the dotenv variables from .env file
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut conn =
        PgConnection::establish(&database_url).expect("Failed to connect");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    println!("Migrations completed");
}