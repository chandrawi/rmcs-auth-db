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

        // create new resource API
        auth.create_resource("NewAPI", "localhost", None).await.unwrap();
        // get newly created resource at the last of resource API list
        let resources = auth.list_resource().await.unwrap();
        let resource = resources.into_iter().last().unwrap();

        // create new procedure for newly created resource
        auth.create_procedure(resource.id, "NewService", "NewProcedure", None).await.unwrap();
        // get newly created procedure at the last of procedure list
        let procedures = auth.list_procedure_by_api(resource.id).await.unwrap();
        let procedure = procedures.into_iter().last().unwrap();

        assert_eq!(
            (resource.name.clone(), resource.address.clone()), 
            ("NewAPI".to_owned(), "localhost".to_owned())
        );
        assert_eq!(
            (procedure.api_id.unwrap(), procedure.service.clone(), procedure.procedure.clone()),
            (resource.id, "NewService".to_owned(), "NewProcedure".to_owned())
        );

        // update created procedure and resource API
        auth.update_procedure(procedure.id, None, None, Some("New procedure")).await.unwrap();
        auth.update_resource(resource.id, None, None, Some("New resource api")).await.unwrap();

        // get updated resource
        let resource = auth.read_resource_by_name(&resource.name.clone()).await.unwrap();
        let procedure = auth.read_procedure_by_name(resource.id, &procedure.service.clone(), &procedure.procedure.clone()).await.unwrap();
        let resource_procedure = resource.procedures.into_iter().last().unwrap();

        assert_eq!(resource.description.unwrap(), "New resource api".to_owned());
        assert_eq!(procedure.description.unwrap(), "New procedure".to_owned());
        assert_eq!(procedure.id, resource_procedure.id);

        // delete resource and procedure
        auth.delete_procedure(procedure.id).await.unwrap();
        auth.delete_resource(resource.id).await.unwrap();

        // try to get procedure and resource API
        let try_procedure = auth.read_procedure(procedure.id).await;
        let try_resource = auth.read_resource(resource.id).await;

        assert!(try_procedure.is_err());
        assert!(try_resource.is_err());

        // drop tables after testing
        sqlx::migrate!().undo(&auth.pool, 2).await.unwrap();
    }

}
