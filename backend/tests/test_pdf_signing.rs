use backend::documents_pdf::{
    ensure_sample_template_pdf, generate_signed_contract_pdf_bytes, save_signed_contract_pdf,
    PdfAuditEntry, PdfSignerInfo,
};

#[test]
fn test_pdf_generation_and_tag_rendering() {
    ensure_sample_template_pdf("uploads");
    ensure_sample_template_pdf("../uploads");
    // Generate a test 100x50 transparent PNG with a red/blue stroke (simulating canvas signature)
    let mut img_buf = image::RgbaImage::new(100, 50);
    for x in 10..90 {
        img_buf.put_pixel(x, 25, image::Rgba([0, 82, 204, 255]));
        img_buf.put_pixel(x, 26, image::Rgba([0, 82, 204, 255]));
    }
    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    img_buf
        .write_to(&mut cursor, image::ImageFormat::Png)
        .expect("PNG encoding failed");

    let b64_sig = format!(
        "data:image/png;base64,{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes)
    );

    let pat_info = PdfSignerInfo {
        name: "Carlos Eduardo Souza".into(),
        document_info: "CPF: 123.456.789-00".into(),
        signed_at: Some("2026-08-18 15:30:00".into()),
        ip_address: Some("192.168.1.100".into()),
        has_signed: true,
        signature_base64: Some(b64_sig.clone()),
    };

    let doc_info = PdfSignerInfo {
        name: "Dr. Andre Martins".into(),
        document_info: "CRO-SP 123456".into(),
        signed_at: Some("2026-08-18 16:00:00".into()),
        ip_address: Some("192.168.1.1".into()),
        has_signed: true,
        signature_base64: Some(b64_sig.clone()),
    };

    let audit_entries = vec![PdfAuditEntry {
        event: "Documento assinado pelo paciente".into(),
        timestamp: "2026-08-18 15:30:00".into(),
        ip_address: "192.168.1.100".into(),
    }];

    let pdf_bytes = generate_signed_contract_pdf_bytes(
        "Clinica Odontologica Smile Plus",
        "Contrato de Prestacao de Servicos Ortodonticos",
        "Ortodontia",
        &pat_info,
        &doc_info,
        &audit_entries,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );

    assert!(!pdf_bytes.is_empty());
    assert!(pdf_bytes.starts_with(b"%PDF-1.4"));

    let out_path = "/tmp/test_generated_contract.pdf";
    std::fs::write(out_path, &pdf_bytes).expect("Failed to write test PDF");
    println!("Test PDF written successfully to {}", out_path);
}

#[test]
fn test_placeholder_font_size_and_logo_scaling() {
    use backend::documents_pdf::{replace_placeholders_with_font_metrics, PdfPlaceholders};

    let stream = "BT\n/F2 20 Tf\n45 790 Td\n({{logo}}  {{clinica_nome}}) Tj\nET\nBT\n/F1 12 Tf\n45 700 Td\n({{paciente_nome}} - CPF: {{paciente_cpf}}) Tj\nET\n";

    let placeholders = PdfPlaceholders {
        clinic_name: "Clinica Odontologica Tooth Plus".into(),
        clinic_cnpj: Some("12.345.678/0001-99".into()),
        clinic_cro: Some("CRO-SP 987654".into()),
        clinic_address: Some("Av. Paulista, 1500".into()),
        clinic_phone: Some("(11) 3333-4444".into()),
        clinic_city_state: Some("Sao Paulo - SP".into()),
        patient_name: "Mariana Silva (Menor)".into(),
        patient_cpf: "123.456.789-00".into(),
        patient_rg: Some("12.345.678-9".into()),
        patient_phone: Some("(11) 98765-4321".into()),
        patient_email: Some("mariana@email.com".into()),
        patient_address: Some("Rua das Flores, 123".into()),
        patient_insurance: Some("Bradesco Dental".into()),
        patient_birth_date: Some("2015-05-10".into()),
        doctor_name: "Dr. Andre Martins".into(),
        doctor_cro: "CRO-SP 123456".into(),
        doctor_specialty: Some("Odontopediatria".into()),
        today_date: "19/08/2026".into(),
        current_time: "14:30".into(),
        logo_base64: None,
    };

    let processed = replace_placeholders_with_font_metrics(stream, &placeholders);
    println!("Processed stream:\n{}", processed);

    // Verificamos que o placeholder {{logo}} sob a fonte de 20pt foi substituído pelo marcador proporcional 20.0x20.0
    assert!(processed.contains("[LOGO: 20.0x20.0]"));
    // Verificamos que {{clinica_nome}} foi substituído
    assert!(processed.contains("Clinica Odontologica Tooth Plus"));
    // Verificamos que {{paciente_nome}} foi substituído (com escape PDF correto)
    assert!(processed.contains(r"Mariana Silva \(Menor\)"));
    assert!(processed.contains("CPF: 123.456.789-00"));
}
