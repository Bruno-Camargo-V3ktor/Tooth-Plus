BEGIN TRANSACTION;

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username ON user TYPE string;
DEFINE FIELD password_hash ON user TYPE string;
DEFINE FIELD full_name ON user TYPE string;
DEFINE FIELD document_cpf ON user TYPE string;
DEFINE FIELD professional_registry ON user TYPE option<string>;
DEFINE FIELD is_active ON user TYPE bool VALUE true;
DEFINE FIELD created_at ON user TYPE datetime VALUE time::now();

DEFINE INDEX idx_username ON user COLUMNS username UNIQUE;
DEFINE INDEX idx_user_cpf ON user COLUMNS document_cpf UNIQUE;

DEFINE TABLE works_at SCHEMAFULL TYPE RELATION FROM user TO clinic;
DEFINE FIELD role ON works_at TYPE string;
DEFINE FIELD permissions ON works_at TYPE array<string>;
DEFINE FIELD assigned_at ON works_at TYPE datetime VALUE time::now();

COMMIT TRANSACTION;
