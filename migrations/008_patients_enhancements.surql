BEGIN TRANSACTION;

-- 1. Modelos de Anamnese por Clínica (Adulto e Menor de Idade)
DEFINE TABLE OVERWRITE anamnesis_template SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON TABLE anamnesis_template TYPE record<clinic>;
DEFINE FIELD OVERWRITE template_type ON TABLE anamnesis_template TYPE string DEFAULT "adult";
DEFINE FIELD OVERWRITE title ON TABLE anamnesis_template TYPE string;
DEFINE FIELD OVERWRITE questions ON TABLE anamnesis_template TYPE array<object> FLEXIBLE DEFAULT [];
DEFINE FIELD OVERWRITE questions.* ON TABLE anamnesis_template TYPE any;
DEFINE FIELD OVERWRITE created_at ON TABLE anamnesis_template TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE anamnesis_template TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_anamnesis_template_clinic ON TABLE anamnesis_template COLUMNS clinic_id, template_type;

-- 2. Atualização dos campos de Pacientes (RG e Responsáveis Legais)
DEFINE FIELD OVERWRITE document_rg ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE legal_guardian_name ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE legal_guardian_cpf ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE legal_guardians ON TABLE patient TYPE option<array<object>> FLEXIBLE DEFAULT [];
DEFINE FIELD OVERWRITE legal_guardians.* ON TABLE patient TYPE any;


-- 3. Atualização de Ficha de Anamnese com Respostas Dinâmicas
DEFINE FIELD OVERWRITE template_type ON TABLE patient_anamnesis TYPE option<string>;
DEFINE FIELD OVERWRITE custom_responses ON TABLE patient_anamnesis TYPE array<object> FLEXIBLE DEFAULT [];
DEFINE FIELD OVERWRITE custom_responses.* ON TABLE patient_anamnesis TYPE any;

-- 4. Atualização de Procedimentos Odontológicos / Evolução Clínica com Relacionamentos
DEFINE FIELD OVERWRITE document_id ON TABLE patient_treatment TYPE option<record<patient_document>>;
DEFINE FIELD OVERWRITE exam_id ON TABLE patient_treatment TYPE option<record<patient_exam>>;
DEFINE FIELD OVERWRITE procedure_category ON TABLE patient_treatment TYPE option<string>;
DEFINE FIELD OVERWRITE surfaces ON TABLE patient_treatment TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE materials_used ON TABLE patient_treatment TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE post_care_instructions ON TABLE patient_treatment TYPE option<string>;
DEFINE FIELD OVERWRITE performed_at ON TABLE patient_treatment TYPE datetime DEFAULT time::now();

COMMIT TRANSACTION;
