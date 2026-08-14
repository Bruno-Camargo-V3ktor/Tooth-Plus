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
        "stock:movement",
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
        "stock:movement",
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

CREATE inventory_item:luvas CONTENT {
    clinic_id: clinic:smile_plus,
    item_type: "material",
    name: "Par de Luvas Cirúrgicas Estéreis (M)",
    unit_type: "par",
    current_stock: 180,
    min_stock: 30,
    cost_price_cents: 450,
    manufacturer: "Supermax",
    attachments: []
};

CREATE inventory_item:resina CONTENT {
    clinic_id: clinic:smile_plus,
    item_type: "material",
    name: "Resina Composta Z350 XT (Tubo A2)",
    unit_type: "unidade",
    current_stock: 4,
    min_stock: 6,
    cost_price_cents: 12500,
    manufacturer: "3M ESPE",
    attachments: []
};

CREATE inventory_item:anestesico CONTENT {
    clinic_id: clinic:smile_plus,
    item_type: "chemical",
    name: "Anestésico Articaína 4% c/ Epinefrina",
    unit_type: "tubete",
    current_stock: 45,
    min_stock: 15,
    cost_price_cents: 890,
    manufacturer: "DFL",
    batch_number: "LT-2026B",
    expiration_date: time::now() + 18d,
    attachments: []
};

CREATE inventory_item:hipoclorito CONTENT {
    clinic_id: clinic:smile_plus,
    item_type: "chemical",
    name: "Hipoclorito de Sódio 2.5% (Endodontia)",
    unit_type: "frasco",
    current_stock: 6,
    min_stock: 2,
    cost_price_cents: 1850,
    manufacturer: "Asfer",
    batch_number: "HP-8802",
    expiration_date: time::now() + 150d,
    attachments: []
};

CREATE inventory_item:autoclave CONTENT {
    clinic_id: clinic:smile_plus,
    item_type: "equipment",
    name: "Autoclave Digital Vitale 12L",
    unit_type: "unidade",
    current_stock: 1,
    min_stock: 1,
    cost_price_cents: 680000,
    manufacturer: "Cristófoli",
    serial_number: "CRIST-2025-9912",
    warranty_until: time::now() + 320d,
    next_maintenance_date: time::now() + 12d,
    equipment_status: "active",
    attachments: []
};

CREATE inventory_item:fotopolimerizador CONTENT {
    clinic_id: clinic:smile_plus,
    item_type: "equipment",
    name: "Fotopolimerizador LED Emitter Now",
    unit_type: "unidade",
    current_stock: 2,
    min_stock: 1,
    cost_price_cents: 195000,
    manufacturer: "Schuster",
    serial_number: "SCH-8831",
    warranty_until: time::now() + 200d,
    next_maintenance_date: time::now() + 60d,
    equipment_status: "active",
    attachments: []
};

CREATE inventory_item:compressor CONTENT {
    clinic_id: clinic:smile_plus,
    item_type: "equipment",
    name: "Compressor Odontológico Isento de Óleo 60L",
    unit_type: "unidade",
    current_stock: 1,
    min_stock: 1,
    cost_price_cents: 480000,
    manufacturer: "Schulz",
    serial_number: "SCH-AIR-990",
    warranty_until: time::now() - 40d,
    next_maintenance_date: time::now() - 3d,
    equipment_status: "in_maintenance",
    attachments: []
};

CREATE stock_movement CONTENT {
    item_id: inventory_item:luvas,
    clinic_id: clinic:smile_plus,
    user_id: user:administrador,
    quantity_change: 200,
    movement_type: "purchase_in",
    unit_cost_cents: 450,
    invoice_number: "NF-88412",
    notes: "Compra inicial de estoque Dental Cremer"
};

CREATE stock_movement CONTENT {
    item_id: inventory_item:luvas,
    clinic_id: clinic:smile_plus,
    user_id: user:administrador,
    quantity_change: -20,
    movement_type: "manual_out",
    unit_cost_cents: 450,
    notes: "Consumo da semana em procedimentos cirúrgicos"
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

