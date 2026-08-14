BEGIN TRANSACTION;

DEFINE TABLE OVERWRITE patient SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON patient TYPE record<clinic>;
DEFINE FIELD OVERWRITE full_name ON patient TYPE string;
DEFINE FIELD OVERWRITE document_cpf ON patient TYPE option<string>;
DEFINE FIELD OVERWRITE phone ON patient TYPE option<string>;
DEFINE FIELD OVERWRITE email ON patient TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON patient TYPE datetime DEFAULT time::now();

DEFINE TABLE OVERWRITE inventory_item SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON inventory_item TYPE record<clinic>;
DEFINE FIELD OVERWRITE name ON inventory_item TYPE string;
DEFINE FIELD OVERWRITE unit ON inventory_item TYPE string;
DEFINE FIELD OVERWRITE current_stock ON inventory_item TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE min_stock ON inventory_item TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE cost_cents ON inventory_item TYPE int DEFAULT 0;

DEFINE TABLE OVERWRITE stock_movement SCHEMAFULL;
DEFINE FIELD OVERWRITE item_id ON stock_movement TYPE record<inventory_item>;
DEFINE FIELD OVERWRITE quantity_change ON stock_movement TYPE int;
DEFINE FIELD OVERWRITE movement_type ON stock_movement TYPE string;
DEFINE FIELD OVERWRITE created_at ON stock_movement TYPE datetime DEFAULT time::now();

DEFINE TABLE OVERWRITE financial_transaction SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON financial_transaction TYPE record<clinic>;
DEFINE FIELD OVERWRITE appointment_id ON financial_transaction TYPE option<record<appointment>>;
DEFINE FIELD OVERWRITE user_id ON financial_transaction TYPE option<record<user>>;
DEFINE FIELD OVERWRITE amount_cents ON financial_transaction TYPE int;
DEFINE FIELD OVERWRITE direction ON financial_transaction TYPE string ASSERT $value INSIDE ['income', 'expense'];
DEFINE FIELD OVERWRITE status ON financial_transaction TYPE string ASSERT $value INSIDE ['pending', 'completed', 'canceled'];
DEFINE FIELD OVERWRITE category ON financial_transaction TYPE string;
DEFINE FIELD OVERWRITE description ON financial_transaction TYPE string;
DEFINE FIELD OVERWRITE created_at ON financial_transaction TYPE datetime DEFAULT time::now();

DEFINE TABLE OVERWRITE appointment SCHEMAFULL;
DEFINE FIELD OVERWRITE clinic_id ON appointment TYPE record<clinic>;
DEFINE FIELD OVERWRITE patient_id ON appointment TYPE option<record<patient>>;
DEFINE FIELD OVERWRITE patient_name ON appointment TYPE option<string>;
DEFINE FIELD OVERWRITE title ON appointment TYPE string;
DEFINE FIELD OVERWRITE scheduled_for ON appointment TYPE datetime;
DEFINE FIELD OVERWRITE duration_minutes ON appointment TYPE int DEFAULT 30;
DEFINE FIELD OVERWRITE status ON appointment TYPE string ASSERT $value INSIDE ['pending', 'confirmed', 'in_progress', 'completed', 'canceled', 'no_show'];
DEFINE FIELD OVERWRITE appointment_type ON appointment TYPE string ASSERT $value INSIDE ['consultation', 'treatment', 'surgery', 'return', 'meeting', 'other'];
DEFINE FIELD OVERWRITE financial_amount_cents ON appointment TYPE option<int>;
DEFINE FIELD OVERWRITE financial_type ON appointment TYPE option<string> ASSERT $value == NONE OR $value INSIDE ['income', 'expense'];
DEFINE FIELD OVERWRITE cancellation_reason ON appointment TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON appointment TYPE datetime DEFAULT time::now();

DEFINE TABLE OVERWRITE assigned_to SCHEMAFULL TYPE RELATION FROM appointment TO user;
DEFINE FIELD OVERWRITE role_in_appointment ON assigned_to TYPE string;
DEFINE FIELD OVERWRITE split_percentage ON assigned_to TYPE int ASSERT $value >= 0 AND $value <= 100;
DEFINE FIELD OVERWRITE assigned_at ON assigned_to TYPE datetime DEFAULT time::now();

DEFINE TABLE OVERWRITE consumes SCHEMAFULL TYPE RELATION FROM appointment TO inventory_item;
DEFINE FIELD OVERWRITE quantity_planned ON consumes TYPE int ASSERT $value >= 0;
DEFINE FIELD OVERWRITE quantity_used ON consumes TYPE option<int>;

COMMIT TRANSACTION;
