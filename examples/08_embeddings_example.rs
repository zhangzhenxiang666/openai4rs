use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    // 1. Single text embedding
    println!("=== Single Text Embedding ===");
    let request = EmbeddingsParam::new("text-embedding-ada-002", "Hello, world!");
    let response = client.embeddings().create(request).await?;
    println!("Generated {} embedding(s)", response.len());
    if let Some(embedding) = response.get_embedding(0) {
        println!("Embedding dimensions: {}", embedding.dimensions());
        let vector = embedding.vector().expect("failed to get embedding vector");
        println!("First 5 values: {:?}", &vector[0..5.min(vector.len())]);
    }

    // 2. Multiple text embeddings
    println!("\n=== Multiple Text Embeddings ===");
    let texts = vec!["Hello, world!", "How are you?", "Rust is awesome!"];
    let request = EmbeddingsParam::new("text-embedding-ada-002", texts);
    let response = client.embeddings().create(request).await?;
    println!("Generated {} embeddings", response.len());
    for (i, embedding) in response.embeddings().iter().enumerate() {
        println!("Embedding {}: {} dimensions", i, embedding.dimensions());
    }

    // 3. Get embedding vectors
    println!("\n=== Embedding Vector Information ===");
    let embedding_vectors = response.embedding_vectors();
    for (i, vector) in embedding_vectors.iter().enumerate() {
        println!("Vector {}: length {}", i, vector.len());
    }

    println!("\nTotal tokens used: {}", response.total_tokens());
    println!("Prompt tokens: {}", response.prompt_tokens());

    Ok(())
}
