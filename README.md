# form

An experimental object relational mapper built on top of sqlx

# Example

```rs
// form = { features = ["mysql"] }
use form::{PoolOptions, Queryable};

#[derive(Debug, Default, Queryable)]
struct User {
  #[serial]
  #[primary_key]
  id: i32,
  #[unique]
  #[sql_type(varchar(16))]
  tag: String,
  #[unique]
  email_address: String,
}

#[tokio::main]
async fn main() -> form::Result<()> {
  let pool = PoolOptions::new()
    .max_connections(5)
    .connect("mysql://user:pass@host:port/schema")
    .await?;

  let query_result = User{
    tag: "user1".to_string(),
    email_address: "user1@example.com".to_string(),
    ..Default::default()
  }.create(&pool).await?;

  let created_user = User::find_id(query_result.last_insert_id() as _, &pool).await?;
  // Prints `User { id: LAST_INSERT_ID, tag: "user1", email_address: "user1@example.com" }`
  println!("{:?}", created_user);

  Ok(())
}
```

# Features:
- mysql database support
- fetch/create/update

# TODO:
- DSL for pgsql and sqlite support
- proper generics lol
- automatic schema migration
- relationships
- a little cleanup
