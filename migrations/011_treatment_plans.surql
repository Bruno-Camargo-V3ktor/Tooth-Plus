BEGIN TRANSACTION;

-- =========================================================================
-- MIGRATION 011: TREATMENT TEMPLATES, PLANS & FINANCIAL INTEGRATION
-- =========================================================================

DEFINE TABLE OVERWRITE treatment_template SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON TABLE treatment_template TYPE record<clinic>;
DEFINE FIELD OVERWRITE name ON TABLE treatment_template TYPE string;
DEFINE FIELD OVERWRITE category ON TABLE treatment_template TYPE option<string>;
DEFINE FIELD OVERWRITE description ON TABLE treatment_template TYPE option<string>;
DEFINE FIELD OVERWRITE default_price_cents ON TABLE treatment_template TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE estimated_duration_minutes ON TABLE treatment_template TYPE option<int>;
DEFINE FIELD OVERWRITE dental_regions ON TABLE treatment_template TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE target_teeth ON TABLE treatment_template TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE required_materials ON TABLE treatment_template TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE required_equipment ON TABLE treatment_template TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE post_care_instructions ON TABLE treatment_template TYPE option<string>;
DEFINE FIELD OVERWRITE clinical_notes ON TABLE treatment_template TYPE option<string>;
DEFINE FIELD OVERWRITE is_active ON TABLE treatment_template TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE created_at ON TABLE treatment_template TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE treatment_template TYPE datetime DEFAULT time::now();
DEFINE INDEX OVERWRITE idx_treatment_template_clinic ON TABLE treatment_template COLUMNS clinic_id;

DEFINE TABLE OVERWRITE patient_treatment_plan SCHEMAFULL;
DEFINE FIELD OVERWRITE patient_id ON TABLE patient_treatment_plan TYPE record<patient>;
DEFINE FIELD OVERWRITE clinic_id ON TABLE patient_treatment_plan TYPE record<clinic>;
DEFINE FIELD OVERWRITE dentist_user_id ON TABLE patient_treatment_plan TYPE option<record<user>>;
DEFINE FIELD OVERWRITE transaction_id ON TABLE patient_treatment_plan TYPE option<record<transaction>>;
DEFINE FIELD OVERWRITE title ON TABLE patient_treatment_plan TYPE string;
DEFINE FIELD OVERWRITE status ON TABLE patient_treatment_plan TYPE string DEFAULT 'draft';
DEFINE FIELD OVERWRITE items ON TABLE patient_treatment_plan TYPE array DEFAULT [];
DEFINE FIELD OVERWRITE items.* ON TABLE patient_treatment_plan TYPE any;
DEFINE FIELD OVERWRITE total_price_cents ON TABLE patient_treatment_plan TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE notes ON TABLE patient_treatment_plan TYPE option<string>;
DEFINE FIELD OVERWRITE planned_start_date ON TABLE patient_treatment_plan TYPE option<string>;
DEFINE FIELD OVERWRITE planned_end_date ON TABLE patient_treatment_plan TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON TABLE patient_treatment_plan TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE patient_treatment_plan TYPE datetime DEFAULT time::now();
DEFINE INDEX OVERWRITE idx_plan_patient ON TABLE patient_treatment_plan COLUMNS patient_id;
DEFINE INDEX OVERWRITE idx_plan_clinic ON TABLE patient_treatment_plan COLUMNS clinic_id;

DEFINE FIELD OVERWRITE treatment_plan_id ON TABLE transaction TYPE option<record<patient_treatment_plan>>;
DEFINE FIELD OVERWRITE treatment_plan_id ON TABLE patient_treatment TYPE option<record<patient_treatment_plan>>;
DEFINE FIELD OVERWRITE transaction_id ON TABLE patient_treatment TYPE option<record<transaction>>;

COMMIT TRANSACTION;
