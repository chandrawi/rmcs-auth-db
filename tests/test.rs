#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::{Pool, Error};
    use sqlx::postgres::{Postgres, PgPoolOptions};
    use sqlx::types::chrono::DateTime;
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use rmcs_auth_db::Auth;

    async fn get_connection_pool() -> Result<Pool<Postgres>, Error>
    {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_AUTH_TEST_URL").unwrap();
        PgPoolOptions::new()
            .max_connections(100)
            .connect(&url)
            .await
    }

    async fn truncate_tables(pool: &Pool<Postgres>) -> Result<(), Error>
    {
        let sql = "TRUNCATE TABLE \"token\", \"user_role\", \"user\", \"role_access\", \"role\", \"api_procedure\", \"api\";";
        sqlx::query(sql)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[sqlx::test]
    async fn test_auth()
    {
        std::env::set_var("RUST_BACKTRACE", "1");

        let pool = get_connection_pool().await.unwrap();
        let auth = Auth::new_with_pool(pool);

        // truncate all auth database tables before test
        truncate_tables(&auth.pool).await.unwrap();

        // create new resource API
        let password_api = "Ap1_P4s5w0rd";
        let api_id1 = auth.create_api("Resource1", "localhost:9001", "RESOURCE", "", password_api).await.unwrap();
        let api_id2 = auth.create_api("Resource_2", "localhost:9002", "RESOURCE", "",  password_api).await.unwrap();

        // get newly created resource at the first of resource API list
        let apis = auth.list_api_by_category("RESOURCE").await.unwrap();
        let first_api = apis.into_iter().next().unwrap();
        let api = auth.read_api(api_id1).await.unwrap();

        // create new procedure for newly created resource API
        let proc_id1 = auth.create_procedure(api_id1, "ReadResourceData", "").await.unwrap();
        let proc_id2 = auth.create_procedure(api_id1, "CreateData", "").await.unwrap();
        let proc_id3 = auth.create_procedure(api_id1, "DeleteData", "").await.unwrap();
        let proc_id4 = auth.create_procedure(api_id2, "ReadConfig", "").await.unwrap();

        // get newly created procedure at the first of procedure list
        let procedures = auth.list_procedure_by_api(api_id1).await.unwrap();
        let procedure = procedures.into_iter().next().unwrap();

        assert_eq!(first_api.id, api_id1);
        assert_eq!(api.name, "Resource1");
        assert_eq!(api.address, "localhost:9001");
        assert_eq!(procedure.api_id, api.id);
        assert_eq!(procedure.name, "ReadResourceData");

        let (pub_key, priv_key) = (api.public_key, api.private_key);
        assert!(pub_key.len() > 64);
        assert!(priv_key.len() > 64);
        let hash = api.password;
        let parsed_hash = PasswordHash::new(hash.as_str()).unwrap();
        assert!(Argon2::default().verify_password(password_api.as_bytes(), &parsed_hash).is_ok());

        // create new role and add access to the procedure
        let role_id1 = auth.create_role(api_id1, "administrator", false, false, 900, 28800).await.unwrap();
        auth.add_role_access(role_id1, proc_id1).await.unwrap();
        auth.add_role_access(role_id1, proc_id2).await.unwrap();
        auth.add_role_access(role_id1, proc_id3).await.unwrap();
        let role_id2 = auth.create_role(api_id1, "user", true, false, 900, 604800).await.unwrap();
        auth.add_role_access(role_id2, proc_id1).await.unwrap();
        let role_id3 = auth.create_role(api_id2, "user", true, false, 900, 604800).await.unwrap();
        auth.add_role_access(role_id3, proc_id4).await.unwrap();

        // get role data
        let roles = auth.list_role_by_api(api_id1).await.unwrap();
        let first_role = roles.into_iter().next().unwrap();
        let role = auth.read_role(role_id1).await.unwrap();

        assert_eq!(first_role.id, role_id1);
        assert_eq!(role.name, "administrator");
        assert_eq!(role.multi, false);
        assert_eq!(role.ip_lock, false);
        assert_eq!(role.procedures, [proc_id1, proc_id2, proc_id3]);

        let access_key = role.access_key;
        assert_eq!(access_key.len(), 32);

        // update a resource API and its procedure and role
        let api_name = "Resource_1";
        let proc_name = "ReadData";
        let role_name = "admin";
        auth.update_api(api_id1, Some(api_name), None, None, Some("New resource api"), None, Some(())).await.unwrap();
        auth.update_procedure(proc_id1, Some(proc_name), Some("Read resource data")).await.unwrap();
        auth.update_role(role_id1, Some(role_name), None, Some(true), None, None).await.unwrap();

        // get updated resource API schema
        let api = auth.read_api_by_name(api_name).await.unwrap();
        let procedure = auth.read_procedure_by_name(api_id1, proc_name).await.unwrap();
        let role = auth.read_role_by_name(api_id1, role_name).await.unwrap();
        let api_procedure = api.procedures.into_iter().next().unwrap();

        assert_eq!(api.name, api_name);
        assert_eq!(api.description, "New resource api");
        assert_eq!(procedure.name, proc_name);
        assert_eq!(procedure.description, "Read resource data");
        assert_eq!(procedure.id, api_procedure.id);
        assert_eq!(role.name, role_name);
        assert_eq!(role.ip_lock, true);

        assert_ne!(api.public_key, pub_key);
        assert_ne!(api.private_key, priv_key);
        assert_ne!(role.access_key, access_key);

        // create new user and add associated roles
        let password_admin = "Adm1n_P4s5w0rd";
        let password_user = "Us3r_P4s5w0rd";
        let user_id1 = auth.create_user("administrator", "admin@mail.co", "+6281234567890", password_admin).await.unwrap();
        auth.add_user_role(user_id1, role_id1).await.unwrap();
        auth.add_user_role(user_id1, role_id3).await.unwrap();
        let user_id2 = auth.create_user("username", "user@mail.co", "+6281234567890", password_user).await.unwrap();
        auth.add_user_role(user_id2, role_id2).await.unwrap();
        auth.add_user_role(user_id2, role_id3).await.unwrap();

        // get user data
        let users = auth.list_user_by_role(role_id3).await.unwrap();
        let first_user = users.into_iter().next().unwrap();
        let user = auth.read_user(user_id1).await.unwrap();

        assert_eq!(first_user.id, user_id1);
        assert_eq!(user.name, "administrator");
        assert_eq!(user.email, "admin@mail.co");
        assert_eq!(user.phone, "+6281234567890");

        let (pub_key, priv_key) = (user.public_key, user.private_key);
        assert!(pub_key.len() > 64);
        assert!(priv_key.len() > 64);
        let hash = user.password;
        let parsed_hash = PasswordHash::new(hash.as_str()).unwrap();
        assert!(Argon2::default().verify_password(password_admin.as_bytes(), &parsed_hash).is_ok());

        // update user
        let password_new = "N3w_P4s5w0rd";
        auth.update_user(user_id2, None, None, None, Some(password_new), Some(())).await.unwrap();

        // get updated user
        let user = auth.read_user_by_name("username").await.unwrap();

        assert_ne!(user.password, hash);
        assert_ne!(user.public_key, pub_key);
        assert_ne!(user.private_key, priv_key);

        // create new access token and refresh token
        let expire1 = DateTime::from_str("2023-01-01T00:00:00Z").unwrap();
        let expire2 = DateTime::from_str("2023-01-01T12:00:00Z").unwrap();
        let auth_token = "rGKrHrDuWXt2CDbjmrt1SHbmea86wIQb";
        let (access_id1, _, auth_token1) = auth.create_access_token(user_id1, auth_token, expire1, &[192, 168, 0, 1]).await.unwrap();
        let access_id2 = access_id1 + 1;
        auth.create_auth_token(user_id1, expire2, &[192, 168, 0, 1], 1).await.unwrap();
        auth.create_access_token(user_id1, auth_token, expire1, &[]).await.unwrap();

        // get token data
        let access_token = auth.read_access_token(access_id2).await.unwrap();
        let auth_token = auth.list_auth_token(&auth_token1).await.unwrap()
            .into_iter().next().unwrap();
        let user_tokens = auth.list_token_by_user(user_id1).await.unwrap();

        assert_eq!(auth_token.user_id, user_id1);
        assert_eq!(auth_token.expire, expire1);
        assert_eq!(auth_token.ip, [192, 168, 0, 1]);
        assert_eq!(access_token.expire, expire2);
        assert_eq!(user_tokens.len(), 3);

        // update token
        let expire3 = DateTime::from_str("2023-01-01T18:00:00Z").unwrap();
        auth.update_access_token(access_id2, Some(expire3), None).await.unwrap();
        auth.update_auth_token(&auth_token1, Some(expire3), Some(&[192, 168, 0, 100])).await.unwrap();

        // get updated token
        let new_access_token = auth.read_access_token(access_id2).await.unwrap();
        let new_auth_token = auth.list_auth_token(&auth_token1).await.unwrap()
            .into_iter().next().unwrap();

        assert_ne!(new_access_token.refresh_token, access_token.refresh_token);
        assert_eq!(new_access_token.expire, expire3);
        assert_eq!(new_auth_token.expire, expire3);
        assert_eq!(new_auth_token.ip, [192, 168, 0, 100]);

        // try to delete resource API, procedure role and user without removing dependent item
        let try_role = auth.delete_role(role_id3).await;
        let try_proc = auth.delete_procedure(proc_id4).await;
        let try_api = auth.delete_api(api_id2).await;
        let try_user = auth.delete_user(user_id2).await;

        assert!(try_proc.is_err());
        assert!(try_role.is_err());
        assert!(try_api.is_err());
        assert!(try_user.is_err());

        // delete user and token
        auth.remove_user_role(user_id2, role_id2).await.unwrap();
        auth.remove_user_role(user_id2, role_id3).await.unwrap();
        auth.delete_user(user_id2).await.unwrap();
        auth.delete_token_by_user(user_id1).await.unwrap();

        // check if token and user already deleted
        let result_token = auth.read_access_token(access_id1).await;
        let result_user = auth.read_user(user_id2).await;

        assert!(result_token.is_err());
        assert!(result_user.is_err());

        // delete resource API, procedure, and role
        auth.remove_user_role(user_id1, role_id3).await.unwrap();
        auth.remove_role_access(role_id3, proc_id4).await.unwrap();
        auth.delete_role(role_id3).await.unwrap();
        auth.delete_procedure(proc_id4).await.unwrap();
        auth.delete_api(api_id2).await.unwrap();

        // check if resource API, procedure, and role already deleted
        let result_role = auth.read_role(role_id3).await;
        let result_proc = auth.read_procedure(proc_id4).await;
        let result_api = auth.read_api(api_id2).await;

        assert!(result_proc.is_err());
        assert!(result_role.is_err());
        assert!(result_api.is_err());
    }

}
