BEGIN TRANSACTION;

DEFINE TABLE OVERWRITE inventory_item SCHEMAFULL;
REMOVE FIELD unit ON inventory_item;
REMOVE FIELD cost_cents ON inventory_item;
DEFINE FIELD OVERWRITE clinic_id ON inventory_item TYPE record<clinic>;
DEFINE FIELD OVERWRITE item_type ON inventory_item TYPE string ASSERT $value INSIDE ['material', 'chemical', 'equipment'];
DEFINE FIELD OVERWRITE name ON inventory_item TYPE string;
DEFINE FIELD OVERWRITE unit_type ON inventory_item TYPE string;
DEFINE FIELD OVERWRITE current_stock ON inventory_item TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE min_stock ON inventory_item TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE cost_price_cents ON inventory_item TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE manufacturer ON inventory_item TYPE option<string>;
DEFINE FIELD OVERWRITE attachments ON inventory_item TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE expiration_date ON inventory_item TYPE option<datetime>;
DEFINE FIELD OVERWRITE batch_number ON inventory_item TYPE option<string>;
DEFINE FIELD OVERWRITE serial_number ON inventory_item TYPE option<string>;
DEFINE FIELD OVERWRITE warranty_until ON inventory_item TYPE option<datetime>;
DEFINE FIELD OVERWRITE next_maintenance_date ON inventory_item TYPE option<datetime>;
DEFINE FIELD OVERWRITE equipment_status ON inventory_item TYPE option<string> ASSERT $value == NONE OR $value INSIDE ['active', 'in_maintenance', 'broken'];
DEFINE FIELD OVERWRITE created_at ON inventory_item TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON inventory_item TYPE option<datetime>;

DEFINE TABLE OVERWRITE stock_movement SCHEMAFULL;
DEFINE FIELD OVERWRITE item_id ON stock_movement TYPE record<inventory_item>;
DEFINE FIELD OVERWRITE clinic_id ON stock_movement TYPE record<clinic>;
DEFINE FIELD OVERWRITE user_id ON stock_movement TYPE option<record<user>>;
DEFINE FIELD OVERWRITE quantity_change ON stock_movement TYPE int;
DEFINE FIELD OVERWRITE movement_type ON stock_movement TYPE string ASSERT $value INSIDE ['purchase_in', 'manual_out', 'appointment_out', 'adjustment', 'loss'];
DEFINE FIELD OVERWRITE unit_cost_cents ON stock_movement TYPE option<int>;
DEFINE FIELD OVERWRITE invoice_number ON stock_movement TYPE option<string>;
DEFINE FIELD OVERWRITE notes ON stock_movement TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON stock_movement TYPE datetime DEFAULT time::now();

COMMIT TRANSACTION;
