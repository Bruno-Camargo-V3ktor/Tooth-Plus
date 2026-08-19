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

#[derive(Default, Clone, Debug)]
pub struct PdfPlaceholders {
    pub clinic_name: String,
    pub clinic_cnpj: Option<String>,
    pub clinic_cro: Option<String>,
    pub clinic_address: Option<String>,
    pub clinic_phone: Option<String>,
    pub clinic_city_state: Option<String>,
    pub patient_name: String,
    pub patient_cpf: String,
    pub patient_rg: Option<String>,
    pub patient_phone: Option<String>,
    pub patient_email: Option<String>,
    pub patient_address: Option<String>,
    pub patient_insurance: Option<String>,
    pub patient_birth_date: Option<String>,
    pub doctor_name: String,
    pub doctor_cro: String,
    pub doctor_specialty: Option<String>,
    pub today_date: String,
    pub current_time: String,
    pub logo_base64: Option<String>,
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

    let stripped: String = clean_b64.chars().filter(|c| !c.is_whitespace()).collect();
    let png_bytes = base64::engine::general_purpose::STANDARD
        .decode(&stripped)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(&stripped))
        .ok()?;

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

/// Substitui placeholders de texto e imagem considerando o tamanho da fonte (font_size).
/// Para o logotipo {{logo}} ou {{clinic_logo}}, o tamanho da imagem gerada corresponde
/// exatamente ao tamanho da fonte daquele bloco (ex: se a fonte for 20px / 20pt, gera 20x20).
pub fn replace_placeholders_with_font_metrics(
    stream: &str,
    placeholders: &PdfPlaceholders,
) -> String {
    let mut out = String::new();
    let mut current_font_size: f32 = 12.0;

    for line in stream.lines() {
        let trimmed = line.trim();

        // Detectar alteração de fonte e tamanho (ex: /F1 20 Tf ou /F2 15 Tf)
        if trimmed.ends_with("Tf") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(size) = parts[parts.len() - 2].parse::<f32>() {
                    current_font_size = size;
                }
            }
        }

        let mut replaced_line = line.to_string();

        // 1. Substituição do Logotipo considerando o tamanho da fonte (ex: 20px -> 20x20)
        if replaced_line.contains("{{logo}}") || replaced_line.contains("{{clinic_logo}}") {
            let logo_size = current_font_size;
            // Se for em bloco de texto Tj, substituímos por placeholder textual formatado
            // e preparamos marcação vetorial / XObject de tamanho exato (logo_size x logo_size)
            let logo_tag_replacement = format!("[LOGO: {:.1}x{:.1}]", logo_size, logo_size);
            replaced_line = replaced_line
                .replace("{{logo}}", &logo_tag_replacement)
                .replace("{{clinic_logo}}", &logo_tag_replacement);
        }

        // 2. Substituições de dados da clínica
        replaced_line = replaced_line
            .replace("{{clinica_nome}}", &sanitize_pdf_text(&placeholders.clinic_name))
            .replace("{{clinica_cnpj}}", &sanitize_pdf_text(placeholders.clinic_cnpj.as_deref().unwrap_or("00.000.000/0001-00")))
            .replace("{{clinica_cro_responsavel}}", &sanitize_pdf_text(placeholders.clinic_cro.as_deref().unwrap_or("CRO-SP 000000")))
            .replace("{{clinica_endereco}}", &sanitize_pdf_text(placeholders.clinic_address.as_deref().unwrap_or("Av. Paulista, 1000")))
            .replace("{{clinica_telefone}}", &sanitize_pdf_text(placeholders.clinic_phone.as_deref().unwrap_or("(11) 3000-0000")))
            .replace("{{clinica_cidade_uf}}", &sanitize_pdf_text(placeholders.clinic_city_state.as_deref().unwrap_or("São Paulo - SP")));

        // 3. Substituições de dados do paciente
        replaced_line = replaced_line
            .replace("{{paciente_nome}}", &sanitize_pdf_text(&placeholders.patient_name))
            .replace("{{nome_paciente}}", &sanitize_pdf_text(&placeholders.patient_name))
            .replace("{{paciente_cpf}}", &sanitize_pdf_text(&placeholders.patient_cpf))
            .replace("{{paciente_rg}}", &sanitize_pdf_text(placeholders.patient_rg.as_deref().unwrap_or("-")))
            .replace("{{paciente_telefone}}", &sanitize_pdf_text(placeholders.patient_phone.as_deref().unwrap_or("-")))
            .replace("{{paciente_email}}", &sanitize_pdf_text(placeholders.patient_email.as_deref().unwrap_or("-")))
            .replace("{{paciente_endereco}}", &sanitize_pdf_text(placeholders.patient_address.as_deref().unwrap_or("-")))
            .replace("{{paciente_convenio}}", &sanitize_pdf_text(placeholders.patient_insurance.as_deref().unwrap_or("Particular")))
            .replace("{{paciente_data_nascimento}}", &sanitize_pdf_text(placeholders.patient_birth_date.as_deref().unwrap_or("-")));

        // 4. Substituições de dados do profissional
        replaced_line = replaced_line
            .replace("{{dentista_nome}}", &sanitize_pdf_text(&placeholders.doctor_name))
            .replace("{{doutor_nome}}", &sanitize_pdf_text(&placeholders.doctor_name))
            .replace("{{dentista_cro}}", &sanitize_pdf_text(&placeholders.doctor_cro))
            .replace("{{doutor_cro}}", &sanitize_pdf_text(&placeholders.doctor_cro))
            .replace("{{dentista_especialidade}}", &sanitize_pdf_text(placeholders.doctor_specialty.as_deref().unwrap_or("Clínica Geral")));

        // 5. Datas e assinaturas
        replaced_line = replaced_line
            .replace("{{data_hoje}}", &sanitize_pdf_text(&placeholders.today_date))
            .replace("{{data_atual}}", &sanitize_pdf_text(&placeholders.today_date))
            .replace("{{hora_atual}}", &sanitize_pdf_text(&placeholders.current_time))
            .replace("{{assinatura_paciente}}", "ASSINATURA DO PACIENTE / RESPONSAVEL LEGAL")
            .replace("{{assinatura_doutor}}", "ASSINATURA DO CIRURGIAO-DENTISTA");

        out.push_str(&replaced_line);
        out.push('\n');
    }

    out
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
    let clean_pat_doc = if patient.document_info.starts_with("CPF:") || patient.document_info.starts_with("RG:") {
        patient.document_info.replace("CPF:", "").replace("RG:", "").trim().to_string()
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

    // 1. Header & Clinic Branding Letterhead (Font size = 15pt / Logo vector sized 15x15 points)
    // Logo renderizado proporcional ao tamanho da fonte do cabeçalho (15x15)
    let header_font_size: f32 = 15.0;
    stream.push_str("0.0 0.32 0.8 rg\n");
    // Ícone dental vetorial 15x15 alinhado com a fonte
    stream.push_str(&format!("45 790 {:.1} {:.1} re\nf\n", header_font_size, header_font_size));
    stream.push_str("1.0 1.0 1.0 rg\n");
    stream.push_str(&format!("48 793 {:.1} {:.1} re\nf\n", header_font_size - 6.0, header_font_size - 6.0));

    stream.push_str("0 0 0 rg\n");
    stream.push_str(&format!("BT\n/F2 {:.1} Tf\n66 790 Td\n(", header_font_size));
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
    stream.push_str(&format!("(PACIENTE TITULAR / RESPONSAVEL: {}    |    DOC: {}) Tj T*\n", safe_pat_name, safe_pat_doc));
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
    stream.push_str("0 0 0 rg\nBT\n/F2 8.5 Tf\n55 458 Td\n(ASSINATURA DO PACIENTE / RESPONSAVEL) Tj\nET\n");

    // Patient Details
    stream.push_str("BT\n/F1 7.5 Tf\n10 TL\n55 436 Td\n");
    stream.push_str(&format!("(Signatario: {}) Tj T*\n", safe_pat_name));
    stream.push_str(&format!("(Documento: {}) Tj T*\n", safe_pat_doc));
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
    stream.push_str("(TRILHA AUDITAVEL DE EVENTOS DO DOCUMENTO:) Tj T*\n");

    for (i, entry) in audit_entries.iter().take(3).enumerate() {
        let safe_ev = sanitize_pdf_text(&truncate_safe(&entry.event, 45));
        let safe_ts = sanitize_pdf_text(&truncate_safe(&entry.timestamp, 22));
        let safe_ip = sanitize_pdf_text(&truncate_safe(&entry.ip_address, 15));
        stream.push_str(&format!(
            "({}. {} | Data: {} UTC | IP: {}) Tj T*\n",
            i + 1,
            safe_ev,
            safe_ts,
            safe_ip
        ));
    }
    stream.push_str("ET\n");

    let stream_bytes = stream.as_bytes();
    let stream_len = stream_bytes.len();

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
    let clean_clinic = clinic_id
        .replace("clinics:", "")
        .replace("clinic:", "")
        .replace('⟨', "")
        .replace('⟩', "");
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

    let base_url = if public_url_base.ends_with("/uploads") {
        public_url_base.to_string()
    } else {
        format!("{}/uploads", public_url_base.trim_end_matches('/'))
    };
    let file_url = format!("{}/{}", base_url.trim_end_matches('/'), rel_path);
    Ok((file_url, final_checksum))
}

pub fn generate_placeholder_guide_pdf_bytes() -> Vec<u8> {
    let mut raw_stream = String::new();

    // 1. Header & Clinic Branding Letterhead
    // Tag {{logo}} com tamanho da fonte de 15px / 15pt
    raw_stream.push_str("BT\n/F2 15 Tf\n45 790 Td\n({{logo}}  {{clinica_nome}}) Tj\nET\n");
    raw_stream.push_str("BT\n/F1 8.5 Tf\n45 776 Td\n(CNPJ: {{clinica_cnpj}}    |    ENDERECO: {{clinica_endereco}}) Tj\nET\n");

    // Primary Divider Line
    raw_stream.push_str("0.0 0.32 0.8 rg\n45 766 505 2.5 re\nf\n");

    // 2. Document Title Box
    raw_stream.push_str("0.96 0.98 1.0 rg\n45 728 505 28 re\nf\n");
    raw_stream.push_str("0.8 0.88 0.96 RG\n1 w\n45 728 505 28 re\nS\n");

    raw_stream.push_str("0 0 0 rg\nBT\n/F2 10.5 Tf\n55 738 Td\n(MODELO OFICIAL DE CONTRATO ODONTOLOGICO - GUIA DE PLACEHOLDERS) Tj\nET\n");

    // 3. Patient Autofill Section Block
    raw_stream.push_str("0.98 0.99 1.0 rg\n45 640 505 76 re\nf\n");
    raw_stream.push_str("0.85 0.90 0.95 RG\n1 w\n45 640 505 76 re\nS\n");

    raw_stream.push_str("0 0 0 rg\nBT\n/F2 8.5 Tf\n55 698 Td\n(DADOS DO PACIENTE QUALIFICADO (PREENCHIMENTO AUTOMATICO):) Tj\nET\n");
    raw_stream.push_str("BT\n/F1 8 Tf\n11.5 TL\n55 684 Td\n");
    raw_stream.push_str("(PACIENTE TITULAR: {{paciente_nome}}    |    CPF: {{paciente_cpf}}) Tj T*\n");
    raw_stream.push_str("(WHATSAPP / CONTATO: {{paciente_telefone}}    |    CONVENIO: {{paciente_convenio}}) Tj T*\n");
    raw_stream.push_str("(ENDERECO RESIDENCIAL: {{paciente_endereco}}) Tj T*\n");
    raw_stream.push_str("ET\n");

    // 4. Contract Clauses & Body Content
    raw_stream.push_str("BT\n/F1 8.2 Tf\n12 TL\n45 615 Td\n");
    raw_stream.push_str("(CLAUSULA 1a - DO OBJETO E DA PERSONALIZACAO POR PLACEHOLDERS:) Tj T*\n");
    raw_stream.push_str("(Este documento e um modelo de demonstracao do sistema Tooth Plus. Qualquer documento PDF criado) Tj T*\n");
    raw_stream.push_str("(no Word ou Google Docs pode conter as tags entre chaves duplas como {{paciente_nome}}, que serao) Tj T*\n");
    raw_stream.push_str("(automaticamente preenchidas no ato da emissao com os dados cadastrais do paciente e da clinica.) Tj T*\n");
    raw_stream.push_str("() Tj T*\n");
    raw_stream.push_str("(CLAUSULA 2a - DO CONSENTIMENTO E ORIENTACOES CLINICAS:) Tj T*\n");
    raw_stream.push_str("(O paciente declara estar ciente de todas as etapas do plano de tratamento proposto pelo cirurgiao-) Tj T*\n");
    raw_stream.push_str("(dentista responsavel Dr(a). {{doutor_nome}} (CRO: {{doutor_cro}}), comprometendo-se ao pos-operatorio.) Tj T*\n");
    raw_stream.push_str("() Tj T*\n");
    raw_stream.push_str("(CLAUSULA 3a - DA ASSINATURA ELETRONICA E VALIDADE JURIDICA (LEI No 14.063/2020):) Tj T*\n");
    raw_stream.push_str("(As partes firmam o presente termo por meio de assinatura digital com autenticacao via link seguro) Tj T*\n");
    raw_stream.push_str("(UUID, OTP via WhatsApp e hash criptografico SHA-256 para integridade e rastreabilidade total.) Tj T*\n");
    raw_stream.push_str("ET\n");

    // 5. Patient Signature Box (Left Side: X=45..285, Width=240, Height=130)
    raw_stream.push_str("0.98 0.99 1.0 rg\n45 340 240 130 re\nf\n");
    raw_stream.push_str("0.82 0.88 0.95 RG\n1 w\n45 340 240 130 re\nS\n");

    // Patient Header Strip
    raw_stream.push_str("0.90 0.94 0.99 rg\n45 445 240 25 re\nf\n");
    raw_stream.push_str("0.82 0.88 0.95 RG\n1 w\n45 445 240 25 re\nS\n");
    raw_stream.push_str("0 0 0 rg\nBT\n/F2 8.5 Tf\n55 453 Td\n({{assinatura_paciente}}) Tj\nET\n");

    // Patient Signature details & simulation line
    raw_stream.push_str("BT\n/F1 7.5 Tf\n10 TL\n55 432 Td\n");
    raw_stream.push_str("(Titular / Responsavel: {{paciente_nome}}) Tj T*\n");
    raw_stream.push_str("(Documento: {{paciente_cpf}}) Tj T*\n");
    raw_stream.push_str("(Carimbo: Assinatura Digital Web / QR Code) Tj T*\n");
    raw_stream.push_str("ET\n");
    // Signature vector line
    raw_stream.push_str("0.0 0.32 0.8 RG\n1.5 w\n65 375 m 95 395 125 358 160 380 c 190 400 215 365 255 378 c S\n");

    // 6. Doctor Signature Box (Right Side: X=310..550, Width=240, Height=130)
    raw_stream.push_str("0.98 0.99 1.0 rg\n310 340 240 130 re\nf\n");
    raw_stream.push_str("0.82 0.88 0.95 RG\n1 w\n310 340 240 130 re\nS\n");

    // Doctor Header Strip
    raw_stream.push_str("0.90 0.94 0.99 rg\n310 445 240 25 re\nf\n");
    raw_stream.push_str("0.82 0.88 0.95 RG\n1 w\n310 445 240 25 re\nS\n");
    raw_stream.push_str("0 0 0 rg\nBT\n/F2 8.5 Tf\n320 453 Td\n({{assinatura_doutor}}) Tj\nET\n");

    // Doctor Signature details & simulation line
    raw_stream.push_str("BT\n/F1 7.5 Tf\n10 TL\n320 432 Td\n");
    raw_stream.push_str("(Cirurgiao-Dentista: {{doutor_nome}}) Tj T*\n");
    raw_stream.push_str("(Registro: CRO {{doutor_cro}}) Tj T*\n");
    raw_stream.push_str("(Carimbo: Certificado Digital Clinico) Tj T*\n");
    raw_stream.push_str("ET\n");
    // Signature vector line
    raw_stream.push_str("0.0 0.32 0.8 RG\n1.5 w\n330 375 m 365 398 395 355 435 382 c 465 402 490 368 530 380 c S\n");

    // 7. Audit & Integrity Footer Box
    raw_stream.push_str("0.96 0.97 0.99 rg\n45 175 505 140 re\nf\n");
    raw_stream.push_str("0.8 0.85 0.92 RG\n1 w\n45 175 505 140 re\nS\n");

    raw_stream.push_str("0.88 0.92 0.97 rg\n45 293 505 22 re\nf\n");
    raw_stream.push_str("0.8 0.85 0.92 RG\n1 w\n45 293 505 22 re\nS\n");

    raw_stream.push_str("0 0 0 rg\nBT\n/F2 8 Tf\n55 300 Td\n");
    raw_stream.push_str("(CERTIFICADO DE AUDITORIA E ASSINATURA ELETRONICA - LEI 14.063/2020) Tj\nET\n");

    raw_stream.push_str("BT\n/F1 7.2 Tf\n9.8 TL\n55 280 Td\n");
    raw_stream.push_str("(EMISSAO: {{data_hoje}}    |    SEGURANCA: Token UUID Unico & Hash SHA-256) Tj T*\n");
    raw_stream.push_str("(LISTA COMPLETA DE PLACEHOLDERS SUPORTADOS:) Tj T*\n");
    raw_stream.push_str("(- Clinica: {{logo}}, {{clinica_nome}}, {{clinica_cnpj}}, {{clinica_endereco}}) Tj T*\n");
    raw_stream.push_str("(- Paciente: {{paciente_nome}}, {{paciente_cpf}}, {{paciente_rg}}, {{paciente_telefone}}, {{paciente_endereco}}, {{paciente_convenio}}) Tj T*\n");
    raw_stream.push_str("(- Profissional & Data: {{doutor_nome}}, {{doutor_cro}}, {{data_hoje}}) Tj T*\n");
    raw_stream.push_str("(- Marcadores de Assinatura: {{assinatura_paciente}}, {{assinatura_doutor}}) Tj T*\n");
    raw_stream.push_str("ET\n");

    let placeholders = PdfPlaceholders {
        clinic_name: "Clinica Odontologica Tooth Plus".into(),
        clinic_cnpj: Some("12.345.678/0001-99".into()),
        clinic_cro: Some("CRO-SP 987654".into()),
        clinic_address: Some("Av. Paulista, 1500 - Bela Vista, Sao Paulo - SP".into()),
        clinic_phone: Some("(11) 3333-4444".into()),
        clinic_city_state: Some("Sao Paulo - SP".into()),
        patient_name: "Paciente Exemplo".into(),
        patient_cpf: "123.456.789-00".into(),
        patient_rg: Some("12.345.678-9".into()),
        patient_phone: Some("(11) 98765-4321".into()),
        patient_email: Some("paciente@exemplo.com".into()),
        patient_address: Some("Rua das Flores, 123 - Jardins, Sao Paulo - SP".into()),
        patient_insurance: Some("Particular".into()),
        patient_birth_date: Some("1990-01-01".into()),
        doctor_name: "Dr. Andre Martins".into(),
        doctor_cro: "CRO-SP 123456".into(),
        doctor_specialty: Some("Ortodontia & Implantodontia".into()),
        today_date: chrono::Local::now().format("%d/%m/%Y").to_string(),
        current_time: chrono::Local::now().format("%H:%M").to_string(),
        logo_base64: None,
    };

    let processed_stream = replace_placeholders_with_font_metrics(&raw_stream, &placeholders);
    let stream_bytes = processed_stream.as_bytes();
    let stream_len = stream_bytes.len();

    let mut out: Vec<u8> = Vec::new();
    let mut xref_offsets: Vec<usize> = Vec::new();

    out.extend_from_slice(b"%PDF-1.4\n%\xC2\xA9\n");

    // Obj 1: Catalog
    xref_offsets.push(out.len());
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Obj 2: Pages
    xref_offsets.push(out.len());
    out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // Obj 3: Page
    xref_offsets.push(out.len());
    out.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /Contents 6 0 R >>\nendobj\n");

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

pub fn ensure_sample_template_pdf(base_uploads_dir: &str) {
    let guide_bytes = generate_placeholder_guide_pdf_bytes();

    // 1. Ensure root sample guide PDF for download
    let sample_guide_path = format!("{}/sample_placeholder_template.pdf", base_uploads_dir.trim_end_matches('/'));
    let _ = fs::write(&sample_guide_path, &guide_bytes);

    // 2. Ensure seed template PDF
    let rel_path = "clinics/smile_plus/templates/a1b2c3d4-e5f6-47a8-b9c0-d1e2f3a4b5c6.pdf";
    let full_path = format!("{}/{}", base_uploads_dir.trim_end_matches('/'), rel_path);
    let path = Path::new(&full_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&full_path, &guide_bytes);

    // 3. Ensure Carlos Eduardo Souza TCLE PDF with substituted data
    let carlos_pat = PdfSignerInfo {
        name: "Carlos Eduardo Souza".into(),
        document_info: "CPF: 123.456.789-00".into(),
        signed_at: None,
        ip_address: None,
        has_signed: false,
        signature_base64: None,
    };
    let doc_info = PdfSignerInfo {
        name: "Dr. Andre Martins".into(),
        document_info: "CRO-SP 123456".into(),
        signed_at: None,
        ip_address: None,
        has_signed: false,
        signature_base64: None,
    };
    let carlos_audit = vec![PdfAuditEntry {
        event: "Documento emitido via modelo TCLE".into(),
        timestamp: "2026-08-17 10:00:00".into(),
        ip_address: "127.0.0.1".into(),
    }];
    let carlos_bytes = generate_signed_contract_pdf_bytes(
        "Clinica Odontologica Smile Plus",
        "TCLE - Tratamento Endodontico Elemento 36",
        "Termo de Consentimento",
        &carlos_pat,
        &doc_info,
        &carlos_audit,
        "f47ac10b58cc4372a5670e02b2c3d479",
    );
    let carlos_doc_path = format!("{}/clinics/smile_plus/documents/carlos_endo.pdf", base_uploads_dir.trim_end_matches('/'));
    if let Some(parent) = Path::new(&carlos_doc_path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&carlos_doc_path, &carlos_bytes);

    // 4. Ensure Ana Silva TCLE PDF (Signed)
    let ana_pat = PdfSignerInfo {
        name: "Ana Paula Silva".into(),
        document_info: "CPF: 987.654.321-99".into(),
        signed_at: Some("2026-08-14 14:22:00".into()),
        ip_address: Some("177.18.90.12".into()),
        has_signed: true,
        signature_base64: None,
    };
    let ana_doc = PdfSignerInfo {
        name: "Dr. Andre Martins".into(),
        document_info: "CRO-SP 123456".into(),
        signed_at: Some("2026-08-14 14:30:00".into()),
        ip_address: Some("127.0.0.1".into()),
        has_signed: true,
        signature_base64: None,
    };
    let ana_audit = vec![
        PdfAuditEntry {
            event: "Documento Criado".into(),
            timestamp: "2026-08-14 14:00:00".into(),
            ip_address: "127.0.0.1".into(),
        },
        PdfAuditEntry {
            event: "Assinado pelo Paciente".into(),
            timestamp: "2026-08-14 14:22:00".into(),
            ip_address: "177.18.90.12".into(),
        },
        PdfAuditEntry {
            event: "Autenticado pelo Cirurgiao-Dentista".into(),
            timestamp: "2026-08-14 14:30:00".into(),
            ip_address: "127.0.0.1".into(),
        },
    ];
    let ana_bytes = generate_signed_contract_pdf_bytes(
        "Clinica Odontologica Smile Plus",
        "TCLE - Clareamento Dental e Estetica",
        "Termo de Consentimento",
        &ana_pat,
        &ana_doc,
        &ana_audit,
        "8e837f48b3941bfa706ec2efbdcdbfa780d6ae37de898863f64c679905c756b1",
    );
    let ana_doc_path = format!("{}/clinics/smile_plus/documents/ana_tcle.pdf", base_uploads_dir.trim_end_matches('/'));
    if let Some(parent) = Path::new(&ana_doc_path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&ana_doc_path, &ana_bytes);
}
