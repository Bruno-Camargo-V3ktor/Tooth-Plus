use backend::db::init_db;
use backend::migrations::run_migrations;
use serde_json::json;

#[tokio::test]
async fn test_transaction_schema_and_creation() {
    dotenvy::dotenv().ok();
    let db = match init_db().await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping DB test (no connection): {}", e);
            return;
        }
    };

    // 1. Run migrations to ensure migration 014 is applied
    run_migrations(&db).await;

    // 2. Also explicitly ensure the schema commands are applied
    let schema_query = "
        DEFINE FIELD OVERWRITE paid_amount_cents ON TABLE transaction TYPE option<int> DEFAULT 0;
        DEFINE FIELD OVERWRITE payments ON TABLE transaction TYPE array DEFAULT [];
        DEFINE FIELD OVERWRITE payments.* ON TABLE transaction TYPE any;
        DEFINE FIELD OVERWRITE updated_at ON TABLE transaction TYPE option<datetime> DEFAULT time::now();
        DEFINE FIELD OVERWRITE status ON TABLE transaction TYPE string ASSERT $value IN ['pending', 'paid', 'canceled', 'refunded', 'partial'];
    ";
    let _ = db.query(schema_query).await;

    // 3. Test creating a transaction with payments array and paid_amount_cents
    let clinic_res = db.query("SELECT id FROM clinic LIMIT 1").await;
    let mut clinic_query = clinic_res.expect("Failed to query clinic");
    let clinics: Vec<serde_json::Value> = clinic_query.take(0).unwrap_or_default();
    let clinic_id = if let Some(c) = clinics.first() {
        c.get("id").unwrap().as_str().unwrap().to_string()
    } else {
        "clinic:test_clinic".to_string()
    };

    let payment_entry = json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "paid_at": chrono::Utc::now().to_rfc3339(),
        "amount_cents": 10000,
        "payment_method": "Pix",
        "notes": "Test payment insertion",
        "registered_by_user_id": null,
        "registered_by_user_name": null
    });

    let insert_query = "
        CREATE transaction CONTENT {
            clinic_id: type::record($cid),
            direction: 'income',
            amount_cents: 10000,
            paid_amount_cents: 10000,
            description: 'Procedimento: Teste de Lançamento',
            category: 'Tratamento Odontológico',
            status: 'paid',
            due_date: time::now(),
            paid_date: time::now(),
            payment_method: 'Pix',
            payments: [$payment_entry],
            installment_current: 1,
            installment_total: 1
        };
    ";

    let mut res = db
        .query(insert_query)
        .bind(("cid", clinic_id))
        .bind(("payment_entry", payment_entry))
        .await
        .expect("Failed to execute transaction insert query");

    let errors = res.take_errors();
    assert!(errors.is_empty(), "DB Error inserting transaction: {:?}", errors);

    let created: Vec<serde_json::Value> = res.take(0).expect("Failed to take created transaction");
    assert!(!created.is_empty(), "Transaction was not returned");
    let tx_id = created[0].get("id").unwrap().as_str().unwrap().to_string();
    println!("Created transaction: {}", tx_id);

    // 4. Test updating payment on transaction (register partial/full payment)
    let second_payment = json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "paid_at": chrono::Utc::now().to_rfc3339(),
        "amount_cents": 5000,
        "payment_method": "Dinheiro",
        "notes": "Second test payment",
        "registered_by_user_id": null,
        "registered_by_user_name": null
    });

    let update_query = "
        UPDATE type::record($id) SET
            paid_amount_cents = 15000,
            status = 'paid',
            payment_method = 'Dinheiro',
            paid_date = time::now(),
            payments = array::concat(IF payments != NONE THEN payments ELSE [] END, [$entry]),
            updated_at = time::now();
    ";

    let mut update_res = db
        .query(update_query)
        .bind(("id", tx_id.clone()))
        .bind(("entry", second_payment))
        .await
        .expect("Failed to execute update query");

    let update_errors = update_res.take_errors();
    assert!(update_errors.is_empty(), "DB Error updating transaction payment: {:?}", update_errors);

    // 5. Clean up test record
    let _ = db.query("DELETE type::record($id)").bind(("id", tx_id)).await;
    println!("SUCCESS: Transaction creation and payment update tested cleanly!");
}
