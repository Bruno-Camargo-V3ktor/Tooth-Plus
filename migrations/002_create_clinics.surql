BEGIN TRANSACTION;

DEFINE TABLE OVERWRITE clinic SCHEMAFULL;

DEFINE FIELD OVERWRITE corporate_name ON clinic TYPE string;
DEFINE FIELD OVERWRITE trading_name ON clinic TYPE string;
DEFINE FIELD OVERWRITE document_cnpj ON clinic TYPE string;
DEFINE FIELD OVERWRITE theme_color ON clinic TYPE string;
DEFINE FIELD OVERWRITE logo_url ON clinic TYPE option<string>;
DEFINE FIELD OVERWRITE whatsapp_instance ON clinic TYPE option<string>;

DEFINE FIELD OVERWRITE street ON clinic TYPE string;
DEFINE FIELD OVERWRITE number ON clinic TYPE string;
DEFINE FIELD OVERWRITE complement ON clinic TYPE option<string>;
DEFINE FIELD OVERWRITE neighborhood ON clinic TYPE string;
DEFINE FIELD OVERWRITE city ON clinic TYPE string;
DEFINE FIELD OVERWRITE state ON clinic TYPE string;
DEFINE FIELD OVERWRITE zip_code ON clinic TYPE string;

DEFINE FIELD OVERWRITE auto_reminders ON clinic TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE require_esign ON clinic TYPE bool DEFAULT true;

DEFINE FIELD OVERWRITE created_at ON clinic TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE idx_clinic_cnpj ON clinic COLUMNS document_cnpj UNIQUE;

COMMIT TRANSACTION;
