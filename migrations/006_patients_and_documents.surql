BEGIN TRANSACTION;

DEFINE TABLE OVERWRITE patient SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON TABLE patient TYPE record<clinic>;
DEFINE FIELD OVERWRITE full_name ON TABLE patient TYPE string;
DEFINE FIELD OVERWRITE document_cpf ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE document_cpf_encrypted ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE document_cpf_hash ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE phone ON TABLE patient TYPE string;
DEFINE FIELD OVERWRITE email ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE birth_date ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE gender ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE marital_status ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE profession ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE emergency_contact_name ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE emergency_contact_phone ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE address_street ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE address_number ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE address_complement ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE address_neighborhood ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE address_city ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE address_state ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE address_zip ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE insurance_plan ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE insurance_number ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE password_hash ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON TABLE patient TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE patient TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_patient_clinic ON TABLE patient COLUMNS clinic_id;
DEFINE INDEX OVERWRITE idx_patient_cpf_hash ON TABLE patient COLUMNS clinic_id, document_cpf_hash;

DEFINE TABLE OVERWRITE patient_anamnesis SCHEMAFULL;
DEFINE FIELD OVERWRITE patient_id ON TABLE patient_anamnesis TYPE record<patient>;
DEFINE FIELD OVERWRITE clinic_id ON TABLE patient_anamnesis TYPE record<clinic>;
DEFINE FIELD OVERWRITE allergies ON TABLE patient_anamnesis TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE continuous_medications ON TABLE patient_anamnesis TYPE option<string>;
DEFINE FIELD OVERWRITE systemic_diseases ON TABLE patient_anamnesis TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE is_pregnant ON TABLE patient_anamnesis TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE has_bleeding_disorder ON TABLE patient_anamnesis TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE smoker ON TABLE patient_anamnesis TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE bruxism ON TABLE patient_anamnesis TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE chief_complaint ON TABLE patient_anamnesis TYPE option<string>;
DEFINE FIELD OVERWRITE clinical_notes ON TABLE patient_anamnesis TYPE option<string>;
DEFINE FIELD OVERWRITE signature_status ON TABLE patient_anamnesis TYPE option<string> DEFAULT "pending";
DEFINE FIELD OVERWRITE signing_token ON TABLE patient_anamnesis TYPE option<string>;
DEFINE FIELD OVERWRITE signed_at ON TABLE patient_anamnesis TYPE option<datetime>;
DEFINE FIELD OVERWRITE signed_pdf_url ON TABLE patient_anamnesis TYPE option<string>;
DEFINE FIELD OVERWRITE updated_at ON TABLE patient_anamnesis TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_anamnesis_patient ON TABLE patient_anamnesis COLUMNS patient_id UNIQUE;

DEFINE TABLE OVERWRITE patient_exam SCHEMAFULL;
DEFINE FIELD OVERWRITE patient_id ON TABLE patient_exam TYPE record<patient>;
DEFINE FIELD OVERWRITE clinic_id ON TABLE patient_exam TYPE record<clinic>;
DEFINE FIELD OVERWRITE title ON TABLE patient_exam TYPE string;
DEFINE FIELD OVERWRITE exam_type ON TABLE patient_exam TYPE string;
DEFINE FIELD OVERWRITE requested_by_user_id ON TABLE patient_exam TYPE option<record<user>>;
DEFINE FIELD OVERWRITE status ON TABLE patient_exam TYPE string DEFAULT "requested";
DEFINE FIELD OVERWRITE requested_date ON TABLE patient_exam TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE result_date ON TABLE patient_exam TYPE option<datetime>;
DEFINE FIELD OVERWRITE file_urls ON TABLE patient_exam TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE clinical_interpretation ON TABLE patient_exam TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON TABLE patient_exam TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_exam_patient ON TABLE patient_exam COLUMNS patient_id;

DEFINE TABLE OVERWRITE patient_treatment SCHEMAFULL;
DEFINE FIELD OVERWRITE patient_id ON TABLE patient_treatment TYPE record<patient>;
DEFINE FIELD OVERWRITE clinic_id ON TABLE patient_treatment TYPE record<clinic>;
DEFINE FIELD OVERWRITE dentist_user_id ON TABLE patient_treatment TYPE option<record<user>>;
DEFINE FIELD OVERWRITE appointment_id ON TABLE patient_treatment TYPE option<record<appointment>>;
DEFINE FIELD OVERWRITE procedure_name ON TABLE patient_treatment TYPE string;
DEFINE FIELD OVERWRITE tooth_number ON TABLE patient_treatment TYPE option<string>;
DEFINE FIELD OVERWRITE status ON TABLE patient_treatment TYPE string DEFAULT "planned";
DEFINE FIELD OVERWRITE cost_cents ON TABLE patient_treatment TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE clinical_notes ON TABLE patient_treatment TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON TABLE patient_treatment TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_treatment_patient ON TABLE patient_treatment COLUMNS patient_id;

DEFINE TABLE OVERWRITE contract_template SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON TABLE contract_template TYPE record<clinic>;
DEFINE FIELD OVERWRITE title ON TABLE contract_template TYPE string;
DEFINE FIELD OVERWRITE category ON TABLE contract_template TYPE string DEFAULT "contract";
DEFINE FIELD OVERWRITE description ON TABLE contract_template TYPE option<string>;
DEFINE FIELD OVERWRITE pdf_url ON TABLE contract_template TYPE string;
DEFINE FIELD OVERWRITE signature_fields ON TABLE contract_template TYPE array<object> FLEXIBLE DEFAULT [];
DEFINE FIELD OVERWRITE signature_fields.* ON TABLE contract_template TYPE any;
DEFINE FIELD OVERWRITE requires_patient_signature ON TABLE contract_template TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE requires_doctor_signature ON TABLE contract_template TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE allow_any_dentist_signature ON TABLE contract_template TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE created_at ON TABLE contract_template TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE contract_template TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_template_clinic ON TABLE contract_template COLUMNS clinic_id;

DEFINE TABLE OVERWRITE patient_document SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON TABLE patient_document TYPE record<clinic>;
DEFINE FIELD OVERWRITE patient_id ON TABLE patient_document TYPE record<patient>;
DEFINE FIELD OVERWRITE template_id ON TABLE patient_document TYPE option<record<contract_template>>;
DEFINE FIELD OVERWRITE doctor_user_id ON TABLE patient_document TYPE option<record<user>>;
DEFINE FIELD OVERWRITE appointment_id ON TABLE patient_document TYPE option<record<appointment>>;
DEFINE FIELD OVERWRITE title ON TABLE patient_document TYPE string;
DEFINE FIELD OVERWRITE document_type ON TABLE patient_document TYPE string DEFAULT "contract";
DEFINE FIELD OVERWRITE original_pdf_url ON TABLE patient_document TYPE string;
DEFINE FIELD OVERWRITE signed_pdf_url ON TABLE patient_document TYPE option<string>;
DEFINE FIELD OVERWRITE status ON TABLE patient_document TYPE string DEFAULT "pending_signatures";
DEFINE FIELD OVERWRITE signing_token ON TABLE patient_document TYPE string;
DEFINE FIELD OVERWRITE requires_patient_signature ON TABLE patient_document TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE requires_doctor_signature ON TABLE patient_document TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE allow_any_dentist_signature ON TABLE patient_document TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE patient_signed_at ON TABLE patient_document TYPE option<datetime>;
DEFINE FIELD OVERWRITE patient_signature_data ON TABLE patient_document TYPE option<string>;
DEFINE FIELD OVERWRITE doctor_signed_at ON TABLE patient_document TYPE option<datetime>;
DEFINE FIELD OVERWRITE doctor_signature_data ON TABLE patient_document TYPE option<string>;
DEFINE FIELD OVERWRITE require_whatsapp_otp ON TABLE patient_document TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE patient_otp_verified ON TABLE patient_document TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE otp_code_hash ON TABLE patient_document TYPE option<string>;
DEFINE FIELD OVERWRITE otp_expires_at ON TABLE patient_document TYPE option<datetime>;
DEFINE FIELD OVERWRITE final_checksum_sha256 ON TABLE patient_document TYPE option<string>;
DEFINE FIELD OVERWRITE audit_trail ON TABLE patient_document TYPE array<object> FLEXIBLE DEFAULT [];
DEFINE FIELD OVERWRITE audit_trail.* ON TABLE patient_document TYPE any;
DEFINE FIELD OVERWRITE created_at ON TABLE patient_document TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE patient_document TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_patient_doc_clinic ON TABLE patient_document COLUMNS clinic_id;
DEFINE INDEX OVERWRITE idx_patient_doc_patient ON TABLE patient_document COLUMNS patient_id;
DEFINE INDEX OVERWRITE idx_patient_doc_token ON TABLE patient_document COLUMNS signing_token UNIQUE;

COMMIT TRANSACTION;
