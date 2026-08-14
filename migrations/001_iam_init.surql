BEGIN TRANSACTION;

DEFINE TABLE OVERWRITE user SCHEMAFULL;
DEFINE FIELD OVERWRITE username ON user TYPE string;
DEFINE FIELD OVERWRITE password_hash ON user TYPE string;
DEFINE FIELD OVERWRITE full_name ON user TYPE string;
DEFINE FIELD OVERWRITE document_cpf ON user TYPE string;
DEFINE FIELD OVERWRITE professional_registry ON user TYPE option<string>;
DEFINE FIELD OVERWRITE is_active ON user TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE created_at ON user TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_username ON user COLUMNS username UNIQUE;
DEFINE INDEX OVERWRITE idx_user_cpf ON user COLUMNS document_cpf UNIQUE;

DEFINE TABLE OVERWRITE works_at SCHEMAFULL TYPE RELATION FROM user TO clinic;
DEFINE FIELD OVERWRITE role ON works_at TYPE string;
DEFINE FIELD OVERWRITE permissions ON works_at TYPE array<string>;
DEFINE FIELD OVERWRITE assigned_at ON works_at TYPE datetime DEFAULT time::now();

COMMIT TRANSACTION;
