use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use iggy::prelude::Client as IggyClientTrait;
use iggy::prelude::*;
use milvus::client::Client as MilvusClient;
use milvus::data::FieldColumn;
use milvus::mutate::UpsertOptions;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

const IGGY_URL: &str = "iggy://iggy:iggy@127.0.0.1:8090";
const MILVUS_URL: &str = "http://127.0.0.1:19530";

const STREAM: &str = "movies";
const TOPIC: &str = "movie-events";
const COLLECTION: &str = "movie_events";
const DIM: i64 = 384;

#[derive(Debug, Deserialize)]
struct MovieIndexed {
    event_id: String,
    event: String,
    title: String,
    genre: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iggy = IggyClient::from_connection_string(IGGY_URL)?;
    iggy.connect().await?;

    let milvus = MilvusClient::new(MILVUS_URL).await?;

    let mut embedder =
        TextEmbedding::try_new(TextInitOptions::new(EmbeddingModel::MultilingualE5Small))?;

    let schema = CollectionSchemaBuilder::new(COLLECTION, "Eventos de filmes recebidos via Iggy")
        .add_field(FieldSchema::new_primary_varchar(
            "event_id",
            "idempotency key do evento",
            false,
            64,
        ))
        .add_field(FieldSchema::new_varchar("title", "título", 256))
        .add_field(FieldSchema::new_varchar("genre", "gênero", 64))
        .add_field(FieldSchema::new_float_vector(
            "embedding",
            "vetor FastEmbed",
            DIM,
        ))
        .build()?;

    let consumer = Consumer::new(Identifier::named("milvus-indexer")?);
    let stream_id: Identifier = STREAM.try_into()?;
    let topic_id: Identifier = TOPIC.try_into()?;

    println!("Consumer conectado. FastEmbed carregado. Aguardando eventos...");

    loop {
        let polled = iggy
            .poll_messages(
                &stream_id,
                &topic_id,
                Some(0),
                &consumer,
                &PollingStrategy::next(),
                10,
                false,
            )
            .await?;

        if polled.messages.is_empty() {
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        for message in polled.messages {
            let offset = message.header.offset;
            let movie: MovieIndexed = serde_json::from_slice(&message.payload)?;

            // Prefixo "passage:" é a convenção do modelo E5 para documentos.
            let document = format!("passage: Filme: {}. Gênero: {}.", movie.title, movie.genre);

            let embedding = embedder
                .embed(vec![document], None)?
                .into_iter()
                .next()
                .ok_or_else(|| std::io::Error::other("FastEmbed não retornou vetor"))?;

            let event_id = FieldColumn::new(
                schema.get_field("event_id").unwrap(),
                vec![movie.event_id.clone()],
            );
            let title = FieldColumn::new(
                schema.get_field("title").unwrap(),
                vec![movie.title.clone()],
            );
            let genre = FieldColumn::new(
                schema.get_field("genre").unwrap(),
                vec![movie.genre.clone()],
            );
            let embedding = FieldColumn::new(schema.get_field("embedding").unwrap(), embedding);

            milvus
                .upsert(
                    COLLECTION,
                    vec![event_id, title, genre, embedding],
                    UpsertOptions::default(),
                )
                .await?;

            milvus.flush(COLLECTION).await?;

            iggy.store_consumer_offset(&consumer, &stream_id, &topic_id, Some(0), offset)
                .await?;

            println!(
                "Indexado no Milvus | event_id={} | título={} | evento={} | offset confirmado={}",
                movie.event_id, movie.title, movie.event, offset
            );
        }
    }
}
