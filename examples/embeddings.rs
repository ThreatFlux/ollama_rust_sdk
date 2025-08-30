//! Embeddings example

use ollama_rust_sdk::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = OllamaClient::new("http://localhost:11434")?;

    // Check server health
    if !client.health().await? {
        eprintln!("Ollama server is not available");
        return Ok(());
    }

    println!("=== Embeddings Example ===");

    // Try to use an embedding model, fallback to any available model
    let embedding_models = ["qwen3-embedding:8b", "nomic-embed-text", "all-minilm"];
    let models = client.list_models().await?;

    let embedding_model = embedding_models
        .iter()
        .find(|&model| models.models.iter().any(|m| m.name.contains(model)))
        .copied()
        .unwrap_or_else(|| {
            // Use first available model as fallback
            models
                .models
                .first()
                .map(|m| m.name.as_str())
                .unwrap_or("qwen3:30b-a3b")
        });

    println!("Using embedding model: {}", embedding_model);

    // Single text embedding
    println!("\n--- Single Text Embedding ---");
    let single_text = "The quick brown fox jumps over the lazy dog.";

    match client
        .embed()
        .model(embedding_model)
        .input(single_text)
        .send()
        .await
    {
        Ok(response) => {
            println!("Input text: \"{}\"", single_text);
            println!(
                "Embedding dimensions: {}",
                response.dimensions().unwrap_or(0)
            );

            if let Some(embedding) = response.get_embedding(0) {
                println!(
                    "First 10 values: {:?}",
                    &embedding[..10.min(embedding.len())]
                );

                // Calculate magnitude (L2 norm)
                let magnitude: f64 = embedding.iter().map(|x| x * x).sum::<f64>().sqrt();
                println!("Embedding magnitude: {:.4}", magnitude);
            }
        }
        Err(e) => {
            eprintln!("Single embedding failed: {}", e);
        }
    }

    // Batch embeddings
    println!("\n--- Batch Embeddings ---");
    let texts = vec![
        "I love programming in Rust.",
        "Python is a great language for data science.",
        "Machine learning is fascinating.",
        "The weather is nice today.",
        "I enjoy reading books.",
    ];

    match client
        .embed()
        .model(embedding_model)
        .input(texts.clone())
        .truncate(true)
        .send()
        .await
    {
        Ok(response) => {
            println!("Generated {} embeddings", response.count());
            println!(
                "Embedding dimensions: {}",
                response.dimensions().unwrap_or(0)
            );

            // Calculate pairwise similarities
            println!("\n--- Similarity Analysis ---");
            for (i, text1) in texts.iter().enumerate() {
                for (j, text2) in texts.iter().enumerate() {
                    if i < j {
                        if let (Some(emb1), Some(emb2)) =
                            (response.get_embedding(i), response.get_embedding(j))
                        {
                            if let Some(similarity) =
                                ollama_rust_sdk::models::embedding::EmbedResponse::cosine_similarity(
                                    emb1, emb2,
                                )
                            {
                                println!(
                                    "Similarity between \"{}\" and \"{}\": {:.4}",
                                    text1, text2, similarity
                                );
                            }
                        }
                    }
                }
            }

            // Find most similar pair
            let mut max_similarity = -1.0;
            let mut most_similar = (0, 0);

            for i in 0..texts.len() {
                for j in (i + 1)..texts.len() {
                    if let (Some(emb1), Some(emb2)) =
                        (response.get_embedding(i), response.get_embedding(j))
                    {
                        if let Some(similarity) =
                            ollama_rust_sdk::models::embedding::EmbedResponse::cosine_similarity(
                                emb1, emb2,
                            )
                        {
                            if similarity > max_similarity {
                                max_similarity = similarity;
                                most_similar = (i, j);
                            }
                        }
                    }
                }
            }

            println!("\nMost similar texts (similarity: {:.4}):", max_similarity);
            println!("  1: \"{}\"", texts[most_similar.0]);
            println!("  2: \"{}\"", texts[most_similar.1]);
        }
        Err(e) => {
            eprintln!("Batch embeddings failed: {}", e);

            // If embedding model failed, suggest downloading one
            if e.to_string().contains("not found") {
                println!("\nTo use embeddings, try pulling an embedding model:");
                println!("  ollama pull qwen3-embedding:8b");
                println!("  ollama pull nomic-embed-text");
                println!("  ollama pull all-minilm");
            }
        }
    }

    Ok(())
}
