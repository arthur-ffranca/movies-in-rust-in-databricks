use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::index::{IndexParams, IndexType, MetricType};
use milvus::options::LoadOptions;
use milvus::query::SearchOptions;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;
use std::collections::HashMap;

const MILVUS_URL: &str = "http://127.0.0.1:19530";
const COLLECTION: &str = "movie_vectors";
const DIM: i64 = 8;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(MILVUS_URL).await?;
    println!("Conectado no Milvus.");

    if client.has_collection(COLLECTION).await? {
        client.drop_collection(COLLECTION).await?;
        println!("Collection de teste anterior removida.");
    }

    let schema = CollectionSchemaBuilder::new(COLLECTION, "Filmes indexados pelo Iggy")
        .add_field(FieldSchema::new_primary_int64("id", "chave interna", true))
        .add_field(FieldSchema::new_varchar("title", "título do filme", 256))
        .add_field(FieldSchema::new_varchar("genre", "gênero", 64))
        .add_field(FieldSchema::new_float_vector(
            "embedding",
            "vetor do filme",
            DIM,
        ))
        .build()?;

    client.create_collection(schema.clone(), None).await?;
    println!("Collection criada: {COLLECTION}");

    let index = IndexParams::new(
        "embedding_idx".to_owned(),
        IndexType::Flat,
        MetricType::L2,
        HashMap::new(),
    );

    client.create_index(COLLECTION, "embedding", index).await?;
    println!("Índice vetorial criado.");

    let title = FieldColumn::new(
        schema.get_field("title").unwrap(),
        vec!["The Matrix".to_string()],
    );
    let genre = FieldColumn::new(
        schema.get_field("genre").unwrap(),
        vec!["sci-fi".to_string()],
    );
    let embedding = FieldColumn::new(
        schema.get_field("embedding").unwrap(),
        vec![0.12_f32, 0.91, 0.34, 0.77, 0.28, 0.65, 0.43, 0.88],
    );

    client
        .insert(COLLECTION, vec![title, genre, embedding], None)
        .await?;
    client.flush(COLLECTION).await?;
    println!("The Matrix inserido no Milvus.");

    client
        .load_collection(COLLECTION, Some(LoadOptions::default()))
        .await?;

    let search = client
        .search(
            COLLECTION,
            vec![Value::from(vec![
                0.12_f32, 0.91, 0.34, 0.77, 0.28, 0.65, 0.43, 0.88,
            ])],
            Some(
                SearchOptions::with_limit(1)
                    .output_fields(vec!["title".to_string(), "genre".to_string()])
                    .add_param("anns_field", "embedding")
                    .add_param("metric_type", "L2"),
            ),
        )
        .await?;

    println!("Busca vetorial retornou:\n{search:#?}");

    Ok(())
}
