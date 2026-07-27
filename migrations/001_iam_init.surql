BEGIN TRANSACTION;

DEFINE TABLE institution SCHEMAFULL;
DEFINE FIELD name ON institution TYPE string;
DEFINE FIELD active ON institution TYPE bool VALUE true;

DEFINE TABLE clinic SCHEMAFULL;
DEFINE FIELD institution ON clinic TYPE record<institution>;
DEFINE FIELD trading_name ON clinic TYPE string;
DEFINE FIELD document_cnpj ON clinic TYPE string;
DEFINE FIELD theme_color ON clinic TYPE string VALUE "#0052cc";
DEFINE FIELD is_active ON clinic TYPE bool VALUE true;


DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username ON user TYPE string ASSERT string::lowercase($value) AND string::is::alphanum($value);
DEFINE INDEX idx_user_username ON user COLUMNS username UNIQUE;
DEFINE FIELD password_hash ON user TYPE string;
DEFINE FIELD full_name ON user TYPE string;
DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value);
DEFINE INDEX idx_user_email ON user COLUMNS email UNIQUE;


DEFINE TABLE works_at SCHEMAFULL TYPE RELATION FROM user TO clinic;
DEFINE FIELD role ON works_at TYPE string ASSERT $value INSIDE ['admin', 'dentist', 'receptionist'];
DEFINE FIELD assigned_at ON works_at TYPE datetime VALUE time::now();

COMMIT TRANSACTION;
