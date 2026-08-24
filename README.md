# 🦷 Tooth Plus (V2)

Sistema SaaS Moderno e Completo para Gestão de Clínicas e Consultórios Odontológicos.

## 🚀 Tecnologias

- **Backend:** [Rust](https://www.rust-lang.org/) (Actix-Web 4.x + Tokio)
- **Frontend:** [Dioxus 0.7](https://dioxuslabs.com/) (Web / Desktop / Mobile)
- **Banco de Dados:** [SurrealDB 3.x](https://surrealdb.com/)
- **Criptografia & LGPD:** AES-256-GCM + Argon2id + Blind Indexing SHA-256

## 📁 Estrutura do Workspace

```text
Tooth-Plus/
├── backend/      # API REST assíncrona em Actix-Web
├── frontend/     # Interface do Usuário em Dioxus
├── shared/       # Modelos de domínio e tipos compartilhados
└── migrations/   # Scripts de definição de esquema SurrealDB (SurQL)
```

## 🛠️ Como Executar

### Pré-requisitos
- Rust & Cargo (versão 1.85+)
- Dioxus CLI (`cargo install dioxus-cli --locked`)
- SurrealDB CLI (`surreal start ...`)

### Inicialização Rápida
1. Inicie o banco de dados SurrealDB:
   ```bash
   surreal start --user root --pass root rocksdb://database.db
   ```
2. Inicie o backend:
   ```bash
   cargo run -p backend
   ```
3. Inicie o frontend:
   ```bash
   cd frontend && dx serve
   ```