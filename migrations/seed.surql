BEGIN TRANSACTION;

DELETE patient_document;
DELETE contract_template;
DELETE patient_treatment;
DELETE patient_exam;
DELETE patient_anamnesis;
DELETE consumes;
DELETE assigned_to;
DELETE appointment;
DELETE financial_transaction;
DELETE stock_movement;
DELETE inventory_item;
DELETE patient;
DELETE works_at;
DELETE user;
DELETE clinic;

-- =========================================================================
-- CLINICS
-- =========================================================================
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

-- =========================================================================
-- USERS & ACCESS
-- =========================================================================
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
        "clinics:read", "clinics:write", "clinics:delete",
        "whatsapp:read", "whatsapp:write",
        "advanced:read", "advanced:write",
        "users:read", "users:write", "users:manage_status",
        "patients:read", "patients:write", "patients:delete",
        "agenda:read", "agenda:write", "agenda:delete", "agenda:finance",
        "appointments:read", "appointments:write", "appointments:delete", "appointments:finance",
        "finance:read", "finance:read_all", "finance:read_income", "finance:read_expense", "finance:read_pending",
        "finance:write", "finance:write_income", "finance:write_expense", "finance:update_status", "finance:delete",
        "stock:read", "stock:write", "stock:delete", "stock:movement",
        "documents:read", "documents:write", "documents:delete"
    ]
};

CREATE user:dr_andre CONTENT {
    username: "dr.andre",
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$qvKgAYt6Mjuvveqb72VDRg$lQSuMHI7Fv4EcpvQyp3/r9mQjx0JE5TskDnv0B6A7AU",
    full_name: "Dr. André Martins",
    document_cpf: "111.222.333-44",
    professional_registry: "CRO/SP 123456",
    is_active: true
};

RELATE user:dr_andre->works_at->clinic:smile_plus CONTENT {
    role: "dentist",
    permissions: [
        "patients:read", "patients:write",
        "agenda:read", "agenda:write", "agenda:delete", "agenda:finance",
        "appointments:read", "appointments:write", "appointments:delete", "appointments:finance",
        "documents:read", "documents:write"
    ]
};

CREATE user:dra_camila CONTENT {
    username: "dra.camila",
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$qvKgAYt6Mjuvveqb72VDRg$lQSuMHI7Fv4EcpvQyp3/r9mQjx0JE5TskDnv0B6A7AU",
    full_name: "Dra. Camila Toledo",
    document_cpf: "555.666.777-88",
    professional_registry: "CRO/SP 987654",
    is_active: true
};

RELATE user:dra_camila->works_at->clinic:smile_plus CONTENT {
    role: "dentist",
    permissions: [
        "patients:read", "patients:write",
        "agenda:read", "agenda:write",
        "appointments:read", "appointments:write",
        "documents:read", "documents:write"
    ]
};

-- =========================================================================
-- PATIENTS (WITH ENCRYPTED CPF & BLIND INDEX)
-- =========================================================================
CREATE patient:ana_silva CONTENT {
    clinic_id: clinic:smile_plus,
    full_name: "Ana Clara Silva",
    document_cpf: "123.456.789-00",
    document_cpf_encrypted: "94iWSwugEtZK5tTCZ1cIkahAE7fhKLI4NH7roNDInMpHUAkTAlkAWvS5",
    document_cpf_hash: "a8476735b37a541a38402a2e7037c79e2d217fe9780e5e34347156ef61eff42b",
    phone: "(11) 98765-4321",
    email: "ana.silva@email.com",
    birth_date: "1994-05-12",
    gender: "Feminino",
    marital_status: "Solteira",
    profession: "Arquiteta",
    emergency_contact_name: "Marcos Silva (Pai)",
    emergency_contact_phone: "(11) 98111-2233",
    address_street: "Rua Augusta",
    address_number: "1450",
    address_complement: "Apto 82",
    address_neighborhood: "Consolação",
    address_city: "São Paulo",
    address_state: "SP",
    address_zip: "01304-001",
    insurance_plan: "Amil Dental Premium",
    insurance_number: "98745612300",
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$qvKgAYt6Mjuvveqb72VDRg$lQSuMHI7Fv4EcpvQyp3/r9mQjx0JE5TskDnv0B6A7AU"
};

CREATE patient:carlos_souza CONTENT {
    clinic_id: clinic:smile_plus,
    full_name: "Carlos Eduardo Souza",
    document_cpf: "234.567.890-11",
    document_cpf_encrypted: "Rb/RTUsvjVdpPSamn7Sf6i0EE2AO8B22hf/IZMNRWjvhEbfADK18Y8nw",
    document_cpf_hash: "4549dc14132cf0b858e5ff1e5cef1390761e1122dcf00f8e190d95df21e9f7a1",
    phone: "(11) 97654-3210",
    email: "carlos.souza@email.com",
    birth_date: "1988-11-24",
    gender: "Masculino",
    marital_status: "Casado",
    profession: "Engenheiro de Software",
    emergency_contact_name: "Patricia Souza (Esposa)",
    emergency_contact_phone: "(11) 97222-3344",
    address_street: "Av. Brigadeiro Luis Antonio",
    address_number: "2300",
    address_complement: "Conj 41",
    address_neighborhood: "Jardins",
    address_city: "São Paulo",
    address_state: "SP",
    address_zip: "01402-000",
    insurance_plan: "SulAmérica Odonto Mais",
    insurance_number: "45678912301",
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$qvKgAYt6Mjuvveqb72VDRg$lQSuMHI7Fv4EcpvQyp3/r9mQjx0JE5TskDnv0B6A7AU"
};

CREATE patient:juliana_prado CONTENT {
    clinic_id: clinic:smile_plus,
    full_name: "Juliana Mendes Prado",
    document_cpf: "345.678.901-22",
    document_cpf_encrypted: "jSbTGlnvCamDB2Xm+pdoo2wvR22IjHrO0DigE/vxOQy1kpwzNL1de5FR",
    document_cpf_hash: "03203a457cc4c03b828f35be54f2e29eb29b5d4cd711145212ed2a8cc6316720",
    phone: "(11) 96543-2109",
    email: "juliana.prado@email.com",
    birth_date: "1999-03-08",
    gender: "Feminino",
    marital_status: "Solteira",
    profession: "Designer de Interiores",
    address_street: "Rua Oscar Freire",
    address_number: "750",
    address_neighborhood: "Cerqueira César",
    address_city: "São Paulo",
    address_state: "SP",
    address_zip: "01426-001",
    insurance_plan: "Particular",
    insurance_number: NONE,
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$qvKgAYt6Mjuvveqb72VDRg$lQSuMHI7Fv4EcpvQyp3/r9mQjx0JE5TskDnv0B6A7AU"
};

CREATE patient:roberto_neto CONTENT {
    clinic_id: clinic:smile_plus,
    full_name: "Roberto Albuquerque Neto",
    document_cpf: "456.789.012-33",
    document_cpf_encrypted: "9yyBpyqATuECDq1JHmWGttkbY876aS5wCttMoLC2jBt8SeZTVHTHgBQ/",
    document_cpf_hash: "e464c08d7cc57079c86f31ecbd54175fc8de650b8c6ebc154c86bdd45fb67d57",
    phone: "(11) 95432-1098",
    email: "roberto.neto@email.com",
    birth_date: "1975-09-17",
    gender: "Masculino",
    marital_status: "Casado",
    profession: "Empresário",
    address_street: "Alameda Santos",
    address_number: "1800",
    address_neighborhood: "Cerqueira César",
    address_city: "São Paulo",
    address_state: "SP",
    address_zip: "01418-102",
    insurance_plan: "Porto Seguro Odonto Ouro",
    insurance_number: "78912345600",
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$qvKgAYt6Mjuvveqb72VDRg$lQSuMHI7Fv4EcpvQyp3/r9mQjx0JE5TskDnv0B6A7AU"
};

CREATE patient:mariana_castro CONTENT {
    clinic_id: clinic:smile_plus,
    full_name: "Mariana Castro Fernandes",
    document_cpf: "567.890.123-44",
    document_cpf_encrypted: "e7/ComGlRlwgGsv6O6rdopMSzcrtRwq8ltW5wWMdsc02dy4xDgQRrBaP",
    document_cpf_hash: "6da96fae5f098bc43ec8c421fc83c795fcac445863677ee5c79677b48994c7e2",
    phone: "(11) 94321-0987",
    email: "mariana.castro@email.com",
    birth_date: "2001-12-03",
    gender: "Feminino",
    marital_status: "Solteira",
    profession: "Estudante Universitária",
    address_street: "Rua Haddock Lobo",
    address_number: "920",
    address_neighborhood: "Cerqueira César",
    address_city: "São Paulo",
    address_state: "SP",
    address_zip: "01414-001",
    insurance_plan: "Bradesco Dental Top",
    insurance_number: "32165498700",
    password_hash: "$argon2id$v=19$m=19456,t=2,p=1$qvKgAYt6Mjuvveqb72VDRg$lQSuMHI7Fv4EcpvQyp3/r9mQjx0JE5TskDnv0B6A7AU"
};

-- =========================================================================
-- ANAMNESES
-- =========================================================================
CREATE patient_anamnesis:ana_anamnesis CONTENT {
    patient_id: patient:ana_silva,
    clinic_id: clinic:smile_plus,
    allergies: ["Penicilina", "Dipirona"],
    continuous_medications: "Antialérgico sob demanda",
    systemic_diseases: ["Rinite Alérgica"],
    is_pregnant: false,
    has_bleeding_disorder: false,
    smoker: false,
    bruxism: true,
    chief_complaint: "Deseja realizar clareamento dental a laser e alinhadores invisíveis.",
    clinical_notes: "Paciente com sensibilidade leve em incisivos inferiores. Bruxismo noturno moderado."
};

CREATE patient_anamnesis:carlos_anamnesis CONTENT {
    patient_id: patient:carlos_souza,
    clinic_id: clinic:smile_plus,
    allergies: ["Sulfa"],
    continuous_medications: "Losartana 50mg (Hipertensão)",
    systemic_diseases: ["Hipertensão Controlada"],
    is_pregnant: false,
    has_bleeding_disorder: false,
    smoker: false,
    bruxism: false,
    chief_complaint: "Dor no dente 36 ao mastigar alimentos frios e duros.",
    clinical_notes: "Necessário restauração profunda ou tratamento endodôntico no elemento 36."
};

-- =========================================================================
-- EXAMS
-- =========================================================================
CREATE patient_exam:exam_panoramica_ana CONTENT {
    patient_id: patient:ana_silva,
    clinic_id: clinic:smile_plus,
    title: "Radiografia Panorâmica Digital",
    exam_type: "panoramic",
    requested_by_user_id: user:dr_andre,
    status: "completed",
    requested_date: time::now() - 5d,
    result_date: time::now() - 2d,
    file_urls: ["https://placehold.co/900x600/0f172a/38bdf8?text=Radiografia+Panorâmica+Digital+-+Ana+Clara"],
    clinical_interpretation: "Presença dos 4 terceiros molares inclusos sem lesões periapicais visíveis."
};

CREATE patient_exam:exam_tomografia_carlos CONTENT {
    patient_id: patient:carlos_souza,
    clinic_id: clinic:smile_plus,
    title: "Tomografia Cone Beam Mandibular Elemento 36",
    exam_type: "tomography",
    requested_by_user_id: user:dr_andre,
    status: "completed",
    requested_date: time::now() - 10d,
    result_date: time::now() - 7d,
    file_urls: ["https://placehold.co/900x600/0f172a/10b981?text=Tomografia+Cone+Beam+-+Elemento+36"],
    clinical_interpretation: "Fratura radicular descartada. Lesão cariosa profunda próxima à câmara pulpar."
};

-- =========================================================================
-- TREATMENTS
-- =========================================================================
CREATE patient_treatment:treat_clareamento_ana CONTENT {
    patient_id: patient:ana_silva,
    clinic_id: clinic:smile_plus,
    dentist_user_id: user:dr_andre,
    procedure_name: "Clareamento Dental em Consultório (Sessão 1)",
    tooth_number: "Arcada Superior e Inferior",
    status: "completed",
    cost_cents: 85000,
    clinical_notes: "Sessão realizada com gel de Peróxido de Hidrogênio 35% e barreira gengival."
};

CREATE patient_treatment:treat_alinhador_ana CONTENT {
    patient_id: patient:ana_silva,
    clinic_id: clinic:smile_plus,
    dentist_user_id: user:dr_andre,
    procedure_name: "Planejamento e Escaneamento Intraoral 3D para Alinhadores",
    tooth_number: "Geral",
    status: "in_progress",
    cost_cents: 450000,
    clinical_notes: "Setup virtual aprovado. Aguardando entrega dos alinhadores."
};

CREATE patient_treatment:treat_endo_carlos CONTENT {
    patient_id: patient:carlos_souza,
    clinic_id: clinic:smile_plus,
    dentist_user_id: user:dr_andre,
    procedure_name: "Tratamento Endodôntico Unirradicular/Multirradicular",
    tooth_number: "36",
    status: "in_progress",
    cost_cents: 120000,
    clinical_notes: "Odontometria realizada. Canais instrumentados e curativo de hidróxido de cálcio."
};

-- =========================================================================
-- CONTRACT TEMPLATES & DIGITAL DOCUMENTS
-- =========================================================================
CREATE contract_template:tcle_geral CONTENT {
    clinic_id: clinic:smile_plus,
    title: "Termo de Consentimento Livre e Esclarecido (TCLE Geral)",
    category: "consent",
    description: "Termo padrão de esclarecimento de riscos, benefícios e responsabilidades para tratamentos clínicos e cirúrgicos.",
    pdf_url: "https://placehold.co/800x1100/ffffff/0f172a?text=Termo+TCLE+Odontologico",
    signature_fields: [
        {
            id: "sig_pat_1",
            signer_type: "patient",
            page_number: 1,
            x_pct: 15.0,
            y_pct: 82.0,
            width_pct: 30.0,
            height_pct: 10.0,
            label: "Assinatura do Paciente / Responsável",
            is_required: true
        },
        {
            id: "sig_doc_1",
            signer_type: "doctor",
            page_number: 1,
            x_pct: 55.0,
            y_pct: 82.0,
            width_pct: 30.0,
            height_pct: 10.0,
            label: "Assinatura do Cirurgião-Dentista Responsável",
            is_required: true
        }
    ]
};

CREATE contract_template:contrato_orto CONTENT {
    clinic_id: clinic:smile_plus,
    title: "Contrato de Prestação de Serviços Ortodônticos / Alinhadores",
    category: "orthodontics",
    description: "Contrato detalhado para planejamento, manutenção e fornecimento de alinhadores estéticos ou aparelhos fixos.",
    pdf_url: "https://placehold.co/800x1100/ffffff/0f172a?text=Contrato+de+Prestação+de+Servicos+Ortodonticos",
    signature_fields: [
        {
            id: "sig_pat_orto",
            signer_type: "patient",
            page_number: 2,
            x_pct: 15.0,
            y_pct: 85.0,
            width_pct: 30.0,
            height_pct: 10.0,
            label: "Assinatura do Contratante",
            is_required: true
        },
        {
            id: "sig_doc_orto",
            signer_type: "doctor",
            page_number: 2,
            x_pct: 55.0,
            y_pct: 85.0,
            width_pct: 30.0,
            height_pct: 10.0,
            label: "Assinatura do Responsável Técnico",
            is_required: true
        }
    ]
};

CREATE patient_document:doc_ana_tcle CONTENT {
    clinic_id: clinic:smile_plus,
    patient_id: patient:ana_silva,
    template_id: contract_template:tcle_geral,
    doctor_user_id: user:dr_andre,
    title: "TCLE - Clareamento Dental e Estética",
    document_type: "consent",
    original_pdf_url: "https://placehold.co/800x1100/ffffff/0f172a?text=Termo+TCLE+Clareamento+-+Ana+Clara",
    status: "signed",
    signing_token: "tok_ana_tcle_2026_sample_ok",
    patient_signed_at: time::now() - 4d,
    patient_signature_data: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 300 100'><path d='M10,80 Q90,10 180,60 T290,30' stroke='%230052cc' stroke-width='3' fill='none'/></svg>",
    doctor_signed_at: time::now() - 4d,
    doctor_signature_data: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 300 100'><path d='M20,60 Q120,20 200,80 T280,40' stroke='%230052cc' stroke-width='3' fill='none'/></svg>",
    final_checksum_sha256: "8e837f48b3941bfa706ec2efbdcdbfa780d6ae37de898863f64c679905c756b1",
    require_whatsapp_otp: false,
    audit_trail: [
        {
            event: "document_created",
            timestamp: time::now() - 4d,
            ip: "127.0.0.1",
            user: "dr.andre"
        },
        {
            event: "patient_signed",
            timestamp: time::now() - 4d,
            ip: "127.0.0.1",
            signer: "Ana Clara Silva"
        },
        {
            event: "doctor_signed",
            timestamp: time::now() - 4d,
            ip: "127.0.0.1",
            signer: "Dr. André Martins"
        }
    ]
};

CREATE patient_document:doc_carlos_endo CONTENT {
    clinic_id: clinic:smile_plus,
    patient_id: patient:carlos_souza,
    template_id: contract_template:tcle_geral,
    doctor_user_id: user:dr_andre,
    title: "TCLE - Tratamento Endodôntico Elemento 36",
    document_type: "consent",
    original_pdf_url: "https://placehold.co/800x1100/ffffff/0f172a?text=Termo+TCLE+Endodontia+-+Carlos+Souza",
    status: "pending_signatures",
    signing_token: "tok_carlos_endo_2026_pending",
    require_whatsapp_otp: true,
    audit_trail: [
        {
            event: "document_created",
            timestamp: time::now() - 1d,
            ip: "127.0.0.1",
            user: "dr.andre"
        }
    ]
};

-- =========================================================================
-- STOCK & INVENTORY
-- =========================================================================
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
    cost_price_cents: 420000,
    manufacturer: "Cristófoli",
    serial_number: "CF-2025-9981",
    warranty_until: time::now() + 365d,
    next_maintenance_date: time::now() - 5d,
    equipment_status: "active",
    attachments: []
};

-- =========================================================================
-- APPOINTMENTS & FINANCIAL TRANSACTIONS
-- =========================================================================
CREATE appointment:apt_ana CONTENT {
    clinic_id: clinic:smile_plus,
    patient_id: patient:ana_silva,
    patient_name: "Ana Clara Silva",
    title: "Consulta Inicial & Avaliação Estética",
    scheduled_for: time::now() + 2h,
    duration_minutes: 45,
    status: "confirmed",
    appointment_type: "consultation",
    financial_amount_cents: 25000,
    financial_type: "income"
};

RELATE appointment:apt_ana->assigned_to->user:dr_andre CONTENT {
    role_in_appointment: "Cirurgião-Dentista Principal",
    split_percentage: 40
};

CREATE appointment:apt_carlos CONTENT {
    clinic_id: clinic:smile_plus,
    patient_id: patient:carlos_souza,
    patient_name: "Carlos Eduardo Souza",
    title: "Sessão de Endodontia Elemento 36",
    scheduled_for: time::now() + 5h,
    duration_minutes: 60,
    status: "in_progress",
    appointment_type: "treatment",
    financial_amount_cents: 60000,
    financial_type: "income"
};

RELATE appointment:apt_carlos->assigned_to->user:dr_andre CONTENT {
    role_in_appointment: "Endodontista",
    split_percentage: 50
};

CREATE financial_transaction:tx_1 CONTENT {
    clinic_id: clinic:smile_plus,
    appointment_id: appointment:apt_ana,
    user_id: user:dr_andre,
    amount_cents: 25000,
    direction: "income",
    status: "completed",
    category: "Consultas & Avaliações",
    description: "Pagamento de Avaliação Inicial - Ana Clara Silva",
    created_at: time::now() - 3h
};

CREATE financial_transaction:tx_2 CONTENT {
    clinic_id: clinic:smile_plus,
    amount_cents: 45000,
    direction: "expense",
    status: "completed",
    category: "Materiais Odontológicos",
    description: "Compra de Reposição - Luvas e Sugadores descartáveis",
    created_at: time::now() - 1d
};

CREATE financial_transaction:tx_3 CONTENT {
    clinic_id: clinic:smile_plus,
    appointment_id: appointment:apt_carlos,
    user_id: user:dr_andre,
    amount_cents: 60000,
    direction: "income",
    status: "pending",
    category: "Procedimentos Clínicos",
    description: "Endodontia Sessão 1 - Carlos Eduardo Souza",
    created_at: time::now()
};

COMMIT TRANSACTION;
