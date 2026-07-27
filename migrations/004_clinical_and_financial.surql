BEGIN TRANSACTION;

DEFINE TABLE budget SCHEMAFULL;
DEFINE FIELD patient ON budget TYPE record<patient>;
DEFINE FIELD clinic ON budget TYPE record<clinic>;
DEFINE FIELD created_by ON budget TYPE record<user>;
DEFINE FIELD status ON budget TYPE string ASSERT $value INSIDE ['pending', 'approved', 'rejected'];
DEFINE FIELD total_amount ON budget TYPE decimal;
DEFINE FIELD created_at ON budget TYPE datetime VALUE time::now();


DEFINE TABLE budget_item SCHEMAFULL TYPE RELATION FROM budget TO procedure_catalog;
DEFINE FIELD quantity ON budget_item TYPE number;
DEFINE FIELD unit_price ON budget_item TYPE decimal;
DEFINE FIELD tooth_info ON budget_item TYPE option<string>;


DEFINE TABLE treatment_evolution SCHEMAFULL;
DEFINE FIELD patient ON treatment_evolution TYPE record<patient>;
DEFINE FIELD dentist ON treatment_evolution TYPE record<user>;
DEFINE FIELD clinic ON treatment_evolution TYPE record<clinic>;
DEFINE FIELD description ON treatment_evolution TYPE string;
DEFINE FIELD performed_at ON treatment_evolution TYPE datetime VALUE time::now();

DEFINE EVENT prevent_evolution_delete ON treatment_evolution WHEN $event = "DELETE" THEN (
    THROW "Operação Negada: O registo clínico é imutável."
);


DEFINE TABLE financial_transaction SCHEMAFULL;
DEFINE FIELD patient ON financial_transaction TYPE record<patient>;
DEFINE FIELD clinic ON financial_transaction TYPE record<clinic>;
DEFINE FIELD type ON financial_transaction TYPE string ASSERT $value INSIDE ['charge', 'payment'];
DEFINE FIELD amount ON financial_transaction TYPE decimal;
DEFINE FIELD payment_method ON financial_transaction TYPE option<string>; -- 'pix', 'credit_card', 'cash'
DEFINE FIELD reference_date ON financial_transaction TYPE datetime VALUE time::now();
DEFINE FIELD receipt_issued ON financial_transaction TYPE bool VALUE false;

COMMIT TRANSACTION;
