BEGIN TRANSACTION;


DEFINE TABLE patient SCHEMAFULL;
DEFINE FIELD full_name ON patient TYPE string;
DEFINE FIELD cpf ON patient TYPE string;
DEFINE FIELD date_of_birth ON patient TYPE datetime;
DEFINE FIELD phone ON patient TYPE string;
DEFINE FIELD address ON patient TYPE object;
DEFINE FIELD address.street ON patient TYPE string;
DEFINE FIELD address.city ON patient TYPE string;
DEFINE FIELD address.postal_code ON patient TYPE string;

DEFINE FIELD clinic ON patient TYPE record<clinic>;
DEFINE INDEX idx_patient_cpf ON patient COLUMNS cpf, clinic UNIQUE;


DEFINE TABLE chair SCHEMAFULL;
DEFINE FIELD name ON chair TYPE string;
DEFINE FIELD is_active ON chair TYPE bool VALUE true;
DEFINE FIELD clinic ON chair TYPE record<clinic>;


DEFINE TABLE procedure_catalog SCHEMAFULL;
DEFINE FIELD name ON procedure_catalog TYPE string;
DEFINE FIELD base_price ON procedure_catalog TYPE decimal;
DEFINE FIELD clinic ON procedure_catalog TYPE record<clinic>;

COMMIT TRANSACTION;
