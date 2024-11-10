#[cfg(test)]
mod tests {
    use sqlx::{Pool, Error};
    use sqlx::postgres::{Postgres, PgPoolOptions};
    use sqlx::types::chrono::DateTime;
    use uuid::Uuid;
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use rmcs_auth_db::Auth;
    use rmcs_auth_db::utility::generate_access_key;
    use rmcs_auth_db::ProfileMode::*;
    use rmcs_resource_db::{DataType::*, DataValue::*};

    async fn get_connection_pool() -> Result<Pool<Postgres>, Error>
    {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL_AUTH_TEST").unwrap();
        PgPoolOptions::new()
            .max_connections(100)
            .connect(&url)
            .await
    }

    async fn truncate_tables(pool: &Pool<Postgres>) -> Result<(), Error>
    {
        let sql = "TRUNCATE TABLE \"profile_user\", \"profile_role\", \"token\", \"user_role\", \"user\", \"role_access\", \"role\", \"api_procedure\", \"api\";";
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
        let access_key = generate_access_key();
        let api_id1 = auth.create_api(Uuid::new_v4(), "Resource1", "localhost:9001", "RESOURCE", "", password_api, &access_key).await.unwrap();
        let api_id2 = auth.create_api(Uuid::new_v4(), "Resource_2", "localhost:9002", "RESOURCE", "",  password_api, &access_key).await.unwrap();

        // create new procedure for newly created resource API
        let proc_id1 = auth.create_procedure(Uuid::new_v4(), api_id1, "ReadResourceData", "").await.unwrap();
        let proc_id2 = auth.create_procedure(Uuid::new_v4(), api_id1, "CreateData", "").await.unwrap();
        let proc_id3 = auth.create_procedure(Uuid::new_v4(), api_id1, "DeleteData", "").await.unwrap();
        let proc_id4 = auth.create_procedure(Uuid::new_v4(), api_id2, "ReadConfig", "").await.unwrap();

        // get newly created resource at the first of resource API list
        let apis = auth.list_api_by_category("RESOURCE").await.unwrap();
        let api_ids: Vec<Uuid> = apis.iter().map(|e| e.id).collect();
        let api = auth.read_api(api_id1).await.unwrap();
        let api_proc_ids: Vec<Uuid> = api.procedures.iter().map(|e| e.id).collect();

        // get newly created procedure at the first of procedure list
        let procedures = auth.list_procedure_by_api(api_id1).await.unwrap();
        let proc_ids: Vec<Uuid> = procedures.iter().map(|e| e.id).collect();

        assert_eq!(api.name, "Resource1");
        assert_eq!(api.address, "localhost:9001");
        assert!(api_ids.contains(&api_id1));
        assert!(proc_ids.contains(&proc_id1));
        assert_eq!(api_proc_ids, proc_ids);

        let hash = api.password;
        let parsed_hash = PasswordHash::new(hash.as_str()).unwrap();
        assert!(Argon2::default().verify_password(password_api.as_bytes(), &parsed_hash).is_ok());

        // create new role and add access to the procedure
        let role_id1 = auth.create_role(Uuid::new_v4(), api_id1, "administrator", false, false, 900, 28800).await.unwrap();
        auth.add_role_access(role_id1, proc_id1).await.unwrap();
        auth.add_role_access(role_id1, proc_id2).await.unwrap();
        auth.add_role_access(role_id1, proc_id3).await.unwrap();
        let role_id2 = auth.create_role(Uuid::new_v4(), api_id1, "user", true, false, 900, 604800).await.unwrap();
        auth.add_role_access(role_id2, proc_id1).await.unwrap();
        let role_id3 = auth.create_role(Uuid::new_v4(), api_id2, "user", true, false, 900, 604800).await.unwrap();
        auth.add_role_access(role_id3, proc_id4).await.unwrap();

        // get role data
        let roles = auth.list_role_by_api(api_id1).await.unwrap();
        let role_ids: Vec<Uuid> = roles.iter().map(|e| e.id).collect();
        let role = auth.read_role(role_id1).await.unwrap();

        assert!(role_ids.contains(&role_id1));
        assert_eq!(role.name, "administrator");
        assert_eq!(role.multi, false);
        assert_eq!(role.ip_lock, false);
        assert!(role.procedures.contains(&proc_id1));
        assert!(role.procedures.contains(&proc_id2));
        assert!(role.procedures.contains(&proc_id3));

        let access_key = role.access_key;
        assert_eq!(access_key.len(), 32);

        // update a resource API and its procedure and role
        let api_name = "Resource_1";
        let proc_name = "ReadData";
        let role_name = "admin";
        let access_key_new = generate_access_key();
        auth.update_api(api_id1, Some(api_name), None, None, Some("New resource api"), None, Some(&access_key_new)).await.unwrap();
        auth.update_procedure(proc_id1, Some(proc_name), Some("Read resource data")).await.unwrap();
        auth.update_role(role_id1, Some(role_name), None, Some(true), None, None).await.unwrap();

        // get updated resource API schema
        let api = auth.read_api_by_name(api_name).await.unwrap();
        let procedure = auth.read_procedure_by_name(api_id1, proc_name).await.unwrap();
        let role = auth.read_role_by_name(api_id1, role_name).await.unwrap();

        assert_eq!(api.name, api_name);
        assert_eq!(api.description, "New resource api");
        assert_eq!(procedure.name, proc_name);
        assert_eq!(procedure.description, "Read resource data");
        assert_eq!(role.name, role_name);
        assert_eq!(role.ip_lock, true);
        assert_ne!(role.access_key, access_key);

        // create new user and add associated roles
        let password_admin = "Adm1n_P4s5w0rd";
        let password_user = "Us3r_P4s5w0rd";
        let user_id1 = auth.create_user(Uuid::new_v4(), "administrator", "admin@mail.co", "+6281234567890", password_admin).await.unwrap();
        auth.add_user_role(user_id1, role_id1).await.unwrap();
        auth.add_user_role(user_id1, role_id3).await.unwrap();
        let user_id2 = auth.create_user(Uuid::new_v4(), "username", "user@mail.co", "+6281234567890", password_user).await.unwrap();
        auth.add_user_role(user_id2, role_id2).await.unwrap();
        auth.add_user_role(user_id2, role_id3).await.unwrap();

        // get user data
        let users = auth.list_user_by_role(role_id3).await.unwrap();
        let user_ids: Vec<Uuid> = users.iter().map(|e| e.id).collect();
        let user = auth.read_user(user_id1).await.unwrap();

        assert!(user_ids.contains(&user_id1));
        assert_eq!(user.name, "administrator");
        assert_eq!(user.email, "admin@mail.co");
        assert_eq!(user.phone, "+6281234567890");

        let hash = user.password;
        let parsed_hash = PasswordHash::new(hash.as_str()).unwrap();
        assert!(Argon2::default().verify_password(password_admin.as_bytes(), &parsed_hash).is_ok());

        // update user
        let password_new = "N3w_P4s5w0rd";
        auth.update_user(user_id2, None, None, None, Some(password_new)).await.unwrap();

        // get updated user
        let user = auth.read_user_by_name("username").await.unwrap();

        assert_ne!(user.password, hash);

        // create role and user profile
        let profile_role_id1 = auth.create_role_profile(role_id1, "name", StringT, SingleRequired).await.unwrap();
        let profile_role_id2 = auth.create_role_profile(role_id1, "age", U16T, SingleOptional).await.unwrap();
        let profile_user_id1 = auth.create_user_profile(user_id1, "name", String("john doe".to_owned())).await.unwrap();
        let profile_user_id2 = auth.create_user_profile(user_id1, "age", U16(20)).await.unwrap();

        // read role and user profile
        let profile_role1 = auth.read_role_profile(profile_role_id1).await.unwrap();
        let profile_role2 = auth.read_role_profile(profile_role_id2).await.unwrap();
        let profile_user1 = auth.read_user_profile(profile_user_id1).await.unwrap();
        let profile_users = auth.list_user_profile_by_user(user_id1).await.unwrap();

        assert_eq!(profile_role1.name, "name");
        assert_eq!(profile_role2.mode, SingleOptional);
        assert_eq!(profile_user1.value, String("john doe".to_owned()));
        assert!(profile_users.contains(&profile_user1));

        // update user profile
        auth.update_user_profile(profile_user_id2, None, Some(U16(21))).await.unwrap();
        let profile_user2 = auth.read_user_profile(profile_user_id2).await.unwrap();

        assert_eq!(profile_user2.value, U16(21));

        // create new access token and refresh token
        let expire1 = DateTime::parse_from_str("2023-01-01 00:00:00 +0000", "%Y-%m-%d %H:%M:%S %z").unwrap().into();
        let expire2 = DateTime::parse_from_str("2023-01-01 12:00:00 +0000", "%Y-%m-%d %H:%M:%S %z").unwrap().into();
        let auth_token = "rGKrHrDuWXt2CDbjmrt1SHbmea86wIQb";
        let (access_id1, _, auth_token1) = auth.create_access_token(user_id1, auth_token, expire1, &[192, 168, 0, 1]).await.unwrap();
        let access_id2 = access_id1 + 1;
        auth.create_auth_token(user_id1, expire2, &[192, 168, 0, 1], 1).await.unwrap();
        auth.create_access_token(user_id1, auth_token, expire1, &[]).await.unwrap();

        // get token data
        let access_token = auth.read_access_token(access_id2).await.unwrap();
        let auth_tokens = auth.list_auth_token(&auth_token1).await.unwrap();
        let auth_token = auth_tokens.iter().filter(|x| x.auth_token == auth_token1).next().unwrap();
        let user_tokens = auth.list_token_by_user(user_id1).await.unwrap();

        assert_eq!(auth_token.user_id, user_id1);
        assert_eq!(auth_token.expire, expire1);
        assert_eq!(auth_token.ip, [192, 168, 0, 1]);
        assert_eq!(access_token.expire, expire2);
        assert_eq!(user_tokens.len(), 3);

        // update token
        let expire3 = DateTime::parse_from_str("2023-01-01 18:00:00 +0000", "%Y-%m-%d %H:%M:%S %z").unwrap().into();
        auth.update_access_token(access_id2, Some(expire3), None).await.unwrap();
        auth.update_auth_token(&auth_token1, Some(expire3), Some(&[192, 168, 0, 100])).await.unwrap();

        // get updated token
        let new_access_token = auth.read_access_token(access_id2).await.unwrap();
        let new_auth_tokens = auth.list_auth_token(&auth_token1).await.unwrap();
        let new_auth_token = new_auth_tokens.iter().filter(|x| x.auth_token == auth_token1).next().unwrap();

        assert_ne!(new_access_token.refresh_token, access_token.refresh_token);
        assert_eq!(new_access_token.expire, expire3);
        assert_eq!(new_auth_token.expire, expire3);
        assert_eq!(new_auth_token.ip, [192, 168, 0, 100]);

        // delete role and user profile
        auth.delete_user_profile(profile_user_id1).await.unwrap();
        auth.delete_role_profile(profile_role_id1).await.unwrap();

        // check if role and user profile already deleted
        let result_profile_user = auth.read_role_profile(profile_user_id1).await;
        let result_profile_role = auth.read_role_profile(profile_role_id1).await;

        assert!(result_profile_user.is_err());
        assert!(result_profile_role.is_err());

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
