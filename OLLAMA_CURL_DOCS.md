# Ollama API Complete Curl Reference - Qwen3 Edition

## Base Configuration
- **Default Base URL**: `http://localhost:11434`
- **Primary Test Model**: `qwen3:30b-a3b` (Qwen3 30B model)
- **Secondary Test Model**: `gpt-oss:20b` (OpenAI GPT-OSS 20B)
- **Embedding Model**: `qwen3-embedding:8b` (Qwen3 Embedding 8B)
- **No Authentication Required** (for local setup)

## Environment Variables
```bash
export OLLAMA_HOST="0.0.0.0"        # Bind to all interfaces
export OLLAMA_PORT="11434"          # Default port
export OLLAMA_MODELS="/usr/share/ollama/.ollama/models"  # Model storage path
export OLLAMA_KEEP_ALIVE="5m"       # Model memory retention
export OLLAMA_NUM_PARALLEL="4"      # Parallel request handling
export OLLAMA_MAX_LOADED_MODELS="1" # Max models in memory
```

## Model Setup Commands
```bash
# Pull primary models
ollama pull qwen3:30b-a3b
ollama pull gpt-oss:20b
ollama pull qwen3-embedding:8b

# Alternative: Create custom modelfile for Qwen3
echo "FROM qwen/qwen3-30b-a3b" > Modelfile
ollama create qwen3:30b-a3b -f Modelfile
```

---

## 1. Core Generation APIs

### Generate Completion (Streaming)
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Why is the sky blue?",
    "stream": true
  }'
```

### Generate Completion (Non-Streaming)
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Why is the sky blue?",
    "stream": false
  }'
```

### Generate with Options (Qwen3 Optimized)
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Tell me a story",
    "stream": false,
    "options": {
      "temperature": 0.7,
      "top_p": 0.9,
      "top_k": 40,
      "num_predict": 200,
      "num_ctx": 8192,
      "stop": ["\\n", "END"],
      "seed": 42
    }
  }'
```

### Test with GPT-OSS 20B
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "gpt-oss:20b",
    "prompt": "Explain quantum computing in simple terms",
    "stream": false,
    "options": {
      "temperature": 0.8,
      "max_tokens": 150
    }
  }'
```

### Generate with System Prompt
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Hello",
    "system": "You are a helpful AI assistant that speaks like a pirate.",
    "stream": false
  }'
```

### Generate with Context (Conversation Memory)
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "What did I just ask?",
    "context": [1, 2, 3, 4, 5],
    "stream": false
  }'
```

### Generate with Custom Template
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Hello",
    "template": "{{ .System }}\nUSER: {{ .Prompt }}\nASSISTANT:",
    "stream": false
  }'
```

### Generate with Keep Alive
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Quick question: what is 2+2?",
    "keep_alive": "10m",
    "stream": false
  }'
```

### Generate with JSON Format
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "List 3 colors in JSON format",
    "format": "json",
    "stream": false
  }'
```

### Generate Raw (No Formatting)
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "<|im_start|>user\nHello<|im_end|>\n<|im_start|>assistant",
    "raw": true,
    "stream": false
  }'
```

---

## 2. Chat APIs

### Chat Completion (Streaming)
```bash
curl http://localhost:11434/api/chat \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [
      {
        "role": "user",
        "content": "Hello! How are you?"
      }
    ],
    "stream": true
  }'
```

### Chat Completion (Non-Streaming)
```bash
curl http://localhost:11434/api/chat \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [
      {
        "role": "user",
        "content": "Hello! How are you?"
      }
    ],
    "stream": false
  }'
```

### Multi-Turn Conversation with Qwen3
```bash
curl http://localhost:11434/api/chat \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful math tutor."
      },
      {
        "role": "user",
        "content": "What is calculus?"
      },
      {
        "role": "assistant",
        "content": "Calculus is a branch of mathematics that studies continuous change."
      },
      {
        "role": "user",
        "content": "Can you give me an example?"
      }
    ],
    "stream": false
  }'
```

### Compare Models Side by Side
```bash
# Test with Qwen3
curl http://localhost:11434/api/chat \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [{"role": "user", "content": "Write a haiku about AI"}],
    "stream": false
  }'

# Test with GPT-OSS
curl http://localhost:11434/api/chat \
  -d '{
    "model": "gpt-oss:20b",
    "messages": [{"role": "user", "content": "Write a haiku about AI"}],
    "stream": false
  }'
```

### Chat with Options (Qwen3 Optimized)
```bash
curl http://localhost:11434/api/chat \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [
      {
        "role": "user",
        "content": "Write a creative story"
      }
    ],
    "options": {
      "temperature": 0.9,
      "top_p": 0.95,
      "max_tokens": 500,
      "num_ctx": 8192
    },
    "stream": false
  }'
```

### Chat with Tools/Functions
```bash
curl http://localhost:11434/api/chat \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [
      {
        "role": "user",
        "content": "What is the weather in San Francisco?"
      }
    ],
    "tools": [
      {
        "type": "function",
        "function": {
          "name": "get_weather",
          "description": "Get the weather for a location",
          "parameters": {
            "type": "object",
            "properties": {
              "location": {
                "type": "string",
                "description": "The city and state"
              }
            },
            "required": ["location"]
          }
        }
      }
    ],
    "stream": false
  }'
```

---

## 3. Embedding APIs

### Generate Embeddings with Qwen3 Embedding Model
```bash
curl http://localhost:11434/api/embed \
  -d '{
    "model": "qwen3-embedding:8b",
    "input": "The sky is blue because of Rayleigh scattering"
  }'
```

### Generate Batch Embeddings
```bash
curl http://localhost:11434/api/embed \
  -d '{
    "model": "qwen3-embedding:8b",
    "input": [
      "The sky is blue",
      "Grass is green",
      "Roses are red"
    ]
  }'
```

### Generate Embeddings with Options
```bash
curl http://localhost:11434/api/embed \
  -d '{
    "model": "qwen3-embedding:8b",
    "input": "Machine learning is fascinating",
    "truncate": true,
    "options": {
      "temperature": 0.1
    },
    "keep_alive": "5m"
  }'
```

### Generate Embeddings (Deprecated API)
```bash
curl http://localhost:11434/api/embeddings \
  -d '{
    "model": "qwen3-embedding:8b",
    "prompt": "The quick brown fox"
  }'
```

---

## 4. Model Management APIs

### Pull Qwen3 Models
```bash
# Pull Qwen3 30B A3B model
curl http://localhost:11434/api/pull \
  -d '{
    "name": "qwen3:30b-a3b",
    "stream": true
  }'

# Pull GPT-OSS 20B model
curl http://localhost:11434/api/pull \
  -d '{
    "name": "gpt-oss:20b",
    "stream": true
  }'

# Pull Qwen3 Embedding model
curl http://localhost:11434/api/pull \
  -d '{
    "name": "qwen3-embedding:8b",
    "stream": true
  }'
```

### Pull Model (Non-Streaming)
```bash
curl http://localhost:11434/api/pull \
  -d '{
    "name": "qwen3:30b-a3b",
    "stream": false
  }'
```

### Create Custom Qwen3 Model
```bash
curl http://localhost:11434/api/create \
  -d '{
    "name": "qwen3-custom",
    "modelfile": "FROM qwen3:30b-a3b\nSYSTEM You are a specialized AI assistant for coding tasks.\nPARAMETER temperature 0.7\nPARAMETER num_ctx 8192",
    "stream": true
  }'
```

### Create with Quantization
```bash
curl http://localhost:11434/api/create \
  -d '{
    "name": "qwen3:30b-quantized",
    "modelfile": "FROM qwen3:30b-a3b",
    "quantize": "q4_K_M",
    "stream": false
  }'
```

### Copy a Model
```bash
curl http://localhost:11434/api/copy \
  -d '{
    "source": "qwen3:30b-a3b",
    "destination": "my-qwen3"
  }'
```

### Delete a Model
```bash
curl -X DELETE http://localhost:11434/api/delete \
  -d '{
    "name": "qwen3:old"
  }'
```

### Show Model Information
```bash
curl http://localhost:11434/api/show \
  -d '{
    "name": "qwen3:30b-a3b"
  }'
```

### Show Model with Verbose Details
```bash
curl http://localhost:11434/api/show \
  -d '{
    "name": "qwen3:30b-a3b",
    "verbose": true
  }'
```

---

## 5. Model Listing APIs

### List All Models
```bash
curl http://localhost:11434/api/tags
```

### List Running Models
```bash
curl http://localhost:11434/api/ps
```

---

## 6. Blob Management APIs

### Check if Blob Exists
```bash
curl -I http://localhost:11434/api/blobs/sha256:29fdb92e57cf0827ded04ae6461b5931d01fa595843f55d36f5b275a52087dd2
```

### Create a Blob
```bash
curl -T model.gguf -X POST \
  http://localhost:11434/api/blobs/sha256:29fdb92e57cf0827ded04ae6461b5931d01fa595843f55d36f5b275a52087dd2
```

---

## 7. OpenAI Compatibility API

### Chat Completions with Qwen3 (OpenAI Format)
```bash
curl http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ollama" \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "Hello!"
      }
    ]
  }'
```

### Streaming Chat Completions
```bash
curl http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ollama" \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [
      {
        "role": "user",
        "content": "Tell me a story"
      }
    ],
    "stream": true
  }'
```

### Chat with Temperature (GPT-OSS Test)
```bash
curl http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ollama" \
  -d '{
    "model": "gpt-oss:20b",
    "messages": [
      {
        "role": "user",
        "content": "Write a creative poem"
      }
    ],
    "temperature": 0.9,
    "max_tokens": 200,
    "top_p": 0.95
  }'
```

### Chat with Tools (OpenAI Format)
```bash
curl http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ollama" \
  -d '{
    "model": "qwen3:30b-a3b",
    "messages": [
      {
        "role": "user",
        "content": "What is the weather in Paris?"
      }
    ],
    "tools": [
      {
        "type": "function",
        "function": {
          "name": "get_weather",
          "description": "Get weather for a city",
          "parameters": {
            "type": "object",
            "properties": {
              "city": {"type": "string"}
            },
            "required": ["city"]
          }
        }
      }
    ]
  }'
```

### Embeddings (OpenAI Format)
```bash
curl http://localhost:11434/v1/embeddings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ollama" \
  -d '{
    "model": "qwen3-embedding:8b",
    "input": "The quick brown fox jumps over the lazy dog"
  }'
```

### List Models (OpenAI Format)
```bash
curl http://localhost:11434/v1/models \
  -H "Authorization: Bearer ollama"
```

### Legacy Completions (OpenAI Format)
```bash
curl http://localhost:11434/v1/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ollama" \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Once upon a time",
    "max_tokens": 100,
    "temperature": 0.7
  }'
```

---

## 8. Advanced Features

### Qwen3 Optimized Parameters
```bash
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Explain quantum computing",
    "options": {
      "num_ctx": 8192,        # Qwen3 supports larger context
      "num_batch": 512,
      "num_gpu": 2,           # Use multiple GPUs for 30B model
      "main_gpu": 0,
      "low_vram": false,
      "f16_kv": true,
      "logits_all": false,
      "vocab_only": false,
      "use_mmap": true,
      "use_mlock": false,
      "num_thread": 16,       # More threads for larger model
      "num_keep": 24,
      "seed": 42,
      "num_predict": 256,
      "top_k": 40,
      "top_p": 0.9,
      "tfs_z": 1.0,
      "typical_p": 1.0,
      "repeat_last_n": 64,
      "temperature": 0.8,
      "repeat_penalty": 1.1,
      "presence_penalty": 0.0,
      "frequency_penalty": 0.0,
      "mirostat": 0,
      "mirostat_tau": 5.0,
      "mirostat_eta": 0.1,
      "penalize_newline": true,
      "stop": ["\\n", "User:"]
    },
    "stream": false
  }'
```

### Model Comparison Script
```bash
#!/bin/bash
# Compare responses from Qwen3 and GPT-OSS

PROMPT="Explain the concept of neural networks in simple terms"

echo "=== Qwen3 30B Response ==="
curl -s http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "'"$PROMPT"'",
    "stream": false
  }' | jq -r '.response'

echo -e "\n=== GPT-OSS 20B Response ==="
curl -s http://localhost:11434/api/generate \
  -d '{
    "model": "gpt-oss:20b",
    "prompt": "'"$PROMPT"'",
    "stream": false
  }' | jq -r '.response'
```

### Batch Processing with Multiple Models
```bash
# Process prompts with both models
for model in "qwen3:30b-a3b" "gpt-oss:20b"; do
  echo "Testing model: $model"
  for prompt in "What is AI?" "Explain ML" "Define NLP"; do
    echo "Prompt: $prompt"
    curl -s http://localhost:11434/api/generate \
      -d '{
        "model": "'"$model"'",
        "prompt": "'"$prompt"'",
        "stream": false
      }' | jq -r '.response' | head -n 3
    echo "---"
  done
  echo "=========="
done
```

### Health Check
```bash
# Check if Ollama is running
curl http://localhost:11434/
# Returns: "Ollama is running"
```

### Version Check
```bash
curl http://localhost:11434/api/version
```

---

## Performance Benchmarking

### Benchmark Generation Speed
```bash
#!/bin/bash
# Compare generation speeds

echo "Benchmarking Qwen3 30B..."
time curl -s http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "prompt": "Write a 100-word story",
    "stream": false,
    "options": {"num_predict": 100}
  }' > /dev/null

echo "Benchmarking GPT-OSS 20B..."
time curl -s http://localhost:11434/api/generate \
  -d '{
    "model": "gpt-oss:20b",
    "prompt": "Write a 100-word story",
    "stream": false,
    "options": {"num_predict": 100}
  }' > /dev/null
```

### Benchmark Embeddings
```bash
#!/bin/bash
# Test embedding generation speed

echo "Testing Qwen3 Embedding 8B..."
time curl -s http://localhost:11434/api/embed \
  -d '{
    "model": "qwen3-embedding:8b",
    "input": ["Test sentence 1", "Test sentence 2", "Test sentence 3"]
  }' > /dev/null
```

---

## Best Practices for Qwen3 Models

### 1. Model Selection
```bash
# For highest quality responses
ollama pull qwen3:30b-a3b    # Best quality, requires ~32GB RAM

# For balanced performance
ollama pull gpt-oss:20b       # Good balance, requires ~20GB RAM

# For embeddings
ollama pull qwen3-embedding:8b  # High-quality embeddings

# For resource-constrained environments (if available)
ollama pull qwen3:7b          # Smaller variant if needed
```

### 2. Performance Optimization for Large Models
```bash
# Pre-load Qwen3 30B in memory
curl http://localhost:11434/api/generate \
  -d '{
    "model": "qwen3:30b-a3b",
    "keep_alive": "30m",
    "prompt": ""
  }'

# Use appropriate context window
# Qwen3 supports up to 8K context efficiently
# Adjust num_ctx based on your needs (default: 2048, max: 8192)
```

### 3. Docker Deployment for Large Models
```bash
# With GPU support (NVIDIA) - recommended for 30B model
docker run -d \
  --gpus=all \
  -v ollama:/root/.ollama \
  -p 11434:11434 \
  --name ollama \
  --shm-size=8gb \
  ollama/ollama

# Pull models in container
docker exec -it ollama ollama pull qwen3:30b-a3b
docker exec -it ollama ollama pull gpt-oss:20b
docker exec -it ollama ollama pull qwen3-embedding:8b
```

### 4. Memory Requirements
- **Qwen3 30B A3B**: ~32GB RAM (GPU recommended)
- **GPT-OSS 20B**: ~20GB RAM
- **Qwen3 Embedding 8B**: ~8GB RAM

### 5. GPU Configuration
```bash
# Check GPU availability
nvidia-smi

# Set GPU layers for optimal performance
export OLLAMA_NUM_GPU=2  # Use 2 GPUs if available
export OLLAMA_GPU_LAYERS=35  # Offload layers to GPU
```

### 6. Security Considerations
- Use reverse proxy with authentication for production
- Limit access to model management endpoints
- Monitor resource usage for large models
- Consider rate limiting for API endpoints
