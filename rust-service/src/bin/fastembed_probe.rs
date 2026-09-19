use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = TextEmbedding::try_new(
        TextInitOptions::new(EmbeddingModel::MultilingualE5Small).with_show_download_progress(true),
    )?;

    let texts = vec![
        "passage: The Matrix é um filme de ficção científica sobre realidade simulada.",
        "query: filme sci-fi com uma realidade controlada por máquinas",
    ];

    let embeddings = model.embed(texts, None)?;

    println!("Embeddings gerados: {}", embeddings.len());
    println!("Dimensão do vetor: {}", embeddings[0].len());
    println!("Primeiros valores: {:?}", &embeddings[0][..5]);

    Ok(())
}
