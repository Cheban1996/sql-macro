<!-- markdownlint-disable MD033 -->

# Crate sql-macros

`sql-macros` - it's simple lib for generate sql query for select, select_many, select_all, insert, update, delete

## Install

```toml
# Cargo.toml
[dependencies]
sql-macros = { version = "0.1" }
```

## Usage

## Table name

```rust
use sql_macros::SqlSelect;

#[derive(SqlSelect)]
pub struct User {
    #[table(select)]
    pub id: i32,
    pub email: String,
}
```

Table name will be generated as `users`

If you need use special name use `#[table(name = users)]`

## Select one

```rust
use sql_macros::SqlSelect;

#[derive(SqlSelect)]
pub struct User {
    #[table(select)]
    pub id: i32,
    pub email: String,
}

pub async fn get_by_id(pool: &sqlx::PgPool, id: i32) -> Result<Option<User>, sqlx::Error> {
    let user = User::select_by_id(pool, id).await?;
    Ok(user)
}
```

<details>
    <summary>View generated code</summary>

```rust
impl User {
    #[doc = "SELECT id, email FROM users WHERE id=$1"]
    pub async fn select_by_id(pool: &sqlx::PgPool, id: i32) -> Result<Option<User>, sqlx::Error> {
        let object = sqlx::query_as!(User, "SELECT id, email FROM users WHERE id=$1", id)
            .fetch_optional(pool)
            .await?;
        Ok(object)
    }
}
```

</details>

## Select all

```rust
use sql_macros::SqlSelectAll;

#[derive(SqlSelectAll)]
pub struct User {
    pub id: i32,
    pub email: String,
}
pub async fn get_all_users(pool: &sqlx::PgPool) -> Result<Vec<User>, sqlx::Error> {
    let users = User::select_all(pool).await?;
    Ok(users)
}
```

<details>
    <summary>View generated code</summary>

```rust
impl User {
    #[doc = "SELECT id, email FROM users"]
    pub async fn select_all(pool: &sqlx::PgPool) -> Result<Vec<User>, sqlx::Error> {
        let object = sqlx::query_as!(User, "SELECT id, email FROM users")
            .fetch_all(pool)
            .await?;
        Ok(object)
    }
}
```

</details>

## Select many

```rust
use sql_macros::SqlSelectMany;

#[derive(SqlSelectMany)]
pub struct User {
    pub id: i32,
    pub email: String,
    #[table(select_many)]
    pub is_removed: bool,
}
pub async fn get_by_removed(pool: &sqlx::PgPool, is_removed: bool) -> Result<Vec<User>, sqlx::Error> {
    let users = User::select_many(pool, is_removed).await?;
    Ok(users)
}
```

<details>
    <summary>View generated code</summary>

```rust
impl User {
    #[doc = "SELECT id, email, is_removed FROM users WHERE is_removed=$1"]
    pub async fn select_many_by_is_removed(
        pool: &sqlx::PgPool,
        is_removed: bool,
    ) -> Result<Vec<User>, sqlx::Error> {
        let object = sqlx::query_as!(
            User,
            "SELECT id, email, is_removed FROM users WHERE is_removed=$1",
            is_removed
        )
        .fetch_all(pool)
        .await?;
        Ok(object)
    }
}
```

</details>

## Insert

### Insert without returning

```rust
use sql_macros::SqlInsert;

#[derive(Debug, SqlInsert)]
#[table(name = users)]
pub struct CreateUser {
    pub email: String,
}

pub async fn create(pool: &sqlx::PgPool, data: &CreateUser) -> Result<u64, sqlx::Error> {
    let query_result = data.insert(pool).await?;
    Ok(query_result.rows_affected())
}
```

<details>
    <summary>View generated code</summary>

```rust
impl CreateUser {
    #[doc = "INSERT INTO users (email) VALUES ($1)"]
    pub async fn insert(
        &self,
        conn: &mut sqlx::PgConnection,
    ) -> Result<sqlx::any::AnyQueryResult, sqlx::Error> {
        let query_result = sqlx::query!("INSERT INTO users (email) VALUES ($1)", &self.email)
            .execute(&mut *conn)
            .await?;
        Ok(query_result.into())
    }
}
```

</details>

### Insert with returning

```rust
use sql_macros::SqlInsert;

#[derive(Debug, SqlInsert)]
#[table(name = users, return_type = User)]
pub struct CreateUser {
    pub email: String,
}

pub async fn create(pool: &sqlx::PgPool, data: &CreateUser) -> Result<User, sqlx::Error> {
    let user = data.insert(pool).await?;
    Ok(user)
}
```

<details>
    <summary>View generated code</summary>

```rust
impl CreateUser {
    #[doc = "INSERT INTO users (email) VALUES ($1) RETURNING *"]
    pub async fn insert(&self, conn: &mut sqlx::PgConnection) -> Result<User, sqlx::Error> {
        let object = sqlx::query_as!(
            User,
            "INSERT INTO users (email) VALUES ($1) RETURNING *",
            &self.email
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(object)
    }
}

```

</details>

### Insert with returning fields

```rust
use sql_macros::SqlInsert;

#[derive(sqlx::FromRow)]
struct CreateUserResponse {
    pub id: i32,
}

#[derive(Debug, SqlInsert)]
#[table(name = users, return_type = CreateUserResponse, return_fields = "id")]
pub struct CreateUser {
    pub email: String,
}

pub async fn create(pool: &sqlx::PgPool, data: &CreateUser) -> Result<CreateUserResponse, sqlx::Error> {
    let user = data.insert(pool).await?;
    Ok(user)
}
```

<details>
    <summary>View generated code</summary>

```rust
impl CreateUser {
    #[doc = "INSERT INTO users (email) VALUES ($1) RETURNING id"]
    pub async fn insert(&self, conn: &mut sqlx::PgConnection) -> Result<CreateUserResponse, sqlx::Error> {
        let object = sqlx::query_as!(
            CreateUserResponse,
            "INSERT INTO users (email) VALUES ($1) RETURNING id",
            &self.email
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(object)
    }
}

```

</details>

## Update

### Update without returning

It just return query result (see `sqlx::any::AnyQueryResult`)

```rust
use sql_macros::SqlUpdate;

#[derive(SqlUpdate)]
#[table(name = users)]
pub struct UpdateUser {
    #[table(update)]
    pub id: i32,
    pub email: String,
}

pub async fn update(pool: &sqlx::PgPool, data: &UpdateUser) -> Result<u64, sqlx::Error> {
    let query_result = data.update(pool).await?;
    Ok(query_result.rows_affected())
}
```

<details>
    <summary>View generated code</summary>

```rust
impl UpdateUser {
    #[doc = "UPDATE users SET email=$1 WHERE id=$2"]
    pub async fn update(
        &self,
        conn: &mut sqlx::PgConnection,
    ) -> Result<sqlx::any::AnyQueryResult, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE users SET email=$1 WHERE id=$2",
           &self.email,
           &self.id
       )
       .execute(&mut *conn)
       .await?;
       Ok(result.into())
   }
}
```

</details>

### Update with returning type

```rust
use sql_macros::SqlUpdate;

#[derive(SqlUpdate)]
#[table(name = users, return_type = User)]
pub struct UpdateUser {
    #[table(update)]
    pub id: i32,
    pub email: String,
}

pub async fn update(pool: &sqlx::PgPool, data: &UpdateUser) -> Result<User, sqlx::Error> {
    let user = data.update(pool).await?;
    Ok(user)
}
```

<details>
    <summary>View generated code</summary>

```rust
impl UpdateUser {
   #[doc = "UPDATE users SET email=$1 WHERE id=$2 RETURNING *"]
   pub async fn update(&self, conn: &mut sqlx::PgConnection) -> Result<User, sqlx::Error> {
       let object = sqlx::query_as!(
           User,
           "UPDATE users SET email=$1 WHERE id=$2 RETURNING *",
            &self.email,
            &self.id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(object)
    }
}
```

</details>

### Update with returning fields

```rust
use sql_macros::SqlUpdate;

#[derive(sqlx::FromRow)]
struct UpdateUserResponse {
    pub id: i32,
}

#[derive(SqlUpdate)]
#[table(name = users, return_type = UpdateUserResponse, return_fields = "id")]
pub struct UpdateUser {
    #[table(update)]
    pub id: i32,
    pub email: String,
}

pub async fn update(pool: &sqlx::PgPool, data: &UpdateUser) -> Result<UpdateUserResponse, sqlx::Error> {
    let res = data.update(pool).await?;
    Ok(res)
}
```

<details>
    <summary>View generated code</summary>

```rust
impl UpdateUser {
   #[doc = "UPDATE users SET email=$1 WHERE id=$2 RETURNING id"]
   pub async fn update(&self, conn: &mut sqlx::PgConnection) -> Result<UpdateUserResponse, sqlx::Error> {
       let object = sqlx::query_as!(
           UpdateUserResponse,
           "UPDATE users SET email=$1 WHERE id=$2 RETURNING id",
            &self.email,
            &self.id
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(object)
    }
}
```

</details>

### Update with special columns

```rust
use sql_macros::SqlUpdate;

#[derive(SqlUpdate)]
#[table(name = users, spec_columns = "updated_at=NOW()")]
pub struct UpdateUser {
    #[table(update)]
    pub id: i32,
    pub email: String,
}

pub async fn update(pool: &sqlx::PgPool, data: &UpdateUser) -> Result<u64, sqlx::Error> {
    let query_result = data.update(pool).await?;
    Ok(query_result.rows_affected())
}
```

<details>
  <summary>View generated code</summary>

```rust
impl UpdateUser {
   #[doc = "UPDATE users SET email=$1, updated_at=NOW() WHERE id=$2"]
   pub async fn update(
       &self,
       conn: &mut sqlx::PgConnection,
   ) -> Result<sqlx::any::AnyQueryResult, sqlx::Error> {
       let result = sqlx::query!(
            "UPDATE users SET email=$1, updated_at=NOW() WHERE id=$2",
            &self.email,
            &self.id
        )
        .execute(&mut *conn)
        .await?;
        Ok(result.into())
    }
}
```

</details>

## Delete

```rust
use sql_macros::SqlDelete;

#[derive(SqlDelete)]
pub struct User {
    #[table(delete)]
    pub id: i32,
}

async fn delete(conn: &mut sqlx::PgConnection, id: i32) -> Result<u64, sqlx::Error> {
    let result = User::delete(pool, id).await?;
    Ok(result.rows_affected())
}
```

<details>
    <summary>View generated code</summary>

```rust
impl User {
    #[doc = "DELETE FROM users WHERE id=$1"]
    pub async fn delete_by_id(
        conn: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<sqlx::any::AnyQueryResult, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM users WHERE id=$1", id)
            .execute(&mut *conn)
            .await?;
        Ok(result.into())
    }
}
```

</details>

## Generate methods with many fields

```rust
use sql_macros::SqlSelect;

#[derive(SqlSelect)]
#[table(select = get_active_user(is_active, is_removed))]
#[table(select_many = get_user_by_removed(is_active, is_removed))]
#[table(delete = delete_user(is_active, is_removed))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub is_active: bool,
    pub is_removed: bool,
}
```

Worked with select, select_many, delete

Self methods can't named as select, select_many, delete

`#[table(update = update_user(email, other_field))]` - now is not supported

## Select with enum

```rust
use sql_macros::SqlSelect;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "role", rename_all = "snake_case")]
pub enum Role {
    Admin,
    User,
    SuperAdmin,
}

#[derive(SqlSelect)]
pub struct User {
    #[table(select)]
    pub id: i32,
    pub email: String,
    #[table(as_type = "role!: Role")]
    pub role: Role,
}

pub async fn get_by_id(pool: &sqlx::PgPool, id: i32) -> Result<Option<User>, sqlx::Error> {
    let user = User::select_by_id(pool, id).await?;
    Ok(user)
}
```

<details>
    <summary>View generated code</summary>

```rust
impl User {
    #[doc = "SELECT id, email, role AS \"role!: Role\" FROM users WHERE id=$1"]
    pub async fn select_by_id(pool: &sqlx::PgPool, id: i32) -> Result<Option<User>, sqlx::Error> {
        let object = sqlx::query_as!(
            User,
            "SELECT id, email, role AS \"role!: Role\" FROM users WHERE id=$1",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(object)
    }
}
```

</details>

## Attention

If you use `return_type` and you're table has a column with type enum - it will don't work because we can't get of type of return type since we have in macros token(it's just a string) not a type.

```rust
#[derive(Debug, SqlInsert)]
#[table(name = users, return_type = User)]
pub struct CreateUser {
    pub email: String,
}
```

If table User has a column with type enum `SqlInsert` or `SqlUpdate` will not be work
