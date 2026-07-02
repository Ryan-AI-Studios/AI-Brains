@echo off
set AI_BRAINS_VAULT_PATH=C:\dev\ai-brains\vault.db
set AI_BRAINS_MODEL_URL=http://127.0.0.1:8081
set AI_BRAINS_COMPLETION_MODEL=gemma-4-E4B-it-Q6_K.gguf
set AI_BRAINS_EMBEDDING_URL=http://127.0.0.1:8083
set AI_BRAINS_EMBEDDING_MODEL=nomic-embed-text-v1.5
"C:\Users\RyanB\.cargo\bin\ai-brains.exe" --no-project-context nightly --skip-import --log-format json