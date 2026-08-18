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
