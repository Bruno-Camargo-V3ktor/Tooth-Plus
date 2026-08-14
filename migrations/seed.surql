BEGIN TRANSACTION;

DELETE consumes;
DELETE assigned_to;
DELETE appointment;
DELETE transaction;
DELETE financial_transaction;
DELETE stock_movement;
DELETE inventory_item;
DELETE patient;
DELETE works_at;
DELETE user;
DELETE clinic;

CREATE clinic:smile_plus CONTENT {
    corporate_name: "Smile Plus Odontologia LTDA",
    trading_name: "Smile Plus",
    document_cnpj: "11.111.111/0001-11",
    theme_color: "#0052cc",
    logo_url: "https://placehold.co/400x120/transparent/00a0e4?text=Smile+Plus",
    whatsapp_instance: "smile_plus_matriz",
    street: "Av. Paulista",
    number: "1000",
    complement: "Sala 402",
    neighborhood: "Bela Vista",
    city: "São Paulo",
    state: "SP",
    zip_code: "01310-100",
    auto_reminders: true,
    require_esign: true
};

CREATE clinic:luria_odonto CONTENT {
    corporate_name: "Luria Clínica Odontológica LTDA",
    trading_name: "Luria Odonto",
    document_cnpj: "22.222.222/0001-22",
    theme_color: "#263d25",
    logo_url: "https://placehold.co/400x120/transparent/1e293b?text=Luria+Odonto",
    whatsapp_instance: "luria_matriz",
    street: "Rua Faria Lima",
    number: "200",
    complement: "Andar 2",
    neighborhood: "Itaim Bibi",
    city: "São Paulo",
    state: "SP",
    zip_code: "01452-000",
    auto_reminders: true,
    require_esign: false
};

CREATE user:administrador CONTENT {
    username: "admin",
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$qvKgAYt6Mjuvveqb72VDRg$lQSuMHI7Fv4EcpvQyp3/r9mQjx0JE5TskDnv0B6A7AU",
    full_name: "Administrador do Sistema",
    document_cpf: "000.000.000-00",
    professional_registry: NONE,
    is_active: true
};

RELATE user:administrador->works_at->clinic:smile_plus CONTENT {
    role: "admin",
    permissions: [
        "clinics:read",
        "clinics:write",
        "clinics:delete",
        "whatsapp:read",
        "whatsapp:write",
        "advanced:read",
        "advanced:write",
        "users:read",
        "users:write",
        "users:manage_status",
        "patients:read",
        "patients:write",
        "patients:delete",
        "agenda:read",
        "agenda:write",
        "agenda:delete",
        "agenda:finance",
        "appointments:read",
        "appointments:write",
        "appointments:delete",
        "appointments:finance",
        "finance:read",
        "finance:read_all",
        "finance:read_income",
        "finance:read_expense",
        "finance:read_pending",
        "finance:write",
        "finance:write_income",
        "finance:write_expense",
        "finance:update_status",
        "finance:delete",
        "stock:read",
        "stock:write",
        "stock:delete",
        "documents:read",
        "documents:write",
        "documents:delete"
    ]
};

RELATE user:administrador->works_at->clinic:luria_odonto CONTENT {
    role: "admin",
    permissions: [
        "clinics:read",
        "clinics:write",
        "clinics:delete",
        "whatsapp:read",
        "whatsapp:write",
        "advanced:read",
        "advanced:write",
        "users:read",
        "users:write",
        "users:manage_status",
        "patients:read",
        "patients:write",
        "patients:delete",
        "agenda:read",
        "agenda:write",
        "agenda:delete",
        "agenda:finance",
        "appointments:read",
        "appointments:write",
        "appointments:delete",
        "appointments:finance",
        "finance:read",
        "finance:read_all",
        "finance:read_income",
        "finance:read_expense",
        "finance:read_pending",
        "finance:write",
        "finance:write_income",
        "finance:write_expense",
        "finance:update_status",
        "finance:delete",
        "stock:read",
        "stock:write",
        "stock:delete",
        "documents:read",
        "documents:write",
        "documents:delete"
    ]
};

CREATE patient:ana_silva CONTENT {
    clinic_id: clinic:smile_plus,
    full_name: "Ana Clara Silva",
    document_cpf: "123.456.789-00",
    phone: "(11) 98765-4321",
    email: "ana.silva@email.com"
};

CREATE patient:carlos_souza CONTENT {
    clinic_id: clinic:smile_plus,
    full_name: "Carlos Eduardo Souza",
    document_cpf: "234.567.890-11",
    phone: "(11) 97654-3210",
    email: "carlos.souza@email.com"
};

CREATE inventory_item:anestesico CONTENT {
    clinic_id: clinic:smile_plus,
    name: "Anestésico Articaína 4% (Tubete)",
    unit: "unidade",
    current_stock: 50,
    min_stock: 10,
    cost_cents: 850
};

CREATE inventory_item:luvas CONTENT {
    clinic_id: clinic:smile_plus,
    name: "Par de Luvas Cirúrgicas Estéreis",
    unit: "par",
    current_stock: 200,
    min_stock: 30,
    cost_cents: 400
};

CREATE inventory_item:resina CONTENT {
    clinic_id: clinic:smile_plus,
    name: "Resina Composta Z350 XT (Tubo)",
    unit: "unidade",
    current_stock: 15,
    min_stock: 3,
    cost_cents: 12000
};

CREATE transaction CONTENT {
    clinic_id: clinic:smile_plus,
    direction: "income",
    amount_cents: 35000,
    description: "Tratamento de Clareamento Dental",
    category: "Procedimento Clínico",
    status: "paid",
    due_date: time::now(),
    paid_date: time::now(),
    payment_method: "Pix",
    installment_current: 1,
    installment_total: 1
};

CREATE transaction CONTENT {
    clinic_id: clinic:smile_plus,
    direction: "income",
    amount_cents: 18000,
    description: "Consulta e Limpeza Geral",
    category: "Consulta",
    status: "paid",
    due_date: time::now(),
    paid_date: time::now(),
    payment_method: "Cartão de Crédito",
    installment_current: 1,
    installment_total: 1
};

CREATE transaction CONTENT {
    clinic_id: clinic:smile_plus,
    direction: "expense",
    amount_cents: 45000,
    description: "Reposição de Material Odontológico (Dental Cremer)",
    category: "Insumos & Estoque",
    status: "paid",
    due_date: time::now(),
    paid_date: time::now(),
    payment_method: "Boleto Bancário",
    installment_current: 1,
    installment_total: 1
};

CREATE transaction CONTENT {
    clinic_id: clinic:smile_plus,
    direction: "expense",
    amount_cents: 12000,
    description: "Manutenção Preventiva Compressor de Ar",
    category: "Manutenção",
    status: "pending",
    due_date: time::now() + 5d,
    paid_date: NONE,
    payment_method: "Pix",
    installment_current: 1,
    installment_total: 1
};

COMMIT TRANSACTION;
