BEGIN TRANSACTION;

DEFINE TABLE OVERWRITE transaction SCHEMAFULL;

DEFINE FIELD OVERWRITE clinic_id ON transaction TYPE record<clinic>;
DEFINE FIELD OVERWRITE appointment_id ON transaction TYPE option<record<appointment>>;
DEFINE FIELD OVERWRITE patient_id ON transaction TYPE option<record<patient>>;
DEFINE FIELD OVERWRITE user_id ON transaction TYPE option<record<user>>;
DEFINE FIELD OVERWRITE direction ON transaction TYPE string ASSERT $value IN ['income', 'expense'];
DEFINE FIELD OVERWRITE amount_cents ON transaction TYPE int;
DEFINE FIELD OVERWRITE description ON transaction TYPE string;
DEFINE FIELD OVERWRITE category ON transaction TYPE string;
DEFINE FIELD OVERWRITE status ON transaction TYPE string ASSERT $value IN ['pending', 'paid', 'canceled', 'refunded'];
DEFINE FIELD OVERWRITE due_date ON transaction TYPE datetime;
DEFINE FIELD OVERWRITE paid_date ON transaction TYPE option<datetime>;
DEFINE FIELD OVERWRITE payment_method ON transaction TYPE option<string>;
DEFINE FIELD OVERWRITE installment_current ON transaction TYPE int DEFAULT 1;
DEFINE FIELD OVERWRITE installment_total ON transaction TYPE int DEFAULT 1;
DEFINE FIELD OVERWRITE created_at ON transaction TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_transaction_clinic ON transaction COLUMNS clinic_id;
DEFINE INDEX OVERWRITE idx_transaction_due_date ON transaction COLUMNS due_date;

COMMIT TRANSACTION;
