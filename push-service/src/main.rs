use anyhow::{Error, Result};
use push_service::{
    clients::rbmq::{self, RabbitMqClient},
    config::Config,
};

use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::load()?;

    println!("Configuration validated. Worker is ready to start.");

    let rabbitmq_client = RabbitMqClient::connect(&config).await?;
    let mut consumer = rabbitmq_client.create_consumer().await?;

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let delivery_tag = delivery.delivery_tag;

                match rbmq::parse_message(&delivery.data) {
                    Ok(message) => {
                        println!("Received message:");
                        println!("  - Trace ID: {}", message.trace_id);
                        println!("  - User ID: {}", message.user_id);
                        println!("  - Template: {}", message.template_code);
                        println!("  - Recipient: {}", message.recipient);

                        rabbitmq_client.acknowledge(delivery_tag).await?;
                    }
                    Err(_) => {
                        eprintln!("Failed to parse message");
                        eprintln!("Rejecting invalid message");

                        rabbitmq_client.reject(delivery_tag, false).await?;
                    }
                }
            }
            Err(_) => {
                eprintln!("Error receiving message");
            }
        }
    }

    println!("Consumer closed, shutting down");

    Ok(())
}
