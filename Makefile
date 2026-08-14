.PHONY: dev db migrate backend frontend check clean

all: dev

dev:
	@echo "=========================================================="
	@echo "           🚀 INICIANDO TOOTH PLUS DEV STACK              "
	@echo "=========================================================="
	@if ! curl -s http://127.0.0.1:8000/health > /dev/null 2>&1; then \
		echo "==> Iniciando SurrealDB (rocksdb://database.db)..."; \
		surreal start --user root --pass root rocksdb://database.db & \
		sleep 2; \
	fi
	@echo "==> Aplicando migrações..."
	@curl -s -X POST -u root:root -H "Accept: application/json" -d "USE NS saas; USE DB \`tooth-smile\`; $$(cat migrations/006_patients_and_documents.surql)" http://127.0.0.1:8000/sql > /dev/null
	@curl -s -X POST -u root:root -H "Accept: application/json" -d "USE NS saas; USE DB \`tooth-smile\`; $$(cat migrations/seed.surql)" http://127.0.0.1:8000/sql > /dev/null
	@if ! curl -s http://127.0.0.1:4000/ > /dev/null 2>&1; then \
		echo "==> Iniciando Backend Actix-Web na porta 4000..."; \
		cargo run -p backend & \
		sleep 3; \
	fi
	@echo "==> Iniciando Frontend Dioxus na porta 8080..."
	@cd frontend && dx serve

db:
	surreal start --user root --pass root rocksdb://database.db

migrate:
	@echo "==> Executando migrações no banco..."
	curl -s -X POST -u root:root -H "Accept: application/json" -d "USE NS saas; USE DB \`tooth-smile\`; $$(cat migrations/006_patients_and_documents.surql)" http://127.0.0.1:8000/sql
	curl -s -X POST -u root:root -H "Accept: application/json" -d "USE NS saas; USE DB \`tooth-smile\`; $$(cat migrations/seed.surql)" http://127.0.0.1:8000/sql

backend:
	cargo run -p backend

frontend:
	cd frontend && dx serve

check:
	cargo check --workspace
