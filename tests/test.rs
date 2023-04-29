#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::vec;

    use sqlx::{Pool, Row, Error};
    use sqlx::mysql::{MySql, MySqlRow, MySqlPoolOptions};
    use sqlx::types::chrono::DateTime;
    use rmcs_auth_db::Auth;

    async fn get_connection_pool() -> Result<Pool<MySql>, Error>
    {
        dotenvy::dotenv().ok();
        let url = std::env::var("TEST_DATABASE_URL").unwrap();
        MySqlPoolOptions::new()
            .max_connections(100)
            .connect(&url)
            .await
    }

    async fn check_tables_exist(pool: &Pool<MySql>) -> Result<bool, Error>
    {
        let sql = "SHOW TABLES;";
        let tables: Vec<String> = sqlx::query(sql)
            .map(|row: MySqlRow| row.get(0))
            .fetch_all(pool)
            .await?;

        Ok(tables == vec![
            String::from("_sqlx_migrations"),
            String::from("api"),
            String::from("api_procedure"),
            String::from("auth_access"),
            String::from("auth_role"),
            String::from("auth_token"),
            String::from("auth_user"),
        ])
    }

    #[sqlx::test]
    async fn test_auth()
    {
        // std::env::set_var("RUST_BACKTRACE", "1");

        let pool = get_connection_pool().await.unwrap();
        let auth = Auth::new_with_pool(pool);

        // drop tables from previous test if exists
        if check_tables_exist(&auth.pool).await.unwrap() {
            sqlx::migrate!().undo(&auth.pool, 2).await.unwrap();
        }
        // create tables for testing
        sqlx::migrate!().run(&auth.pool).await.unwrap();

        // drop tables after testing
        sqlx::migrate!().undo(&auth.pool, 2).await.unwrap();
    }

}
