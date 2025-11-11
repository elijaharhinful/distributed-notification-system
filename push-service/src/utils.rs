use anyhow::{Error, Result};

use crate::{
    clients::redis::RedisClient,
    models::{message::NotificationMessage, status::IdempotencyStatus},
};

pub async fn process_message(payload: &str, redis_client: &mut RedisClient) -> Result<(), Error> {
    let message = serde_json::from_str::<NotificationMessage>(payload)?;

    match redis_client
        .check_idempotency(&message.idempotency_key)
        .await
    {
        Ok(IdempotencyStatus::Sent) => {
            println!("Message already processed, skipping.");
            return Ok(());
        }
        Ok(IdempotencyStatus::Processing) => {
            println!("Message is being processed elsewhere, skipping.");
            return Ok(());
        }
        _ => {}
    }

    println!("Processing notification message");

    redis_client
        .mark_as_processing(&message.idempotency_key)
        .await?;

    Ok(())
}
