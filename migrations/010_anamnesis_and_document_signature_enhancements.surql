-- Campos para controle dinâmico de assinaturas em patient_document
DEFINE FIELD OVERWRITE requires_patient_signature ON TABLE patient_document TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE requires_doctor_signature ON TABLE patient_document TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE allow_any_dentist_signature ON TABLE patient_document TYPE bool DEFAULT true;

-- Campos para controle dinâmico de assinaturas em contract_template
DEFINE FIELD OVERWRITE requires_patient_signature ON TABLE contract_template TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE requires_doctor_signature ON TABLE contract_template TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE allow_any_dentist_signature ON TABLE contract_template TYPE bool DEFAULT true;

-- Campos de estado e token de assinatura na ficha de anamnese
DEFINE FIELD OVERWRITE signature_status ON TABLE patient_anamnesis TYPE option<string> DEFAULT "pending";
DEFINE FIELD OVERWRITE signing_token ON TABLE patient_anamnesis TYPE option<string>;
DEFINE FIELD OVERWRITE signed_at ON TABLE patient_anamnesis TYPE option<datetime>;
DEFINE FIELD OVERWRITE signed_pdf_url ON TABLE patient_anamnesis TYPE option<string>;

