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
            (procedure.api_id, procedure.service.clone(), procedure.procedure.clone()),
            (resource.id, "NewService".to_owned(), "NewProcedure".to_owned())
        );

        // update created procedure and resource API
        auth.update_procedure(procedure.id, None, None, Some("New procedure")).await.unwrap();
        auth.update_resource(resource.id, None, None, Some("New resource api")).await.unwrap();

        // get updated resource
        let resource = auth.read_resource_by_name(&resource.name.clone()).await.unwrap();
        let procedure = auth.read_procedure_by_name(resource.id, &procedure.service.clone(), &procedure.procedure.clone()).await.unwrap();
        let resource_procedure = resource.procedures.into_iter().last().unwrap();

        assert_eq!(resource.description, "New resource api".to_owned());
        assert_eq!(procedure.description, "New procedure".to_owned());
        assert_eq!(procedure.id, resource_procedure.id);

        // delete resource and procedure
        auth.delete_procedure(procedure.id).await.unwrap();
        auth.delete_resource(resource.id).await.unwrap();

        // try to get procedure and resource API
        let try_procedure = auth.read_procedure(procedure.id).await;
        let try_resource = auth.read_resource(resource.id).await;

        assert!(try_procedure.is_err());
        assert!(try_resource.is_err());

        // create new Application config API and procedure
        let app_id = auth.create_application("NewApp", "localhost", None).await.unwrap();
        let proc_id = auth.create_procedure(app_id, "NewService", "NewProcedure", None).await.unwrap();

        // create new role and add access to the procedure
        let role_id = auth.create_role("user", true, false, None, None).await.unwrap();
        auth.add_role_access(role_id, proc_id).await.unwrap();

        // create new user and token
        let user_id = auth.create_user(role_id, "username", "secret", "", "", None, None).await.unwrap();
        let token_id = "ANxYLOpYMEUo1s78YMsu8Su1VomCnCQo";
        let expire = DateTime::from_str("2023-01-01T00:00:00Z").unwrap();
        auth.create_token(token_id, role_id, user_id, Some(expire), None, None).await.unwrap();

        // get role data
        let roles = auth.list_role().await.unwrap();
        let last_role = roles.into_iter().last().unwrap();
        let role = auth.read_role(role_id).await.unwrap();

        assert_eq!(last_role.id, role_id);
        assert_eq!(
            (role.name.clone(), role.secured, role.multi, role.access),
            ("user".to_owned(), true, false, vec![proc_id])
        );

        // get user data
        let users = auth.list_user_by_role(role_id).await.unwrap();
        let last_user = users.into_iter().last().unwrap();
        let user = auth.read_user(user_id).await.unwrap();

        assert_eq!(last_user.id, user.id);
        assert_eq!(
            (user.name.clone(), user.password.clone()),
            ("username".to_owned(), "secret".to_owned())
        );

        // get token data
        let role_tokens = auth.list_token_by_user(user_id).await.unwrap();
        let user_tokens = auth.list_token_by_user(user_id).await.unwrap();
        let role_token = role_tokens.into_iter().last().unwrap();
        let user_token = user_tokens.into_iter().last().unwrap();
        let token = auth.read_token(token_id).await.unwrap();

        assert_eq!(role_token.id, token_id);
        assert_eq!(user_token.id, token_id);
        assert_eq!(
            (token.role_id, token.user_id, token.limit, token.expire),
            (role_id, user_id, 0, expire)
        );

        // update role and user
        auth.update_role(role_id, None, None, Some(true), Some(43200), Some(10000)).await.unwrap();
        auth.update_user(user_id, None, None, None, None, Some("user@domain.com"), Some("+6280123456789")).await.unwrap();

        // get role and user by name
        let role = auth.read_role_by_name(&role.name.clone()).await.unwrap();
        let user = auth.read_user_by_name(&user.name.clone()).await.unwrap();

        assert_eq!(
            (role.multi, role.token_expire, role.token_limit),
            (true, 43200, 10000)
        );
        assert_eq!(
            (user.email, user.phone),
            ("user@domain.com".to_owned(), "+6280123456789".to_owned())
        );

        // delete role, user, and token
        auth.delete_token(token_id).await.unwrap();
        auth.delete_user(user_id).await.unwrap();
        auth.remove_role_access(role_id, proc_id).await.unwrap();
        auth.delete_role(role_id).await.unwrap();

        let try_token = auth.read_token(token_id).await;
        let try_user = auth.read_user(user_id).await;
        let try_role = auth.read_role(role_id).await;

        assert!(try_token.is_err());
        assert!(try_user.is_err());
        assert!(try_role.is_err());

        // drop tables after testing
        sqlx::migrate!().undo(&auth.pool, 2).await.unwrap();
    }

}
