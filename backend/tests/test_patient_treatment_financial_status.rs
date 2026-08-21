use backend::db::init_db;
use surrealdb::types::RecordId;
use serde_json::Value;


#[tokio::test]
async fn test_patient_treatment_crud_financial_status() {
    dotenvy::dotenv().ok();
    let db = match init_db().await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping DB test: {}", e);
            return;
        }
    };

    // 1. Find a clinic and a patient
    let mut c_res = db.query("SELECT id FROM clinic LIMIT 1; SELECT id FROM patient LIMIT 1;").await.expect("query failed");
    let clinics: Vec<Value> = c_res.take(0).unwrap_or_default();
    let patients: Vec<Value> = c_res.take(1).unwrap_or_default();

    if clinics.is_empty() || patients.is_empty() {
        println!("No clinic or patient found to run integration test.");
        return;
    }

    let clinic_id = clinics[0]["id"].as_str().unwrap();
    let patient_id = patients[0]["id"].as_str().unwrap();

    let cid_rec = RecordId::new("clinic", clinic_id.strip_prefix("clinic:").unwrap());
    let pid_rec = RecordId::new("patient", patient_id.strip_prefix("patient:").unwrap());

    // 2. Create treatment with financial_status = 'paid'
    let mut create_res = db
        .query(
            "CREATE patient_treatment SET
                patient_id = $pid,
                clinic_id = $cid,
                procedure_name = 'Teste Financeiro',
                status = 'completed',
                cost_cents = 25000,
                financial_status = 'paid',
                created_at = time::now();",
        )
        .bind(("pid", pid_rec.clone()))
        .bind(("cid", cid_rec.clone()))
        .await
        .expect("create query failed");

    let created_vec: Vec<Value> = create_res.take(0).expect("take created");
    assert!(!created_vec.is_empty(), "Created treatment should not be empty");
    let created = &created_vec[0];
    let created_id_str = created["id"].as_str().unwrap();
    assert_eq!(created["financial_status"].as_str(), Some("paid"), "financial_status should be paid on creation");
    println!("Created treatment: {} with financial_status: {:}s", created_id_str, created["financial_status"]);

    let treat_rec = RecordId::new("patient_treatment", created_id_str.strip_prefix("patient_treatment:").unwrap());

    // 3. Update financial_status to 'unpaid'
    let fin_status: Option<String> = Some("unpaid".into());
    let mut update_res = db
        .query(
            "UPDATE type::record($tid) SET
            financial_status = IF $fin_status != NONE THEN $fin_status ELSE financial_status END,
            updated_at = time::now();",
        )
        .bind(("tid", treat_rec.clone()))
        .bind(("fin_status", fin_status))
        .await
        .expect("update query failed");

    let updated_vec: Vec<Value> = update_res.take(0).expect("take updated");
    assert!(!updated_vec.is_empty());
    assert_eq!(updated_vec[0]["financial_status"].as_str(), Some("unpaid"), "financial_status should be updated to unpaid");
    println!("Updated treatment financial_status to: {:}s", updated_vec[0]["financial_status"]);

    // 4. Update financial_status back to 'paid'
    let fin_status_paid: Option<String> = Some("paid".into());
    let mut update_res2 = db
        .query(
            "UPDATE type::record($tid) SET
            financial_status = IF $fin_status != NONE THEN $fin_status ELSE financial_status END,
            updated_at = time::now();",
        )
        .bind(("tid", treat_rec.clone()))
        .bind(("fin_status", fin_status_paid))
        .await
        .expect("update query 2 failed");

    let updated_vec2: Vec<Value> = update_res2.take(0).expect("take updated 2");
    assert_eq!(updated_vec2[0]["financial_status"].as_str(), Some("paid"), "financial_status should be updated back to paid");
    println!("Updated treatment financial_status back to: {:}s", updated_vec2[0]["financial_status"]);

    // 5. Clean up test record
    let _ = db.query("DELETE type::record($tid);").bind(("tid", treat_rec)).await;
    println!("Cleaned up test record successfully!");
}
