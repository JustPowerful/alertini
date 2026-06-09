use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel::RunQueryDsl;    

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn create_pool() -> (bool, Pool) {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");    
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let mut conn = pool.get().expect("Failed to get connection");   

    // returns a boolean when checked
    let status = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
        .first::<i32>(&mut conn)
        .is_ok();

    (status, pool)
}
