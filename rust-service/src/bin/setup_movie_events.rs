use milvus::client::Client;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;

const MILVUS_URL: &str = "http://127.0.0.1:19530";
const COLLECTION: &str = "movie_events";
const DIM: i64 = 8;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(MILVUS_URL).await?;

    if client.has_collection(COLLECTION).await? {
        println!("Collection já existe: {COLLECTION}");
        return Ok(());
    }

    let schema = CollectionSchemaBuilder::new(COLLECTION, "Eventos de filmes recebidos via Iggy")
        .add_field(FieldSchema::new_primary_varchar(
            "event_id",
            "idempotency key do evento",
            false,
            64,
        ))
        .add_field(FieldSchema::new_varchar("title", "título", 256))
        .add_field(FieldSchema::new_varchar("genre", "gênero", 64))
        .add_field(FieldSchema::new_float_vector("embedding", "vetor", DIM))
        .build()?;

    client.create_collection(schema, None).await?;

    let index = IndexParams::new(
        "embedding_idx".to_owned(),
        IndexType::Flat,
        MetricType::L2,
        HashMap::new(),
    );

    client.create_index(COLLECTION, "embedding", index).await?;

    println!("Collection pronta: {COLLECTION}");
    println!("Chave primária: event_id");
    println!("Índice vetorial: embedding_idx");

    Ok(())
}
