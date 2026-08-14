BEGIN TRANSACTION;

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
        "finance:read",
        "finance:write",
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
        "finance:read",
        "finance:write",
        "finance:delete",
        "stock:read",
        "stock:write",
        "stock:delete",
        "documents:read",
        "documents:write",
        "documents:delete"
    ]
};

COMMIT TRANSACTION;
