use super::{MessageQueue, MessageQueueConfig, ReplicationEvent};
use log::{error, info};
use std::{
    process,
    sync::{mpsc, Arc},
};
use tokio::{runtime::Handle, sync::oneshot};
use utils::messaging_system::publisher::CommonPublisher;

pub async fn replicate_db_events(
    config: MessageQueueConfig,
    recv: mpsc::Receiver<ReplicationEvent>,
    runtime: Handle,
    mut kill_signal: oneshot::Receiver<()>,
) {
    let producer = match &config.queue {
        MessageQueue::Kafka(kafka) => CommonPublisher::new_kafka(&kafka.brokers).await,
        MessageQueue::Amqp(amqp) => CommonPublisher::new_amqp(&amqp.connection_string).await,
    }
    .unwrap_or_else(|_e| {
        error!("Fatal error, synchronization channel cannot be created.");
        process::abort();
    });

    let producer = Arc::new(producer);
    loop {
        let event = recv.recv().unwrap_or_else(|_e| {
            error!("Fatal error, synchronization channel closed.");
            process::abort();
        });
        if kill_signal.try_recv().is_ok() {
            info!("Master replication disabled");
            return;
        };

        send_messages_to_kafka(producer.clone(), config.topic_or_exchange.clone(), event, &runtime);
    }
}

fn send_messages_to_kafka(
    producer: Arc<CommonPublisher>,
    topic_or_exchange: String,
    event: ReplicationEvent,
    runtime: &Handle,
) {
    let key = match &event {
        ReplicationEvent::AddSchema { id, .. } => id,
        ReplicationEvent::AddSchemaVersion { id, .. } => id,
        ReplicationEvent::AddViewToSchema { schema_id, .. } => schema_id,
        ReplicationEvent::UpdateSchemaMetadata { id, .. } => id,
        ReplicationEvent::UpdateView { id, .. } => id,
    };
    let serialized = serde_json::to_string(&event).unwrap();
    let serialized_key = key.to_string();

    runtime.spawn(async move {
        let delivery_status = producer.publish_message(
            &topic_or_exchange,
            &serialized_key,
            serialized.as_bytes().to_vec(),
        );
        if delivery_status.await.is_err() {
            error!("Fatal error, delivery status for message not received.");
            process::abort();
        }
    });
}
