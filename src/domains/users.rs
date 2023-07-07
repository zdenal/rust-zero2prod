use argonautica::Hasher;
use sqlx::{query_as, FromRow, PgPool};
use validator::{Validate, ValidationErrors};

#[derive(serde::Serialize, FromRow, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

//fn is_valid_password(password: &Secret<String>) -> Result<(), ValidationError> {
//if password.expose_secret().len() > 6 {
//Ok(())
//} else {
//Err(ValidationError::new("length"))
//}
//}

#[derive(Validate)]
pub struct NewUser {
    #[validate(length(max = 256, min = 1))]
    name: String,
    #[validate(length(min = 6))]
    password: String,
    password_hash: String,
}

impl NewUser {
    pub fn parse(
        name: &str,
        password: &str,
        hash_secret: &str,
    ) -> Result<NewUser, ValidationErrors> {
        let password_hash = Hasher::default()
            .with_password(password)
            .with_secret_key(hash_secret)
            .hash()
            .unwrap();
        let u = Self {
            name: name.to_string(),
            password: password.to_string(),
            password_hash,
        };
        u.validate().map(|_| u)
    }
}
pub async fn add_user(user: NewUser, pool: &PgPool) -> sqlx::Result<User> {
    query_as::<_, User>(
        "insert into users (username, password_hash) values ($1, $2) returning id, username, password_hash",
    )
    .bind(user.name)
    .bind(user.password_hash)
    .fetch_one(pool)
    .await
}

pub async fn get_by_name(name: &str, pool: &PgPool) -> Option<User> {
    query_as::<_, User>("select * from users where username = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn get_all(pool: &PgPool) -> sqlx::Result<Vec<User>> {
    query_as("select * from users").fetch_all(pool).await
}

#[cfg(test)]
mod tests {
    use claims::{assert_none, assert_ok, assert_some};

    use super::*;

    #[sqlx::test]
    fn saving_and_getting_user(pool: PgPool) {
        let new_user = NewUser::parse("tom", "password", "hash_secret").unwrap();
        assert_ok!(add_user(new_user, &pool).await);
        assert_some!(get_by_name("tom", &pool).await);
        assert_none!(get_by_name("john", &pool).await);
    }
}
