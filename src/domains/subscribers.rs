use crate::domains::subscriber::NewSubscriber;
use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;

#[tracing::instrument(name = "Saving a new subscriber", skip(pool))]
pub async fn insert_subscriber(
    subscriber: &NewSubscriber,
    conf_token: &str,
    pool: &PgPool,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, confirmation_token, subscribed_at, status)
    VALUES ($1, $2, $3, $4, $5, 'pending_confirmation')
            "#,
        Uuid::new_v4(),
        subscriber.email(),
        subscriber.name(),
        conf_token,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Error from saving new subscriber {:?}", e);
        e
    })?;
    Ok(())
}

pub async fn confirm_subscriber(token: &str, pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        update subscriptions set status='confirmed' where confirmation_token = $1
        "#,
        token,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(())
}

pub async fn get_confirmed_subscriber_emails<'a>(pool: &PgPool) -> sqlx::Result<Vec<String>> {
    let res = sqlx::query!(r#"select email from subscriptions where status = 'confirmed'"#)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|s| s.email)
        .collect::<Vec<String>>();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    fn saving_subscriber(pool: PgPool) {
        let new_subscriber = NewSubscriber::parse("tom", "tom@gmail.com").unwrap();
        let _ = insert_subscriber(&new_subscriber, "conf_token", &pool).await;

        let res = sqlx::query!("select * from subscriptions where email = 'tom@gmail.com'")
            .fetch_one(&pool)
            .await
            .expect("Failed to select subscriber");
        assert_eq!(res.name, "tom");
        assert_eq!(res.status, "pending_confirmation");
        assert_eq!(res.confirmation_token, "conf_token");
    }

    #[sqlx::test]
    fn get_confirmed_subscriber_emails(pool: PgPool) {
        let conf_subscriber = NewSubscriber::parse("tom", "tom@gmail.com").unwrap();
        let not_conf_subscriber = NewSubscriber::parse("petr", "petr@gmail.com").unwrap();
        let _ = insert_subscriber(&conf_subscriber, "conf_token", &pool)
            .await
            .unwrap();
        let _ = insert_subscriber(&not_conf_subscriber, "conf_token2", &pool)
            .await
            .unwrap();

        let _ = confirm_subscriber("conf_token", &pool).await.unwrap();

        let res = sqlx::query!("select * from subscriptions where status = 'confirmed'")
            .fetch_all(&pool)
            .await
            .expect("Failed to select subscriber");
        assert_eq!(res.len(), 1);
        assert_eq!(&res[0].name, conf_subscriber.name());
    }
}
