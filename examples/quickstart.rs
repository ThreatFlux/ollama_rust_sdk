use ollama_rust_sdk::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::from_env()?;
    let model = std::env::var("OLLAMA_MODEL")?;

    let response = client
        .generate()
        .model(model)
        .prompt("Explain Rust's ownership model in one concise sentence.")
        .send()
        .await?;

    println!("{}", response.response);
    Ok(())
}
