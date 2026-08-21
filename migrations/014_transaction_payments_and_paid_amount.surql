BEGIN TRANSACTION;

DEFINE FIELD OVERWRITE paid_amount_cents ON TABLE transaction TYPE option<int> DEFAULT 0;
DEFINE FIELD OVERWRITE payments ON TABLE transaction TYPE array DEFAULT [];
DEFINE FIELD OVERWRITE payments.* ON TABLE transaction TYPE any;
DEFINE FIELD OVERWRITE updated_at ON TABLE transaction TYPE option<datetime> DEFAULT time::now();
DEFINE FIELD OVERWRITE status ON TABLE transaction TYPE string ASSERT $value IN ['pending', 'paid', 'canceled', 'refunded', 'partial'];

COMMIT TRANSACTION;
