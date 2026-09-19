use iggy::prelude::*;
use std::str::FromStr;

const IGGY_URL: &str = "iggy://iggy:iggy@127.0.0.1:8090";
const STREAM: &str = "movies";
const TOPIC: &str = "movie-events";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IggyClient::from_connection_string(IGGY_URL)?;

    client.connect().await?;
    println!("Conectado no Iggy.");

    match client.create_stream(STREAM).await {
        Ok(_) => println!("Stream criado: {STREAM}"),
        Err(IggyError::StreamNameAlreadyExists(_)) => {
            println!("Stream já existe: {STREAM}");
        }
        Err(error) => return Err(error.into()),
    }

    match client
        .create_topic(
            &STREAM.try_into()?,
            TOPIC,
            1,
            CompressionAlgorithm::None,
            None,
            IggyExpiry::NeverExpire,
            MaxTopicSize::ServerDefault,
        )
        .await
    {
        Ok(_) => println!("Topic criado: {TOPIC}"),
        Err(IggyError::TopicNameAlreadyExists(_, _)) => {
            println!("Topic já existe: {TOPIC}");
        }
        Err(error) => return Err(error.into()),
    }

    let message = IggyMessage::from_str(
        r#"{
            "event_id": "movie-001",
            "event": "movie_indexed",
            "title": "The Matrix",
            "genre": "sci-fi"
        }"#,
    )?;

    client
        .send_messages(
            &STREAM.try_into()?,
            &TOPIC.try_into()?,
            &Partitioning::partition_id(0),
            &mut [message],
        )
        .await?;

    println!("Evento movie-001 enviado.");

    Ok(())
}
