# 🚀 BLUEPRINT MESTRE: ERP ODONTOLÓGICO MULTI-TENANT (SaaS)

Este documento centraliza toda a arquitetura de software, dicionário de dados, design system e estado atual do ERP Odontológico. É o guia absoluto para desenvolvimento e expansão do sistema.

---

## 🛠️ 1. STACK TECNOLÓGICA E REGRAS GERAIS

* **Backend:** Rust + Actix Web.
* **Frontend:** Rust + Dioxus 0.7 (Client-Side Rendering/WebAssembly).
* **Banco de Dados:** SurrealDB (Multi-Modelo: Documento + Grafo Relacional).
* **Comunicação Externa:** Evolution API (WhatsApp OTP e Notificações).
* **Contratos:** Crate `shared` contendo todos os DTOs (`Serialize`, `Deserialize`).
* **Regras de Código:** 
  * Código estritamente em **Inglês** (variáveis, rotas, tabelas).
  * Interface do Usuário (UI) estritamente em **Português (BR)**.
  * Zero comentários no código (o código deve ser limpo e autoexplicativo).
* **Padrões de Dados:**
  * Valores monetários são `int` representando **centavos** (ex: R$ 50,00 = `5000`). O frontend formata para a UI.
  * Datas usam o formato UTC ISO 8601 (`datetime`).
  * IDs externos (N:1) usam ponteiros nativos do SurrealDB: `record<nome_da_tabela>`.

---

## ⚙️ 2. DIRETRIZES DE BACKEND E SEGURANÇA

1. **Autenticação e PBAC (Permission-Based Access Control):** 
   O sistema usa JWT. As rotas Actix são protegidas por um `AuthGuard` que verifica se a aresta de grafo `works_at` (que liga o usuário à clínica atual) possui a permissão específica (ex: `patients:write`) em seu array de `permissions`.
2. **Criptografia Determinística:** 
   O CPF dos pacientes (`document_cpf`) é criptografado no banco usando algoritmos como AES-SIV, garantindo que o banco armazene um hash seguro que **ainda permite buscas exatas** (`WHERE document_cpf = 'hash_do_cpf'`).
3. **Transações ACID:** 
   Lógicas que afetam mais de uma tabela (ex: Agendar e dar baixa no estoque, Rateio financeiro) devem obrigatoriamente ser encapsuladas em blocos `BEGIN TRANSACTION` e `COMMIT TRANSACTION` no SurrealDB.

---

## 🎨 3. DIRETRIZES DE FRONTEND E UI/UX

1. **Arquitetura Mock-First:** 
   Toda chamada de API no frontend DEVE ser abstraída no arquivo `frontend/src/mock_api.rs`. Os componentes Dioxus usam `use_resource` e `spawn` para acionar a API. Nenhum componente UI faz requisições (como `reqwest`) diretamente.
2. **Estilização (CSS Nativo):** 
   Proibido o uso de Tailwind, Bootstrap ou frameworks. Uso exclusivo de CSS nativo (Flexbox/Grid) em `style.css`.
3. **Linguagem Visual (SaaS Moderno):**
   * **Cores Principais:** Fundo principal `#f8fafc` (Slate 50). Textos base `#0f172a` e `#64748b`. Cor primária e botões de destaque `#00a0e4`.
   * **Geometria:** Bordas arredondadas (Radius 8px a 16px). Sombras suaves (`box-shadow: 0 4px 12px rgba(0,0,0,0.05)`).
   * **Ícones:** SVG inline centralizados em `icons.rs` com `stroke_width` de `1.5` a `1.8`.
4. **Componentes Genéricos (`ui_blocks.rs`):** 
   * `PageHeader`: Cabeçalho padrão (título, subtítulo, barra de busca e botão de ação).
   * `ActionModal`: Formulários flutuantes com *backdrop blur* e animação de entrada.

---

## 🗺️ 4. DIAGRAMA DE ARQUITETURA DO BANCO DE DADOS (SurrealDB)

```text
                  ┌─────────────────────────────────┐
                  │    system_settings (Global)     │
                  └────────────────┬────────────────┘
                                   │
                                   ▼
                            ┌──────────────┐
                            │    clinic    │
                            └──────┬───────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │ (1:N)                    │ (Grafo: works_at)        │ (1:N)
        ▼                          ▼                          ▼
┌──────────────┐            ┌──────────────┐           ┌──────────────┐
│  inventory   │            │     user     │           │   patient    │
└──────┬───────┘            └──────┬───────┘           └──────┬───────┘
       │                           │                          │
       │ (Grafo: consumes)         │ (Grafo: assigned_to)     │ (1:N)
       └───────────────┐           │                          │
                       ▼           ▼                          ▼
               ┌──────────────────────────┐        ┌──────────────────────┐
               │       appointment        │        │ patient_document     │
               └───────────┬──────────────┘        │ (Evolution API / OTP)│
                           │                       └──────────────────────┘
                           │ (Grafo: generates)
                           ▼
               ┌──────────────────────────┐
               │       transaction        │
               │   (Entradas / Saídas)    │
               └──────────────────────────┘

🗄️ 5. DICIONÁRIO DE DADOS E GRAFOS (Entidades)
MÓDULO 0: INFRAESTRUTURA E TENANTS

    Tabela: system_settings (Global)

        id: record (Apenas system_settings:global)

        platform_base_url: string

        patient_portal_url: string

        storage_provider: string (Enum: "aws_s3", "cloudflare_r2", "local")

        storage_bucket_name, storage_access_key, storage_secret_key, storage_region: string

        evolution_api_url, evolution_global_api_key: string

    Tabela: clinic (Tenant)

        id: record

        trading_name, corporate_name, document_cnpj: string

        address_street, address_number, address_zipcode, address_city, address_state: string

        theme_color, logo_url: string

        evolution_instance_name, evolution_instance_token: option

        is_active: bool

MÓDULO 1: IDENTIDADE (IAM) E USUÁRIOS

    Tabela: user (Funcionários)

        id: record

        username, password_hash, full_name: string

        document_cpf: string (Criptografado)

        professional_registry: option

        is_active: bool

    Aresta Grafo: works_at (De user para clinic)

        id: record<works_at>

        role: string (Enum: "admin", "dentist", "receptionist", "manager")

        permissions: array

MÓDULO 2: INVENTÁRIO E PATRIMÔNIO

    Tabela: inventory_item

        id: record<inventory_item>, clinic_id: record

        item_type: string (Enum: "material", "chemical", "equipment")

        name, unit_type: string

        current_stock, min_stock, cost_price_cents: int

        attachments: array

        expiration_date: option (Chemicals)

        batch_number: option (Chemicals)

        serial_number, warranty_until, next_maintenance_date: option (Equipments)

        equipment_status: string (Enum: "active", "in_maintenance", "broken")

    Tabela: stock_movement (Extrato Imutável)

        id: record<stock_movement>, item_id: record<inventory_item>, clinic_id: record, user_id: record

        quantity_change: int

        movement_type: string (Enum: "purchase_in", "manual_out", "appointment_out", "adjustment", "loss")

        invoice_number, notes: option

MÓDULO 3: AGENDA E ATENDIMENTO

    Tabela: appointment

        id: record, clinic_id: record

        patient_id: option<record>

        title: string, scheduled_for: datetime, duration_minutes: int

        status: string (Enum: "pending", "confirmed", "in_progress", "completed", "canceled", "no_show")

        appointment_type: string (Enum: "consultation", "treatment", "surgery", "return", "meeting", "other")

        cancellation_reason: option

    Aresta Grafo: assigned_to (De appointment para user)

        role_in_appointment: string, split_percentage: int (Rateio de 0 a 100)

    Aresta Grafo: consumes (De appointment para inventory_item)

        quantity_planned, quantity_used: int

MÓDULO 4: PACIENTES E PRONTUÁRIOS

    Tabela: patient

        id: record, clinic_id: record

        full_name, document_cpf (Criptografado), portal_password_hash: option

        birth_date: datetime, gender: option

        phone_whatsapp: string (Formato E.164 para Evolution API)

        email, emergency_contact_name, emergency_contact_phone: option

        address_street, address_number, address_zipcode: string

    Tabela: clinical_record (Evolução)

        id: record<clinical_record>, patient_id: record, clinic_id: record, doctor_id: record

        appointment_id: option<record>

        clinical_notes: string (Rico/Markdown)

    Tabela: exam_request

        id: record<exam_request>, patient_id: record, clinic_id: record, doctor_id: record

        description: string, attachments: array, status: string

MÓDULO 5: FINANCEIRO

    Tabela: transaction

        id: record, clinic_id: record

        appointment_id, patient_id, user_id: option

        direction: string (Enum: "income", "expense"), amount_cents: int

        description, category: string

        status: string (Enum: "pending", "paid", "canceled", "refunded")

        due_date: datetime, paid_date: option

        payment_method: option, installment_current, installment_total: int

MÓDULO 6: DOCUMENTOS E E-SIGN

    Tabela: document_template

        id: record<document_template>, clinic_id: record

        title, base_pdf_url, signature_map_json: string

    Tabela: patient_document

        id: record<patient_document>, clinic_id: record, patient_id: record, doctor_id: record

        template_id: record<document_template>

        status: string (Enum: "draft", "awaiting_signatures", "completed", "canceled")

        original_pdf_url: string, final_pdf_url, legal_checksum_sha256: option

        otp_current_code, otp_expires_at, patient_otp_verified_at: option

🧠 6. REGRAS DE NEGÓCIO E FLUXOS CRÍTICOS
Gatilho Oculto de Estoque (Auto-Consumo)

Quando o frontend realiza um PATCH /appointments/{id}/status mudando o status para completed, o backend busca todas as arestas consumes desta agenda. Subtrai automaticamente o quantity_used da tabela inventory_item e gera registros em stock_movement com o tipo appointment_out.
Gatilho de Rateio (Contas a Pagar Automático)

No mesmo momento que o agendamento muda para completed, se houver arestas assigned_to com split_percentage > 0, o sistema calcula a comissão em cima do valor total pago pelo paciente e gera uma transaction do tipo expense (pendente) associada ao user_id do doutor responsável.
Fluxo de Assinatura Digital (E-Sign com OTP via WhatsApp)

    Criação (POST /documents/send): O sistema gera o patient_document (awaiting_signatures).

    Notificação (Evolution API): O backend faz um POST para a Evolution API notificando o phone_whatsapp do paciente com o link seguro.

    Desafio OTP (POST /documents/request-otp): Quando o paciente abre o portal web (Base URL pública), o backend gera um código random de 6 dígitos, salva seu hash em patient_document.otp_current_code (validade de 5 mins) e envia via Evolution API.

    Assinatura (POST /documents/sign): O paciente digita o OTP correto e desenha na tela (Canvas). A API mescla a assinatura Base64 no PDF usando o signature_map_json, gera um hash SHA-256 definitivo (legal_checksum_sha256), limpa o OTP e marca o documento como completed.

🎨 7. DIRETRIZES UI/UX POR MÓDULO

    Configurações e Clínicas: Removido do menu principal. Fica no rodapé como "Configurações". Exibido em abas (Sistema vs Clínica).

    Usuários: Card List com badges, botão liga/desliga interativo, e modal de criação com Accordion para gerenciar matriz de permissões PBAC e múltiplas filiais.

    Pacientes: Ao invés de abrir Modal, navega para uma Página Dedicada Completa. Divide a visualização em abas: "Visão Geral", "Prontuário Evolutivo", "Exames" e "Documentos (Assinaturas)".

    Estoque: Abas principais separando "Materiais", "Químicos" e "Equipamentos". Uma quarta aba "Alertas" cruza os dados mostrando itens abaixo do estoque mínimo, validade próxima e revisões pendentes.

    Agenda: Visualização em grade de calendário horária. Formulários de agendamento gerenciam vínculos de paciente, doutor, rateios e estoque preventivo.

    Financeiro: Cards de KPl no topo. Visualização baseada em abas ("Entradas", "Saídas", "Pendentes"). Filtro travado no mês atual por padrão. Lógica de pendências processada com base no due_date versus paid_date.

📈 8. ESTADO ATUAL DO PROJETO (O QUE JÁ FOI FEITO)

    Interface Base Consolidada:

        Menu lateral retrátil 100% responsivo e com posicionamento absoluto para prevenir quebras de layout.

        Estilização baseada no style.css construída sem frameworks externos.

    Módulo de Usuários Finalizado:

        UI em funcionamento baseada em Mock API com uso da estrutura use_resource. Modais de criação (PBAC + Unidades), edição e deleção abstraídos.

        Múltiplas sub-funções criadas e encapsuladas.

    Bibliotecas Geração de Componentes e Ícones:

        Estrutura genérica ActionModal e PageHeader implementada em ui_blocks.rs.

        Arquivo de ícones icons.rs blindado para o padrão rigoroso de snake_case do Dioxus 0.7.

    Integração Backend SurrealDB Base:

        Automação de migrations lendo da variável MIGRATIONS_DIR.

        Estrutura global do Crate shared iniciada.

🤖 9. INSTRUÇÃO DE SISTEMA (SYSTEM PROMPT PARA AGENTES DE IA)

Quando for abrir uma nova thread para codificar uma "Stack" ou "Módulo", envie o resumo do contexto e finalize sempre com este prompt fixo:

    Você é um Agente de Engenharia Full-Stack Sênior. Sua stack é Rust (Actix-Web) no Backend, Dioxus 0.7 (Client-Side Rendering) no Frontend, e SurrealDB como banco.

    REGRAS ESTRITAS:

        Código 100% em Inglês. Apenas as Strings de UI em Português-BR.

        Zero comentários no código gerado. Não escreva textos explicativos no meio da lógica.

        Todos os DTOs (Serialize, Deserialize) devem ir para o módulo shared.

        O Frontend nunca faz requisições reais agora. Utilize use_resource e spawn acionando funções falsas (Mock) em mock_api.rs.

        Estilize apenas via style.css global (sem frameworks).

    Sua Tarefa Atual: Implemente a Stack Completa (Contratos em shared, Migration .surql no SurrealDB, Rotas Actix com AuthGuard PBAC e Componente de Interface Dioxus) para o Módulo de [NOME DO MÓDULO]. Você deve respeitar rigorosamente os nomes de tabelas, relacionamentos e tipos especificados no Blueprint Mestre de Dados.
