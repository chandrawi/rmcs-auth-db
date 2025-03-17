use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
// use sqlx::types::chrono::{DateTime, Utc};
use rmcs_auth_db::Auth;
// use rmcs_resource_db::DataValue;

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect("postgres://postgres:Gundala123@127.0.0.1:5432/test_rmcs_auth")
        .await;
    let auth = Auth::new_with_pool(pool.unwrap());

    let profile_id = 1;
    let profile = auth.read_user_profile(profile_id).await.unwrap();
    println!("{:?}", profile);

    let user_id = Uuid::try_parse("0d50f483-45dc-4074-89fc-d02b90fb568c").unwrap();
    // let value = DataValue::String("085321251627".to_string());
    // let profile_id = auth.create_user_profile(user_id, "phone", value).await.unwrap();
    // println!("{:?}", profile_id);
    auth.swap_user_profile(user_id, "phone", 0, 1).await.unwrap();
}
