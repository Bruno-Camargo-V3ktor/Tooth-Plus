//! # Aba de Anamnese Odontológica (Frontend)
//!
//! Exibe e permite preencher/editar a Ficha Oficial de Anamnese do paciente
//! com layout de tabela Pergunta | Resposta e suporte completo à assinatura digital
//! com invalidação automática de assinaturas anteriores em caso de edição.

use crate::api::{request_anamnesis_signature, save_patient_anamnesis, sync_patient_anamnesis};
use crate::components::icons::{
    IconAlertTriangle, IconCheckCircle, IconCopy, IconExternalLink, IconEye, IconHeartPulse,
    IconQrCode, IconRefresh, IconSignature, IconWhatsApp,
};
use crate::utils::build_signing_url;
use dioxus::prelude::*;
use shared::anamnesis::{AnamnesisResponseItem, SyncAnamnesisRequest};
use shared::patients::{PatientAnamnesis, SaveAnamnesisRequest};

fn generate_qr_svg(url: &str) -> String {
    use qrcode::render::svg;
    use qrcode::QrCode;
    match QrCode::new(url.as_bytes()) {
        Ok(code) => code
            .render::<svg::Color>()
            .min_dimensions(180, 180)
            .dark_color(svg::Color("#0f172a"))
            .light_color(svg::Color("#ffffff"))
            .build(),
        Err(_) => String::new(),
    }
}

/// Componente da aba de Anamnese Médica e Odontológica do Paciente.
#[component]
pub fn PatientAnamneseTab(
    patient_id: String,
    clinic_id: String,
    token: String,
    anamnesis: Option<PatientAnamnesis>,
    can_write: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let anam = anamnesis.clone().unwrap_or_default();
    let template_type_str = anam.template_type.clone().unwrap_or_else(|| "adult".to_string());
    let is_minor_template = template_type_str == "minor";

    // Se já tiver respostas dinâmicas gravadas, usa elas; caso contrário, inicializa com o padrão oficial
    let initial_responses: Vec<AnamnesisResponseItem> = if !anam.custom_responses.is_empty() {
        anam.custom_responses.clone()
    } else if is_minor_template {
        vec![
            AnamnesisResponseItem {
                question_id: "ped_chief_complaint".into(),
                category: "Queixa Principal".into(),
                question_text: "Queixa principal dos pais / responsáveis?".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: anam.chief_complaint.clone(),
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_habits".into(),
                category: "Hábitos Infantis".into(),
                question_text: "Hábitos infantis (Chupeta, sucção de dedo, mamadeira noturna, roer unhas)?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_birth_nursing".into(),
                category: "Histórico Pediátrico".into(),
                question_text: "Histórico de parto e amamentação (Parto normal/cesárea, amamentação)?".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_allergies".into(),
                category: "Alergias Pediátricas".into(),
                question_text: "Possui alguma alergia a medicamentos, alimentos ou substâncias?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(!anam.allergies.is_empty()),
                answer_text: None,
                notes: if anam.allergies.is_empty() { None } else { Some(anam.allergies.join(", ")) },
            },
            AnamnesisResponseItem {
                question_id: "ped_medication".into(),
                category: "Medicamentos".into(),
                question_text: "Faz uso contínuo de algum medicamento, vitamina ou xarope?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.continuous_medications.is_some()),
                answer_text: None,
                notes: anam.continuous_medications.clone(),
            },
            AnamnesisResponseItem {
                question_id: "ped_complication".into(),
                category: "Histórico Clínico".into(),
                question_text: "Já teve alguma complicação em anestesia ou internação prévia?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_respiratory".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Possui problemas respiratórios (Asma, Bronquite, Rinite, respiração bucal)?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_bruxism".into(),
                category: "Hábitos Infantis".into(),
                question_text: "Range os dentes durante o sono (Bruxismo infantil)?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.bruxism),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_trauma".into(),
                category: "Histórico Odontológico".into(),
                question_text: "Sofreu algum trauma dental ou queda recente na boca?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_sweets".into(),
                category: "Hábitos e Dieta".into(),
                question_text: "Frequência de ingestão de doces / açúcar entre as refeições?".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_hygiene".into(),
                category: "Higiene Bucal".into(),
                question_text: "Como é a aceitação da escovação e uso de fio dental em casa?".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "ped_previous_visit".into(),
                category: "Histórico Odontológico".into(),
                question_text: "Já consultou odontopediatra anteriormente?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
        ]
    } else {
        vec![
            AnamnesisResponseItem {
                question_id: "prof_occupation".into(),
                category: "Dados Gerais".into(),
                question_text: "Profissão".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "chief_complaint".into(),
                category: "Queixa Principal".into(),
                question_text: "Queixa principal?".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: anam.chief_complaint.clone(),
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "brushing_frequency".into(),
                category: "Higiene Bucal".into(),
                question_text: "Quantas vezes por dia escova os dentes?".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "under_medical_treatment".into(),
                category: "Saúde Geral".into(),
                question_text: "Está em tratamento médico?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "using_medication".into(),
                category: "Medicamentos".into(),
                question_text: "Está usando medicação?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.continuous_medications.is_some()),
                answer_text: None,
                notes: anam.continuous_medications.clone(),
            },
            AnamnesisResponseItem {
                question_id: "has_allergies".into(),
                category: "Alergias".into(),
                question_text: "Possui alguma alergia? (Como penicilinas, AAS ou outra)".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(!anam.allergies.is_empty()),
                answer_text: None,
                notes: if anam.allergies.is_empty() { None } else { Some(anam.allergies.join(", ")) },
            },
            AnamnesisResponseItem {
                question_id: "diagnosed_hemorrhage".into(),
                category: "Histórico Clínico".into(),
                question_text: "Já teve hemorragia diagnosticada?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.has_bleeding_disorder),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "cardiovascular_disorder".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Possui alguma alteração cardiovascular?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "hypertension".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Tem pressão alta?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "diabetes".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Possui diabetes?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "transmissible_disease".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Possui alguma doença transmissível? (HIV, Hepatite, outra)".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "is_pregnant".into(),
                category: "Saúde Geral".into(),
                question_text: "Está grávida / lactante?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.is_pregnant),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "smoker".into(),
                category: "Hábitos".into(),
                question_text: "Fumante?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.smoker),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "bruxism".into(),
                category: "Hábitos".into(),
                question_text: "Range os dentes (Bruxismo)?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.bruxism),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "alcohol_consumption".into(),
                category: "Hábitos".into(),
                question_text: "Ingere bebidas alcoólicas?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "oral_surgery_history".into(),
                category: "Histórico Odontológico".into(),
                question_text: "Já se submeteu à Cirurgia Oral (exodontia, freio labial, etc.)?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            },
        ]
    };

    let mut responses_signal = use_signal(|| initial_responses);
    let mut is_saving = use_signal(|| false);
    let mut is_sync_modal_open = use_signal(|| false);
    let mut is_syncing = use_signal(|| false);
    let mut is_requesting_sign = use_signal(|| false);

    // Modal de QR Code para assinatura da anamnese
    let mut qr_modal_data = use_signal(|| None::<(String, String, String)>);
    let mut pdf_preview_target = use_signal(|| None::<(String, String)>);

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();
    let t_type_for_save = template_type_str.clone();

    let template_badge = if is_minor_template { "Ficha Oficial: Odontopediatria (Menor)" } else { "Ficha Oficial: Adulto" };
    let current_responses = responses_signal();

    let sign_status = anam.signature_status.as_deref().unwrap_or("not_requested");
    let is_signed = sign_status == "signed";
    let has_token = anam.signing_token.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
    let is_pending = sign_status == "pending" && has_token;

    // Ação para solicitar assinatura
    let handle_request_signature = {
        let t_c = tok.clone();
        let p_c = pat_id.clone();
        let c_c = cid.clone();
        let on_r = reload_patient_details.clone();
        move |_| {
            let t = t_c.clone();
            let p = p_c.clone();
            let c = c_c.clone();
            let on_reload = on_r.clone();
            is_requesting_sign.set(true);
            spawn(async move {
                match request_anamnesis_signature(&t, &p, &c).await {
                    Ok(resp) => {
                        let real_sign_url = build_signing_url(&resp.signing_token);
                        qr_modal_data.set(Some((resp.signing_token, real_sign_url, resp.document_pdf_url)));
                        toast_msg.set(Some("Termo de anamnese gerado para assinatura digital!".into()));
                        on_reload.call(());
                    }
                    Err(e) => {
                        error_toast.set(Some(format!("Erro ao gerar assinatura: {}", e)));
                    }
                }
                is_requesting_sign.set(false);
            });
        }
    };

    let signature_title = if is_signed {
        "Anamnese Assinada Digitalmente"
    } else if is_pending {
        "Aguardando Assinatura do Paciente / Responsável"
    } else {
        "Assinatura Digital da Anamnese"
    };

    let signature_desc = if is_signed {
        if let Some(ref signed_dt) = anam.signed_at {
            format!("Documento assinado eletronicamente em {}", signed_dt.chars().take(16).collect::<String>().replace("T", " às "))
        } else {
            "Documento assinado com certificado e integridade jurídica.".to_string()
        }
    } else if is_pending {
        "Link de assinatura ativo. Envie por WhatsApp ou abra o QR Code para o paciente assinar.".to_string()
    } else {
        "Gere o termo de consentimento da anamnese para coleta de assinatura na clínica ou celular.".to_string()
    };

    rsx! {
        div { class: "anamnesis-view-wrapper", style: "display: flex; flex-direction: column; gap: 20px;",
            // Header com Badges e Ações de Salvar / Sincronizar
            div { style: "display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;",
                div { style: "display: flex; align-items: center; gap: 10px;",
                    span { class: "badge-insurance-plan font-mono font-xs", "{template_badge}" }
                    if !anam.updated_at.is_empty() {
                        span { class: "text-muted font-xs",
                            "Última atualização: {anam.updated_at.chars().take(10).collect::<String>()}"
                        }
                    }
                }
                if can_write {
                    div { style: "display: flex; gap: 10px;",
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            style: "font-size: 13px; padding: 6px 14px; display: inline-flex; align-items: center; gap: 6px;",
                            onclick: move |_| is_sync_modal_open.set(true),
                            IconRefresh { size: 14, color: "currentColor".to_string() }
                            span { " Sincronizar com Modelo da Clínica" }
                        }
                        button {
                            r#type: "button",
                            class: "btn-primary",
                            style: "font-size: 13px; padding: 6px 18px; font-weight: 600;",
                            disabled: is_saving(),
                            onclick: {
                                let t_c = tok.clone();
                                let p_c = pat_id.clone();
                                let c_c = cid.clone();
                                let tt_c = t_type_for_save.clone();
                                let on_r = reload_patient_details.clone();
                                move |_| {
                                    let current_resp = responses_signal();
                                    let mut allergies = Vec::new();
                                    let mut diseases = Vec::new();
                                    let mut continuous_meds = None;
                                    let mut chief_comp = None;
                                    let mut is_preg = false;
                                    let mut has_bleed = false;
                                    let mut smoker = false;
                                    let mut brux = false;

                                    for r in &current_resp {
                                        if (r.category == "Alergias" || r.category == "Alergias Pediátricas") && r.answer_boolean.unwrap_or(false) {
                                            allergies.push(r.notes.clone().unwrap_or_else(|| r.question_text.clone()));
                                        }
                                        if r.category == "Saúde Sistêmica" && r.answer_boolean.unwrap_or(false) {
                                            diseases.push(r.question_text.clone());
                                        }
                                        if r.question_id.contains("preg") {
                                            is_preg = r.answer_boolean.unwrap_or(false);
                                        }
                                        if r.question_id.contains("bleed") || r.question_id.contains("hemorrhage") {
                                            has_bleed = r.answer_boolean.unwrap_or(false);
                                        }
                                        if r.question_id.contains("smoke") {
                                            smoker = r.answer_boolean.unwrap_or(false);
                                        }
                                        if r.question_id.contains("brux") || r.question_id.contains("grind") {
                                            brux = r.answer_boolean.unwrap_or(false);
                                        }
                                        if r.category == "Medicamentos" {
                                            if r.question_type == "yes_no" {
                                                if r.answer_boolean.unwrap_or(false) {
                                                    continuous_meds = r.notes.clone();
                                                }
                                            } else {
                                                continuous_meds = r.answer_text.clone();
                                            }
                                        }
                                        if r.category == "Queixa Principal" {
                                            chief_comp = r.answer_text.clone();
                                        }
                                    }

                                    let req = SaveAnamnesisRequest {
                                        clinic_id: c_c.clone(),
                                        template_type: Some(tt_c.clone()),
                                        custom_responses: Some(current_resp),
                                        allergies,
                                        continuous_medications: continuous_meds,
                                        systemic_diseases: diseases,
                                        is_pregnant: is_preg,
                                        has_bleeding_disorder: has_bleed,
                                        smoker,
                                        bruxism: brux,
                                        chief_complaint: chief_comp,
                                        clinical_notes: None,
                                    };

                                    let tok_clone = t_c.clone();
                                    let pat_clone = p_c.clone();
                                    let reload_c = on_r.clone();
                                    is_saving.set(true);
                                    spawn(async move {
                                        match save_patient_anamnesis(&tok_clone, &pat_clone, req).await {
                                            Ok(_) => {
                                                toast_msg.set(Some("Ficha de anamnese salva com sucesso!".into()));
                                                reload_c.call(());
                                            }
                                            Err(e) => {
                                                error_toast.set(Some(format!("Erro ao salvar anamnese: {}", e)));
                                            }
                                        }
                                        is_saving.set(false);
                                    });
                                }
                            },
                            if is_saving() { "Salvando Ficha..." } else { "Salvar Ficha de Anamnese" }
                        }
                    }
                }
            }

            // Tabela Oficial de Anamnese (Pergunta | Resposta)
            div { class: "table-container", style: "box-shadow: 0 1px 3px rgba(15, 23, 42, 0.05);",
                div { style: "padding: 14px 20px; background: #f8fafc; border-bottom: 1px solid #e2e8f0; display: flex; justify-content: space-between; align-items: center;",
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        IconHeartPulse { size: 20, color: "#0052cc".to_string() }
                        span { style: "font-size: 15px; font-weight: 700; color: #0f172a; letter-spacing: 0.5px;", "FICHA DE ANAMNESE" }
                    }
                    span { class: "text-muted font-xs", "Preenchimento clínico e histórico de saúde" }
                }

                table { class: "modern-table",
                    thead {
                        tr {
                            th { style: "width: 50%; font-size: 13px; text-transform: none;", "Pergunta" }
                            th { style: "width: 50%; font-size: 13px; text-transform: none;", "Resposta" }
                        }
                    }
                    tbody {
                        for (idx, item) in current_responses.iter().enumerate() {
                            {
                                let is_yes_no = item.question_type == "yes_no";
                                let is_yes = item.answer_boolean == Some(true);
                                let is_no = item.answer_boolean == Some(false);
                                let text_val = item.answer_text.clone().unwrap_or_default();
                                let notes_val = item.notes.clone().unwrap_or_default();

                                rsx! {
                                    tr { key: "{item.question_id}",
                                        // Coluna 1: Pergunta com Categoria
                                        td { style: "vertical-align: top; padding: 14px 18px;",
                                            div { style: "display: flex; align-items: flex-start; gap: 8px;",
                                                span { style: "color: #94a3b8; font-weight: 700; font-size: 12px; min-width: 22px; margin-top: 2px;", "#{idx + 1}" }
                                                div {
                                                    p { style: "margin: 0; font-weight: 600; color: #1e293b; font-size: 13.5px; line-height: 1.4;",
                                                        "{item.question_text}"
                                                    }
                                                    span { style: "display: inline-block; margin-top: 4px; font-size: 11px; color: #64748b; background: #f1f5f9; padding: 2px 6px; border-radius: 4px;",
                                                        "{item.category}"
                                                    }
                                                }
                                            }
                                        }

                                        // Coluna 2: Resposta (SIM/NÃO ou Texto)
                                        td { style: "vertical-align: top; padding: 14px 18px;",
                                            if is_yes_no {
                                                div { style: "display: flex; flex-direction: column; gap: 8px;",
                                                    div { style: "display: flex; gap: 10px;",
                                                        // Botão Não (Azul quando ativo)
                                                        button {
                                                            r#type: "button",
                                                            class: if is_no { "btn-primary" } else { "btn-secondary" },
                                                            style: if is_no {
                                                                "padding: 5px 16px; font-size: 12.5px; font-weight: 600; background: #0052cc; border-color: #0052cc;"
                                                            } else {
                                                                "padding: 5px 16px; font-size: 12.5px; color: #64748b;"
                                                            },
                                                            disabled: !can_write,
                                                            onclick: move |_| {
                                                                let mut list = responses_signal();
                                                                if idx < list.len() {
                                                                    list[idx].answer_boolean = Some(false);
                                                                    responses_signal.set(list);
                                                                }
                                                            },
                                                            "Não"
                                                        }
                                                        // Botão Sim (Vermelho quando ativo para destacar alertas clínicos)
                                                        button {
                                                            r#type: "button",
                                                            class: if is_yes { "btn-primary" } else { "btn-secondary" },
                                                            style: if is_yes {
                                                                "padding: 5px 16px; font-size: 12.5px; font-weight: 600; background: #dc2626; border-color: #dc2626;"
                                                            } else {
                                                                "padding: 5px 16px; font-size: 12.5px; color: #64748b;"
                                                            },
                                                            disabled: !can_write,
                                                            onclick: move |_| {
                                                                let mut list = responses_signal();
                                                                if idx < list.len() {
                                                                    list[idx].answer_boolean = Some(true);
                                                                    responses_signal.set(list);
                                                                }
                                                            },
                                                            "Sim"
                                                        }
                                                    }

                                                    // Se marcou SIM, abre campo para detalhamento
                                                    if is_yes {
                                                        div { style: "margin-top: 4px;",
                                                            input {
                                                                class: "form-input",
                                                                style: "font-size: 13px; padding: 6px 10px; background: #fef2f2; border-color: #fecaca;",
                                                                placeholder: "Especifique detalhes (ex: tipo de alergia, medicamento, frequencia)...",
                                                                value: "{notes_val}",
                                                                disabled: !can_write,
                                                                oninput: move |e| {
                                                                    let mut list = responses_signal();
                                                                    if idx < list.len() {
                                                                        list[idx].notes = Some(e.value());
                                                                        responses_signal.set(list);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                div {
                                                    input {
                                                        class: "form-input",
                                                        style: "font-size: 13px; padding: 7px 12px;",
                                                        placeholder: "Digite a resposta...",
                                                        value: "{text_val}",
                                                        disabled: !can_write,
                                                        oninput: move |e| {
                                                             let mut list = responses_signal();
                                                             if idx < list.len() {
                                                                 list[idx].answer_text = Some(e.value());
                                                                 responses_signal.set(list);
                                                             }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // =========================================================================
            // DIVISÃO DA ASSINATURA DIGITAL (DEPOIS DA FICHA DE ANAMNESE)
            // =========================================================================
            div {
                class: "anamnese-signature-status-card",
                style: if is_signed {
                    "background: #f0fdf4; border: 1px solid #bbf7d0; border-radius: 12px; padding: 18px 22px; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 14px;"
                } else if is_pending {
                    "background: #fffbeb; border: 1px solid #fde68a; border-radius: 12px; padding: 18px 22px; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 14px;"
                } else {
                    "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 12px; padding: 18px 22px; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 14px;"
                },
                div { style: "display: flex; align-items: center; gap: 14px;",
                    div {
                        style: if is_signed {
                            "width: 44px; height: 44px; border-radius: 10px; background: #dcfce7; display: flex; align-items: center; justify-content: center;"
                        } else if is_pending {
                            "width: 44px; height: 44px; border-radius: 10px; background: #fef3c7; display: flex; align-items: center; justify-content: center;"
                        } else {
                            "width: 44px; height: 44px; border-radius: 10px; background: #e2e8f0; display: flex; align-items: center; justify-content: center;"
                        },
                        if is_signed {
                            IconCheckCircle { size: 22, color: "#16a34a".to_string() }
                        } else if is_pending {
                            IconSignature { size: 22, color: "#d97706".to_string() }
                        } else {
                            IconSignature { size: 22, color: "#475569".to_string() }
                        }
                    }
                    div {
                        h4 { style: "margin: 0; font-size: 15px; font-weight: 700; color: #0f172a;", "{signature_title}" }
                        p { style: "margin: 2px 0 0; font-size: 12.5px; color: #475569;", "{signature_desc}" }
                    }
                }

                // Ações da Assinatura
                div { style: "display: flex; gap: 8px; align-items: center; flex-wrap: wrap;",
                    if is_signed {
                        span {
                            style: "display: inline-flex; align-items: center; gap: 6px; font-size: 13px; font-weight: 600; color: #16a34a; background: #dcfce7; padding: 6px 12px; border-radius: 8px;",
                            IconCheckCircle { size: 16, color: "#16a34a".to_string() }
                            span { "Termo Válido & Assinado" }
                        }
                        if can_write {
                            button {
                                class: "btn-secondary",
                                style: "font-size: 13px; padding: 7px 14px; display: inline-flex; align-items: center; gap: 6px;",
                                disabled: is_requesting_sign(),
                                onclick: handle_request_signature.clone(),
                                IconRefresh { size: 14, color: "currentColor".to_string() }
                                span { "Solicitar Nova Assinatura" }
                            }
                        }
                    } else if is_pending {
                        if let Some(ref tok_str) = anam.signing_token {
                            {
                                let t_s = tok_str.clone();
                                let p_u = anam.signed_pdf_url.clone().unwrap_or_default();
                                let sign_link = build_signing_url(&t_s);
                                let sign_link_c = sign_link.clone();
                                rsx! {
                                    button {
                                        class: "btn-primary",
                                        style: "font-size: 13px; padding: 7px 16px; display: inline-flex; align-items: center; gap: 6px; background-color: #0052cc;",
                                        onclick: move |_| qr_modal_data.set(Some((t_s.clone(), sign_link_c.clone(), p_u.clone()))),
                                        IconQrCode { size: 14, color: "#ffffff".to_string() }
                                        span { "Ver QR Code / Link" }
                                    }
                                    a {
                                        href: "{sign_link}",
                                        target: "_blank",
                                        class: "btn-secondary",
                                        style: "font-size: 13px; padding: 7px 14px; display: inline-flex; align-items: center; gap: 6px; text-decoration: none;",
                                        IconExternalLink { size: 14, color: "#0052cc".to_string() }
                                        span { "Assinar no Tablet/Tela" }
                                    }
                                    if can_write {
                                        button {
                                            class: "btn-secondary",
                                            style: "font-size: 12.5px; padding: 7px 12px; display: inline-flex; align-items: center; gap: 5px;",
                                            disabled: is_requesting_sign(),
                                            onclick: handle_request_signature.clone(),
                                            IconRefresh { size: 13, color: "currentColor".to_string() }
                                            span { "Gerar Novo Link" }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        if can_write {
                            button {
                                class: "btn-primary",
                                style: "font-size: 13px; padding: 8px 18px; font-weight: 600; display: inline-flex; align-items: center; gap: 6px;",
                                disabled: is_requesting_sign(),
                                onclick: handle_request_signature.clone(),
                                IconSignature { size: 16, color: "#ffffff".to_string() }
                                span { if is_requesting_sign() { "Gerando Termo..." } else { "Enviar para Assinatura do Paciente" } }
                            }
                        }
                    }
                }
            }

            // Modal de Confirmação: Sincronizar Ficha com Modelo Mais Recente
            if is_sync_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal delete-modal-card",
                        div { class: "settings-header",
                            h2 { class: "settings-title", "Atualizar Modelo de Anamnese" }
                            button { class: "close-btn", onclick: move |_| is_sync_modal_open.set(false), "×" }
                        }
                        div { class: "settings-content",
                            p { "Deseja sincronizar esta ficha com a versão mais recente do modelo padrão da clínica?" }
                            p { class: "text-muted font-xs mt-2",
                                "As respostas previamente preenchidas serão preservadas. Apenas perguntas ausentes serão adicionadas à ficha deste paciente."
                            }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| is_sync_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_syncing(),
                                onclick: {
                                    let t_s = token.clone();
                                    let p_s = patient_id.clone();
                                    let c_s = clinic_id.clone();
                                    let tt_s = template_type_str.clone();
                                    let on_r = reload_patient_details.clone();
                                    move |_| {
                                        let req = SyncAnamnesisRequest {
                                            clinic_id: c_s.clone(),
                                            template_type: Some(tt_s.clone()),
                                        };

                                        let tok_clone = t_s.clone();
                                        let pat_clone = p_s.clone();
                                        let reload_clone = on_r.clone();

                                        is_syncing.set(true);
                                        spawn(async move {
                                            match sync_patient_anamnesis(&tok_clone, &pat_clone, req).await {
                                                Ok(updated_anam) => {
                                                    is_sync_modal_open.set(false);
                                                    responses_signal.set(updated_anam.custom_responses);
                                                    toast_msg.set(Some("Ficha sincronizada com as perguntas mais recentes da clínica!".into()));
                                                    reload_clone.call(());
                                                }
                                                Err(e) => {
                                                    error_toast.set(Some(format!("Erro ao sincronizar ficha: {}", e)));
                                                }
                                            }
                                            is_syncing.set(false);
                                        });
                                    }
                                },
                                if is_syncing() { "Atualizando..." } else { "Confirmar Atualização da Ficha" }
                            }
                        }
                    }
                }
            }

            // Modal de QR Code e Envio por WhatsApp para Assinatura da Anamnese
            if let Some((ref sign_token, ref sign_url, ref _pdf_url)) = *qr_modal_data.read() {
                {
                    let qr_svg = generate_qr_svg(sign_url);
                    let wa_msg = format!("Olá! Por favor, acesse o link seguro para assinar digitalmente a sua Ficha de Anamnese: {}", sign_url);
                    let wa_link = format!("https://api.whatsapp.com/send?text={}", wa_msg.replace(" ", "%20"));
                    let s_url = sign_url.clone();
                    rsx! {
                        div { class: "modal-overlay",
                            div { class: "action-modal qr-sign-modal",
                                div { class: "modal-header",
                                    div {
                                        h2 { class: "modal-title", "Assinatura Digital da Anamnese" }
                                        p { class: "modal-subtitle", "Aponte a câmera do celular para o QR Code ou compartilhe o link direto com o paciente." }
                                    }
                                    button { class: "modal-close", onclick: move |_| qr_modal_data.set(None), "×" }
                                }
                                div { class: "modal-body text-center",
                                    div { class: "qr-box-center",
                                        div {
                                            class: "qr-svg-wrapper",
                                            dangerous_inner_html: "{qr_svg}"
                                        }
                                    }

                                    p { class: "qr-doc-title", "Ficha Oficial de Anamnese Odontológica" }
                                    p { class: "qr-hint", "O paciente ou responsável poderá conferir as declarações de saúde e desenhar sua assinatura manuscrita na tela." }

                                    div { class: "qr-link-copy-box",
                                        input {
                                            r#type: "text",
                                            readonly: true,
                                            class: "input-field font-mono font-xs",
                                            value: "{s_url}"
                                        }
                                        a {
                                            href: "{s_url}",
                                            target: "_blank",
                                            class: "btn-secondary",
                                            IconExternalLink { size: 16, color: "#0052cc".to_string() }
                                            span { " Abrir Portal" }
                                        }
                                    }
                                }
                                div { class: "modal-footer", style: "display: flex; gap: 8px; justify-content: flex-end;",
                                    a {
                                        class: "btn-primary",
                                        style: "background-color: #25D366; border-color: #25D366; color: #ffffff; display: inline-flex; align-items: center; gap: 8px; text-decoration: none;",
                                        href: "{wa_link}",
                                        target: "_blank",
                                        IconWhatsApp { size: 18, color: "#ffffff".to_string() }
                                        span { "Enviar Link por WhatsApp" }
                                    }
                                    button { class: "btn-secondary", onclick: move |_| qr_modal_data.set(None), "Fechar" }
                                }
                            }
                        }
                    }
                }
            }

            // Modal de Pré-Visualização de PDF da Anamnese
            if let Some((ref pdf_url, ref title)) = *pdf_preview_target.read() {
                {
                    let resolved_pdf_url = if pdf_url.starts_with("http") {
                        pdf_url.clone()
                    } else {
                        format!("http://localhost:4000{}", pdf_url)
                    };
                    rsx! {
                        div { class: "modal-overlay",
                            onclick: move |_| pdf_preview_target.set(None),
                            div { class: "action-modal pdf-viewer-modal",
                                onclick: move |e| e.stop_propagation(),
                                div { class: "modal-header",
                                    div {
                                        h2 { class: "modal-title", "{title}" }
                                        p { class: "modal-subtitle", "Documento PDF da Anamnese Assinada" }
                                    }
                                    button { class: "modal-close", onclick: move |_| pdf_preview_target.set(None), "×" }
                                }
                                div { class: "modal-body p-0", style: "height: 68vh; overflow: hidden; background: #0f172a;",
                                    iframe {
                                        src: "{resolved_pdf_url}",
                                        style: "width: 100%; height: 100%; border: none;",
                                        title: "{title}"
                                    }
                                }
                                div { class: "modal-footer",
                                    div { class: "flex items-center justify-between full-width",
                                        a {
                                            class: "btn-secondary",
                                            href: "{resolved_pdf_url}",
                                            target: "_blank",
                                            IconExternalLink { size: 14, color: "currentColor".to_string() }
                                            span { " Abrir em Nova Aba" }
                                        }
                                        button { class: "btn-primary", onclick: move |_| pdf_preview_target.set(None), "Fechar" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
