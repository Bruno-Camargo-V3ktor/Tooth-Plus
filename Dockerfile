# ==========================================
# 🦀 STAGE 1: Build Backend (Rust Actix-Web)
# ==========================================
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /usr/src/app

# Instala dependências nativas de compilação
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copia manifests do workspace e código fonte
COPY Cargo.toml Cargo.lock ./
COPY shared/ ./shared/
COPY backend/ ./backend/
COPY frontend/Cargo.toml ./frontend/Cargo.toml

# Compila o binário do backend em modo release
RUN cargo build --release -p backend

# ==========================================
# 🚀 STAGE 2: Runtime Image (Minimal & Secure)
# ==========================================
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Instala dependências de runtime necessárias para SSL/TLS e healthchecks
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Cria diretórios da aplicação e de uploads
RUN mkdir -p /app/uploads /app/migrations

# Copia o binário compilado e os arquivos de migração do SurrealDB
COPY --from=builder /usr/src/app/target/release/backend /app/backend
COPY migrations/ /app/migrations/

# Define permissões de execução
RUN chmod +x /app/backend

# Configurações de ambiente padrão
ENV PORT=4000
ENV MIGRATIONS_DIR=/app/migrations
ENV STORAGE_BUCKET=/app/uploads

# Expõe a porta do backend
EXPOSE 4000

# Ponto de montagem para persistência de arquivos locais
VOLUME ["/app/uploads"]

# Comando de inicialização
CMD ["/app/backend"]
