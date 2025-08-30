//! Ollama CLI tool
//!
//! A command-line interface for interacting with the Ollama API using the Rust SDK.

use clap::{Parser, Subcommand};
use ollama_rust_sdk::{OllamaClient, OllamaError};
use std::io::{self, Write};
use tokio_stream::StreamExt;

#[derive(Parser)]
#[command(name = "ollama-cli")]
#[command(about = "A CLI for interacting with the Ollama API")]
#[command(version)]
struct Cli {
    /// Ollama server URL
    #[arg(long, default_value = "http://localhost:11434")]
    url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate text completion
    Generate {
        /// The prompt to generate from
        prompt: String,
        /// Model to use
        #[arg(short, long, default_value = "qwen3:30b-a3b")]
        model: String,
        /// Enable streaming output
        #[arg(short, long)]
        stream: bool,
        /// Temperature for randomness (0.0 to 1.0)
        #[arg(short, long)]
        temperature: Option<f64>,
        /// Maximum number of tokens to generate
        #[arg(long)]
        max_tokens: Option<u32>,
    },
    /// Start an interactive chat session
    Chat {
        /// Model to use for chat
        #[arg(short, long, default_value = "qwen3:30b-a3b")]
        model: String,
        /// System message to set context
        #[arg(short, long)]
        system: Option<String>,
    },
    /// Embed text and get vectors
    Embed {
        /// Text to embed
        text: Vec<String>,
        /// Model to use for embeddings
        #[arg(short, long, default_value = "qwen3-embedding:8b")]
        model: String,
    },
    /// Model management commands
    #[command(subcommand)]
    Models(ModelCommands),
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List available models
    List,
    /// Show model information
    Show {
        /// Model name to show info for
        name: String,
    },
    /// Pull a model from registry
    Pull {
        /// Model name to pull
        name: String,
    },
    /// Delete a model
    Delete {
        /// Model name to delete
        name: String,
    },
    /// List running models
    Running,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();
    let client = OllamaClient::new(&cli.url)?;

    match cli.command {
        Commands::Generate {
            prompt,
            model,
            stream,
            temperature,
            max_tokens,
        } => {
            handle_generate(client, prompt, model, stream, temperature, max_tokens).await?;
        }
        Commands::Chat { model, system } => {
            handle_chat(client, model, system).await?;
        }
        Commands::Embed { text, model } => {
            handle_embed(client, text, model).await?;
        }
        Commands::Models(model_cmd) => {
            handle_model_commands(client, model_cmd).await?;
        }
    }

    Ok(())
}

async fn handle_generate(
    client: OllamaClient,
    prompt: String,
    model: String,
    stream: bool,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = client.generate().model(&model).prompt(&prompt);

    if let Some(temp) = temperature {
        builder = builder.temperature(temp);
    }

    if let Some(max_tokens) = max_tokens {
        builder = builder.max_tokens(max_tokens);
    }

    if stream {
        let mut stream = builder.stream().await?;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    print!("{}", response.response);
                    io::stdout().flush()?;
                }
                Err(e) => eprintln!("Stream error: {}", e),
            }
        }
        println!();
    } else {
        let response = builder.send().await?;
        println!("{}", response.response);
    }

    Ok(())
}

async fn handle_chat(
    client: OllamaClient,
    model: String,
    system: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting chat session with {}. Type 'quit' to exit.", model);

    let mut chat_builder = client.chat().model(&model);

    if let Some(sys_msg) = system {
        chat_builder = chat_builder.add_system_message(&sys_msg);
    }

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" {
            break;
        }

        chat_builder = chat_builder.add_user_message(input);

        match chat_builder.clone().send().await {
            Ok(response) => {
                println!("{}", response.message.content);
                chat_builder = chat_builder.add_assistant_message(&response.message.content);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}

async fn handle_embed(
    client: OllamaClient,
    texts: Vec<String>,
    model: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .embed()
        .model(&model)
        .input(texts.clone())
        .send()
        .await?;

    println!("Generated {} embeddings:", response.embeddings.len());
    for (i, text) in texts.iter().enumerate() {
        if let Some(embedding) = response.embeddings.get(i) {
            println!("Text: \"{}\"", text);
            println!("Embedding dimensions: {}", embedding.len());
            println!("First 5 values: {:?}", &embedding[..5.min(embedding.len())]);
            println!();
        }
    }

    Ok(())
}

async fn handle_model_commands(
    client: OllamaClient,
    command: ModelCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ModelCommands::List => {
            let models = client.list_models().await?;
            println!("Available models:");
            for model in models.models {
                println!("  {} ({})", model.name, model.size);
                if let Some(modified) = model.modified_at {
                    println!("    Modified: {}", modified.format("%Y-%m-%d %H:%M:%S"));
                }
            }
        }
        ModelCommands::Show { name } => match client.show_model(&name).await {
            Ok(info) => {
                println!("Model: {}", name);
                println!("Template: {}", info.template.unwrap_or_default());
                if let Some(params) = info.parameters {
                    println!("Parameters: {}", params);
                }
                if let Some(details) = info.details {
                    println!("Family: {}", details.family);
                    println!("Format: {}", details.format);
                    println!("Parameter Size: {}", details.parameter_size);
                    println!("Quantization Level: {}", details.quantization_level);
                }
            }
            Err(OllamaError::ModelNotFound(_)) => {
                eprintln!("Model '{}' not found", name);
            }
            Err(e) => return Err(e.into()),
        },
        ModelCommands::Pull { name } => {
            println!("Pulling model '{}'...", name);
            client.pull_model(&name).await?;
            println!("Successfully pulled model '{}'", name);
        }
        ModelCommands::Delete { name } => {
            println!("Deleting model '{}'...", name);
            client.delete_model(&name).await?;
            println!("Successfully deleted model '{}'", name);
        }
        ModelCommands::Running => {
            let running = client.list_running_models().await?;
            if running.models.is_empty() {
                println!("No models currently running");
            } else {
                println!("Running models:");
                for model in running.models {
                    println!("  {} ({})", model.name, model.size);
                }
            }
        }
    }

    Ok(())
}
