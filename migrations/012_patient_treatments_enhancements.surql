-- =========================================================================
-- MIGRATION 012: PATIENT TREATMENTS & TREATMENT PLANS ENHANCEMENTS
-- =========================================================================

-- patient_treatment fields
DEFINE FIELD OVERWRITE updated_at ON TABLE patient_treatment TYPE option<datetime> DEFAULT time::now();
DEFINE FIELD OVERWRITE treatment_plan_item_id ON TABLE patient_treatment TYPE option<string>;
DEFINE FIELD OVERWRITE financial_status ON TABLE patient_treatment TYPE option<string>;
DEFINE FIELD OVERWRITE appointment_id ON TABLE patient_treatment TYPE option<record<appointment>>;
DEFINE FIELD OVERWRITE dentist_user_id ON TABLE patient_treatment TYPE option<record<user>>;
DEFINE FIELD OVERWRITE treatment_plan_id ON TABLE patient_treatment TYPE option<record<patient_treatment_plan>>;
DEFINE FIELD OVERWRITE transaction_id ON TABLE patient_treatment TYPE option<record<transaction>>;

-- patient_treatment_plan fields
DEFINE FIELD OVERWRITE paid_amount_cents ON TABLE patient_treatment_plan TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE remaining_amount_cents ON TABLE patient_treatment_plan TYPE option<int>;
DEFINE FIELD OVERWRITE financial_status ON TABLE patient_treatment_plan TYPE option<string>;
DEFINE FIELD OVERWRITE payments ON TABLE patient_treatment_plan TYPE array DEFAULT [];
DEFINE FIELD OVERWRITE payments.* ON TABLE patient_treatment_plan TYPE any;

