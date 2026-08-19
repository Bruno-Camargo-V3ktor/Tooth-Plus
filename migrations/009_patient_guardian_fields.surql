BEGIN TRANSACTION;

-- Adiciona campos de responsável legal e flexibilização na tabela patient
DEFINE FIELD OVERWRITE legal_guardian_name ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE legal_guardian_cpf ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE document_rg ON TABLE patient TYPE option<string>;
DEFINE FIELD OVERWRITE legal_guardians ON TABLE patient TYPE option<array<object>> FLEXIBLE DEFAULT [];
DEFINE FIELD OVERWRITE legal_guardians.* ON TABLE patient TYPE any;

COMMIT TRANSACTION;
