use std::fs;
use std::path::Path;
use uuid::Uuid;
use crate::security::crypto::calculate_sha256_checksum;

pub struct PdfSignerInfo {
    pub name: String,
    pub document_info: String,
    pub signed_at: Option<String>,
    pub ip_address: Option<String>,
    pub has_signed: bool,
    pub signature_base64: Option<String>,
}

pub struct PdfAuditEntry {
    pub event: String,
    pub timestamp: String,
    pub ip_address: String,
}

struct DecodedImage {
    width: u32,
    height: u32,
    rgb_data: Vec<u8>,
}

fn try_decode_png_base64(b64_str: &str) -> Option<DecodedImage> {
    use base64::Engine;
    let clean_b64 = if let Some(idx) = b64_str.find(',') {
        &b64_str[idx + 1..]
    } else {
        b64_str
    };

    let png_bytes = base64::engine::general_purpose::STANDARD.decode(clean_b64.trim()).ok()?;
    let img = image::load_from_memory(&png_bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
    for pixel in rgba.pixels() {
        let alpha = pixel[3] as f32 / 255.0;
        let r = ((pixel[0] as f32 * alpha) + (255.0 * (1.0 - alpha))) as u8;
        let g = ((pixel[1] as f32 * alpha) + (255.0 * (1.0 - alpha))) as u8;
        let b = ((pixel[2] as f32 * alpha) + (255.0 * (1.0 - alpha))) as u8;
        rgb_data.push(r);
        rgb_data.push(g);
        rgb_data.push(b);
    }

    Some(DecodedImage {
        width,
        height,
        rgb_data,
    })
}

fn sanitize_pdf_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('\r', "")
        .replace('\n', " ")
}

fn truncate_safe(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

pub fn generate_signed_contract_pdf_bytes(
    clinic_name: &str,
    doc_title: &str,
    doc_type: &str,
    patient: &PdfSignerInfo,
    doctor: &PdfSignerInfo,
    audit_entries: &[PdfAuditEntry],
    doc_checksum: &str,
) -> Vec<u8> {
    let safe_clinic = sanitize_pdf_text(clinic_name);
    let safe_title = sanitize_pdf_text(doc_title);
    let safe_type = sanitize_pdf_text(doc_type);

    let safe_pat_name = sanitize_pdf_text(&truncate_safe(&patient.name, 34));
    let clean_pat_doc = if patient.document_info.starts_with("CPF:") {
        patient.document_info.replace("CPF:", "").trim().to_string()
    } else {
        patient.document_info.trim().to_string()
    };
    let safe_pat_doc = sanitize_pdf_text(&clean_pat_doc);

    let safe_doc_name = sanitize_pdf_text(&truncate_safe(&doctor.name, 34));
    let clean_doc_reg = if doctor.document_info.starts_with("Registro:") || doctor.document_info.starts_with("CRO:") {
        doctor.document_info.trim().to_string()
    } else {
        doctor.document_info.trim().to_string()
    };
    let safe_doc_reg = sanitize_pdf_text(&clean_doc_reg);

    let pat_time = patient
        .signed_at
        .as_deref()
        .map(|t| sanitize_pdf_text(&truncate_safe(t, 25)))
        .unwrap_or_else(|| "Pendente".into());
    let pat_ip = patient
        .ip_address
        .as_deref()
        .map(sanitize_pdf_text)
        .unwrap_or_else(|| "N/A".into());

    let doc_time = doctor
        .signed_at
        .as_deref()
        .map(|t| sanitize_pdf_text(&truncate_safe(t, 25)))
        .unwrap_or_else(|| "Pendente".into());
    let doc_ip = doctor
        .ip_address
        .as_deref()
        .map(sanitize_pdf_text)
        .unwrap_or_else(|| "N/A".into());

    let pat_img = patient
        .signature_base64
        .as_deref()
        .and_then(try_decode_png_base64);
    let doc_img = doctor
        .signature_base64
        .as_deref()
        .and_then(try_decode_png_base64);

    let mut stream = String::new();

    // 1. Header & Clinic Branding Letterhead
    stream.push_str("BT\n/F2 15 Tf\n45 790 Td\n(");
    stream.push_str(&safe_clinic.to_uppercase());
    stream.push_str(") Tj\nET\n");

    stream.push_str("BT\n/F1 9 Tf\n45 776 Td\n(PRONTUARIO ELETRONICO ODONTOLOGICO & INSTRUMENTO CONTRATUAL) Tj\nET\n");

    // Primary Divider Line
    stream.push_str("0.0 0.32 0.8 rg\n45 766 505 2.5 re\nf\n");

    // 2. Document Title & Category Bar
    stream.push_str("0.96 0.98 1.0 rg\n45 728 505 28 re\nf\n");
    stream.push_str("0.8 0.88 0.96 RG\n1 w\n45 728 505 28 re\nS\n");

    stream.push_str("0 0 0 rg\nBT\n/F2 10.5 Tf\n55 738 Td\n(");
    stream.push_str(&safe_title.to_uppercase());
    stream.push_str(" [");
    stream.push_str(&safe_type.to_uppercase());
    stream.push_str("]) Tj\nET\n");

    // 3. Contract Clauses & Body Content
    stream.push_str("BT\n/F1 8.5 Tf\n12.5 TL\n45 708 Td\n");
    stream.push_str(&format!("(PACIENTE TITULAR: {}    |    CPF: {}) Tj T*\n", safe_pat_name, safe_pat_doc));
    stream.push_str("() Tj T*\n");
    stream.push_str("(CLAUSULA 1a - DO OBJETO E PROCEDIMENTOS CLINICOS:) Tj T*\n");
    stream.push_str("(O presente instrumento formaliza o consentimento livre e esclarecido do paciente quanto aos procedimentos) Tj T*\n");
    stream.push_str("(odontologicos indicados no plano de tratamento, incluindo tecnicas clinicas, riscos e pos-operatorio.) Tj T*\n");
    stream.push_str("() Tj T*\n");
    stream.push_str("(CLAUSULA 2a - DAS DECLARACOES, ANAMNESE E OBRIGACOES:) Tj T*\n");
    stream.push_str("(O paciente atesta a total veracidade das informacoes de saude prestadas na anamnese e compromete-se a) Tj T*\n");
    stream.push_str("(seguir rigorosamente as orientacoes do corpo clinico para a garantia do sucesso terapeutico.) Tj T*\n");
    stream.push_str("() Tj T*\n");
    stream.push_str("(CLAUSULA 3a - DA VALIDADE JURIDICA E AUDITORIA ELETRONICA (LEI 14.063/2020):) Tj T*\n");
    stream.push_str("(As partes reconhecem a plena eficacia, autenticidade e validade probatoria das assinaturas eletronicas) Tj T*\n");
    stream.push_str("(apostas neste termo, protegidas por hash criptografico SHA-256 e registro auditavel de integridade.) Tj T*\n");
    stream.push_str("ET\n");

    // 4. Patient Signature Box (Left Side: X=45..285, Width=240, Height=155)
    stream.push_str("0.98 0.99 1.0 rg\n45 320 240 155 re\nf\n");
    stream.push_str("0.82 0.88 0.95 RG\n1 w\n45 320 240 155 re\nS\n");

    // Patient Header Strip
    stream.push_str("0.90 0.94 0.99 rg\n45 450 240 25 re\nf\n");
    stream.push_str("0.82 0.88 0.95 RG\n1 w\n45 450 240 25 re\nS\n");
    stream.push_str("0 0 0 rg\nBT\n/F2 8.5 Tf\n55 458 Td\n(ASSINATURA DO PACIENTE / TITULAR) Tj\nET\n");

    // Patient Details
    stream.push_str("BT\n/F1 7.5 Tf\n10 TL\n55 436 Td\n");
    stream.push_str(&format!("(Nome: {}) Tj T*\n", safe_pat_name));
    stream.push_str(&format!("(CPF: {}) Tj T*\n", safe_pat_doc));
    stream.push_str(&format!("(Data/Hora: {} UTC) Tj T*\n", pat_time));
    stream.push_str(&format!("(IP: {}) Tj T*\n", pat_ip));
    if patient.has_signed {
        stream.push_str("(Status: [ASSINADO DIGITALMENTE]) Tj T*\n");
    } else {
        stream.push_str("(Status: [AGUARDANDO ASSINATURA]) Tj T*\n");
    }
    stream.push_str("ET\n");

    // Render patient signature image or vector fallback inside the box
    if pat_img.is_some() {
        stream.push_str("q\n220 0 0 52 55 328 cm\n/SigPatient Do\nQ\n");
    } else if patient.has_signed {
        stream.push_str("0.0 0.32 0.8 RG\n1.8 w\n65 355 m 95 385 125 338 160 368 c 190 390 215 348 255 362 c S\n");
    }

    // 5. Doctor Signature Box (Right Side: X=310..550, Width=240, Height=155)
    stream.push_str("0.98 0.99 1.0 rg\n310 320 240 155 re\nf\n");
    stream.push_str("0.82 0.88 0.95 RG\n1 w\n310 320 240 155 re\nS\n");

    // Doctor Header Strip
    stream.push_str("0.90 0.94 0.99 rg\n310 450 240 25 re\nf\n");
    stream.push_str("0.82 0.88 0.95 RG\n1 w\n310 450 240 25 re\nS\n");
    stream.push_str("0 0 0 rg\nBT\n/F2 8.5 Tf\n320 458 Td\n(ASSINATURA DO CIRURGIAO-DENTISTA) Tj\nET\n");

    // Doctor Details
    stream.push_str("BT\n/F1 7.5 Tf\n10 TL\n320 436 Td\n");
    stream.push_str(&format!("(Profissional: {}) Tj T*\n", safe_doc_name));
    stream.push_str(&format!("(Registro: {}) Tj T*\n", safe_doc_reg));
    stream.push_str(&format!("(Data/Hora: {} UTC) Tj T*\n", doc_time));
    stream.push_str(&format!("(IP: {}) Tj T*\n", doc_ip));
    if doctor.has_signed {
        stream.push_str("(Status: [ASSINADO DIGITALMENTE]) Tj T*\n");
    } else {
        stream.push_str("(Status: [AGUARDANDO ASSINATURA]) Tj T*\n");
    }
    stream.push_str("ET\n");

    // Render doctor signature image or vector fallback inside the box
    if doc_img.is_some() {
        stream.push_str("q\n220 0 0 52 320 328 cm\n/SigDoctor Do\nQ\n");
    } else if doctor.has_signed {
        stream.push_str("0.0 0.32 0.8 RG\n1.8 w\n325 358 m 365 388 395 342 435 372 c 465 392 490 352 530 366 c S\n");
    }

    // 6. Audit & Legal Compliance Stamp Footer (X=45..550, Width=505, Height=145)
    stream.push_str("0.96 0.97 0.99 rg\n45 155 505 145 re\nf\n");
    stream.push_str("0.8 0.85 0.92 RG\n1 w\n45 155 505 145 re\nS\n");

    // Audit Header Strip
    stream.push_str("0.88 0.92 0.97 rg\n45 278 505 22 re\nf\n");
    stream.push_str("0.8 0.85 0.92 RG\n1 w\n45 278 505 22 re\nS\n");

    stream.push_str("0 0 0 rg\nBT\n/F2 8 Tf\n55 285 Td\n");
    stream.push_str("(CERTIFICADO DE CONFORMIDADE E AUDITORIA DIGITAL - LEI FEDERAL No 14.063/2020) Tj\nET\n");

    stream.push_str("BT\n/F1 7.2 Tf\n9.8 TL\n55 264 Td\n");
    stream.push_str("(MODALIDADE: Assinatura Eletronica Avancada com Integridade e Registro de Nao-Repudio) Tj T*\n");
    stream.push_str(&format!("(HASH CRIPTOGRAFICO SHA-256 DO ARQUIVO: {}) Tj T*\n", doc_checksum));
    stream.push_str("(TRILHA DE AUDITORIA E EVENTOS REGISTRADOS:) Tj T*\n");

    for ev in audit_entries.iter().take(4) {
        let safe_action = sanitize_pdf_text(&truncate_safe(&ev.event, 40));
        let safe_ts = sanitize_pdf_text(&truncate_safe(&ev.timestamp, 24));
        let safe_ip = sanitize_pdf_text(&truncate_safe(&ev.ip_address, 18));
        stream.push_str(&format!("(- Evento: {} | Data/Hora: {} UTC | IP: {}) Tj T*\n", safe_action, safe_ts, safe_ip));
    }
    stream.push_str("ET\n");

    let stream_bytes = stream.as_bytes();
    let stream_len = stream_bytes.len();

    // Build PDF objects
    let mut out: Vec<u8> = Vec::new();
    let mut xref_offsets: Vec<usize> = Vec::new();

    out.extend_from_slice(b"%PDF-1.4\n%\xC2\xA9\n");

    // Obj 1: Catalog
    xref_offsets.push(out.len());
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Obj 2: Pages
    xref_offsets.push(out.len());
    out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // Obj 3: Page (references fonts and XObjects)
    xref_offsets.push(out.len());
    let mut xobjects_dict = String::new();
    let mut next_obj = 7;
    let mut pat_obj_id = 0;
    let mut doc_obj_id = 0;

    if pat_img.is_some() {
        pat_obj_id = next_obj;
        next_obj += 1;
        xobjects_dict.push_str(&format!(" /SigPatient {} 0 R", pat_obj_id));
    }
    if doc_img.is_some() {
        doc_obj_id = next_obj;
        next_obj += 1;
        xobjects_dict.push_str(&format!(" /SigDoctor {} 0 R", doc_obj_id));
    }

    let page_obj = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> /XObject << {} >> >> /Contents 6 0 R >>\nendobj\n",
        xobjects_dict
    );
    out.extend_from_slice(page_obj.as_bytes());

    // Obj 4: Font F1 (Regular)
    xref_offsets.push(out.len());
    out.extend_from_slice(b"4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");

    // Obj 5: Font F2 (Bold)
    xref_offsets.push(out.len());
    out.extend_from_slice(b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj\n");

    // Obj 6: Contents stream
    xref_offsets.push(out.len());
    let content_header = format!("6 0 obj\n<< /Length {} >>\nstream\n", stream_len);
    out.extend_from_slice(content_header.as_bytes());
    out.extend_from_slice(stream_bytes);
    out.extend_from_slice(b"endstream\nendobj\n");

    // Optional Obj 7 / 8: Patient Image XObject
    if let Some(ref img) = pat_img {
        xref_offsets.push(out.len());
        let img_header = format!(
            "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n",
            pat_obj_id, img.width, img.height, img.rgb_data.len()
        );
        out.extend_from_slice(img_header.as_bytes());
        out.extend_from_slice(&img.rgb_data);
        out.extend_from_slice(b"\nendstream\nendobj\n");
    }

    // Optional Obj: Doctor Image XObject
    if let Some(ref img) = doc_img {
        xref_offsets.push(out.len());
        let img_header = format!(
            "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n",
            doc_obj_id, img.width, img.height, img.rgb_data.len()
        );
        out.extend_from_slice(img_header.as_bytes());
        out.extend_from_slice(&img.rgb_data);
        out.extend_from_slice(b"\nendstream\nendobj\n");
    }

    let start_xref = out.len();
    let total_objs = xref_offsets.len() + 1;

    let mut xref = format!("xref\n0 {}\n0000000000 65535 f \n", total_objs);
    for offset in xref_offsets {
        xref.push_str(&format!("{:010} 00000 n \n", offset));
    }
    xref.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", total_objs, start_xref));

    out.extend_from_slice(xref.as_bytes());
    out
}

pub fn save_signed_contract_pdf(
    base_uploads_dir: &str,
    public_url_base: &str,
    clinic_id: &str,
    doc_title: &str,
    doc_type: &str,
    clinic_name: &str,
    patient: &PdfSignerInfo,
    doctor: &PdfSignerInfo,
    audit_entries: &[PdfAuditEntry],
) -> Result<(String, String), String> {
    let clean_clinic = clinic_id.replace("clinic:", "");
    let doc_uuid = Uuid::new_v4().to_string();

    let temp_checksum = calculate_sha256_checksum(doc_title.as_bytes());

    let bytes = generate_signed_contract_pdf_bytes(
        clinic_name,
        doc_title,
        doc_type,
        patient,
        doctor,
        audit_entries,
        &temp_checksum,
    );

    let final_checksum = calculate_sha256_checksum(&bytes);

    // Re-generate with final checksum baked in
    let final_bytes = generate_signed_contract_pdf_bytes(
        clinic_name,
        doc_title,
        doc_type,
        patient,
        doctor,
        audit_entries,
        &final_checksum,
    );

    let rel_path = format!("clinics/{}/documents/{}.pdf", clean_clinic, doc_uuid);
    let full_path = format!("{}/{}", base_uploads_dir.trim_end_matches('/'), rel_path);

    let path_obj = Path::new(&full_path);
    if let Some(parent) = path_obj.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(&full_path, &final_bytes).map_err(|e| format!("Failed to save PDF: {}", e))?;

    let file_url = format!("{}/{}", public_url_base.trim_end_matches('/'), rel_path);
    Ok((file_url, final_checksum))
}
