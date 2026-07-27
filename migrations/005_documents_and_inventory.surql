BEGIN TRANSACTION;

DEFINE TABLE signed_document SCHEMAFULL;
DEFINE FIELD patient ON signed_document TYPE record<patient>;
DEFINE FIELD clinic ON signed_document TYPE record<clinic>;
DEFINE FIELD document_type ON signed_document TYPE string ASSERT $value INSIDE ['contract', 'consent_form', 'prescription', 'certificate'];
DEFINE FIELD storage_url ON signed_document TYPE string;
DEFINE FIELD document_hash ON signed_document TYPE string;
DEFINE FIELD signer_ip ON signed_document TYPE string;
DEFINE FIELD signed_at ON signed_document TYPE datetime VALUE time::now();

DEFINE EVENT prevent_doc_update ON signed_document WHEN $event = "UPDATE" THEN (
    THROW "Operação Negada: Documentos assinados são imutáveis."
);
DEFINE EVENT prevent_doc_delete ON signed_document WHEN $event = "DELETE" THEN (
    THROW "Operação Negada: Documentos não podem ser eliminados."
);


DEFINE TABLE inventory_item SCHEMAFULL;
DEFINE FIELD name ON inventory_item TYPE string;
DEFINE FIELD current_quantity ON inventory_item TYPE number;
DEFINE FIELD clinic ON inventory_item TYPE record<clinic>;


DEFINE TABLE stock_movement SCHEMAFULL TYPE RELATION FROM user TO inventory_item;
DEFINE FIELD movement_type ON stock_movement TYPE string ASSERT $value INSIDE ['in', 'out'];
DEFINE FIELD quantity ON stock_movement TYPE number;
DEFINE FIELD patient_context ON stock_movement TYPE option<record<patient>>;
DEFINE FIELD date ON stock_movement TYPE datetime VALUE time::now();

COMMIT TRANSACTION;
