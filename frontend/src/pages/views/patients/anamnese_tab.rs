//! # Aba de Anamnese Odontológica (Frontend)
//!
//! Exibe e permite preencher/editar a Ficha Oficial de Anamnese do paciente
//! com layout de tabela Pergunta | Resposta idêntico ao formulário clínico de referência.

use crate::api::{save_patient_anamnesis, sync_patient_anamnesis};
use crate::components::icons::{IconHeartPulse, IconRefresh};
use dioxus::prelude::*;
use shared::anamnesis::{AnamnesisResponseItem, SyncAnamnesisRequest};
use shared::patients::{PatientAnamnesis, SaveAnamnesisRequest};

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
                answer_text: Some("3 vezes ao dia / usa fio dental".into()),
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
                answer_boolean: Some(anam.systemic_diseases.iter().any(|d| d.contains("Cardiopatia"))),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "hypertension".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Tem pressão alta?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.systemic_diseases.iter().any(|d| d.contains("Hipertensão"))),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "diabetes".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Possui diabetes?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.systemic_diseases.iter().any(|d| d.contains("Diabetes"))),
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
                question_id: "teeth_grinding".into(),
                category: "Hábitos".into(),
                question_text: "Range os dentes?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.bruxism),
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

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();
    let t_type_for_save = template_type_str.clone();

    let template_badge = if is_minor_template { "Ficha Oficial: Odontopediatria (Menor)" } else { "Ficha Oficial: Adulto" };
    let current_responses = responses_signal();

    rsx! {
        div { class: "anamnese-cards-container",
            // Header com Badges e Ações
            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; flex-wrap: wrap; gap: 12px;",
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
            div { class: "table-container", style: "margin-bottom: 24px; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.05);",
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
                                        // Coluna 2: Resposta
                                        td { style: "vertical-align: top; padding: 14px 18px;",
                                            if is_yes_no {
                                                div { style: "display: flex; flex-direction: column; gap: 8px;",
                                                    div { style: "display: flex; align-items: center; gap: 10px;",
                                                        // Botão Não
                                                        button {
                                                            r#type: "button",
                                                            class: if is_no { "btn-primary" } else { "btn-secondary" },
                                                            style: if is_no {
                                                                "padding: 5px 16px; font-size: 12.5px; font-weight: 600; background: #0f172a; border-color: #0f172a;"
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
                                                        // Botão Sim
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
                                                    // Campo de observação detalhada quando "Sim"
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
        }
    }
}
