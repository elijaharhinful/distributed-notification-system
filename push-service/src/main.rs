use std::sync::Arc;

use anyhow::{Error, Result};
use chrono::{SecondsFormat, Utc};
use push_service::{
    clients::{
        circuit_breaker::CircuitBreaker, fcm::FcmClient, rbmq::RabbitMqClient, redis::RedisClient,
        template::TemplateServiceClient,
    },
    config::Config,
    models::message::{DlqMessage, NotificationMessage},
    utils::process_message,
};

use futures_util::StreamExt;
use tokio::sync::{Mutex, Semaphore};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::load()?;

    println!("Configuration validated. Worker is ready to start.");

    let rabbitmq_client = Arc::new(RabbitMqClient::connect(&config).await?);
    let mut consumer = rabbitmq_client.create_consumer().await?;

    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis_conn = redis_client.get_multiplexed_async_connection().await?;

    let fcm_circuit_breaker = CircuitBreaker::new(
        "fcm".to_string(),
        redis_conn.clone(),
        config.circuit_breaker_config(),
    );

    let template_circuit_breaker = CircuitBreaker::new(
        "template_service".to_string(),
        redis_conn,
        config.circuit_breaker_config(),
    );

    let template_service_client = Arc::new(Mutex::new(
        TemplateServiceClient::new(&config, template_circuit_breaker).await?,
    ));

    let fcm_client = Arc::new(Mutex::new(
        FcmClient::new(&config, fcm_circuit_breaker).await,
    ));

    let semaphore = Arc::new(Semaphore::new(config.worker_concurrency));

    println!(
        "Worker started with concurrency limit: {}",
        config.worker_concurrency
    );

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let delivery_tag = delivery.delivery_tag;
                let payload = String::from_utf8_lossy(&delivery.data).to_string();

                let rabbitmq_client = Arc::clone(&rabbitmq_client);
                let template_service_client = Arc::clone(&template_service_client);
                let fcm_client = Arc::clone(&fcm_client);
                let semaphore = Arc::clone(&semaphore);
                let config = config.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    let mut redis_client = match RedisClient::connect(&config).await {
                        Ok(client) => client,
                        Err(e) => {
                            eprintln!("Failed to connect to Redis: {}", e);
                            if let Err(reject_err) =
                                rabbitmq_client.reject(delivery_tag, true).await
                            {
                                eprintln!("Failed to requeue message: {}", reject_err);
                            }
                            return;
                        }
                    };

                    let mut template_client = template_service_client.lock().await;
                    let mut fcm = fcm_client.lock().await;

                    match process_message(
                        &payload,
                        &mut redis_client,
                        &mut template_client,
                        &mut fcm,
                    )
                    .await
                    {
                        Ok(_) => {
                            println!("Message processed successfully");
                            if let Err(ack_err) = rabbitmq_client.acknowledge(delivery_tag).await {
                                eprintln!("Failed to acknowledge message: {}", ack_err);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to process message: {}", e);

                            match serde_json::from_str::<NotificationMessage>(&payload) {
                                Ok(original_message) => {
                                    let dlq_message = DlqMessage {
                                        original_message,
                                        failure_reason: e.to_string(),
                                        failed_at: Utc::now()
                                            .to_rfc3339_opts(SecondsFormat::Millis, true),
                                    };

                                    if let Err(dlq_err) =
                                        rabbitmq_client.publish_to_dlq(&dlq_message).await
                                    {
                                        eprintln!("Failed to publish to DLQ: {}", dlq_err);
                                    }
                                }
                                Err(parse_err) => {
                                    eprintln!(
                                        "Cannot parse message as JSON: {}. Raw payload: {}",
                                        parse_err, payload
                                    );
                                }
                            }

                            if let Err(reject_err) =
                                rabbitmq_client.reject(delivery_tag, false).await
                            {
                                eprintln!("Failed to reject message: {}", reject_err);
                            }
                        }
                    }
                });
            }
            Err(_) => {
                eprintln!("Error receiving message");
            }
        }
    }

    println!("Consumer closed, shutting down");

    Ok(())
}
