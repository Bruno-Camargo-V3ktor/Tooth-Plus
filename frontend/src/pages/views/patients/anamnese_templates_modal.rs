//! # Modal de Configuração de Modelos de Anamnese (Frontend)
//!
//! Permite à clínica configurar e customizar as perguntas da ficha de anamnese
//! para pacientes Adultos e Menores de Idade (Odontopediatria), com adição, edição em linha
//! e restauração para as perguntas oficiais da clínica (conforme referência física).

use crate::api::{fetch_anamnesis_templates, save_anamnesis_template};
use crate::components::icons::{IconCheck, IconEdit, IconHeartPulse, IconPlus, IconRefresh, IconTrash, IconX};
use dioxus::prelude::*;
use shared::anamnesis::{AnamnesisQuestion, AnamnesisTemplate, SaveAnamnesisTemplateRequest};

/// Retorna as 15 perguntas padrão oficiais para adultos da clínica (conforme ficha física de referência).
fn get_official_adult_questions() -> Vec<AnamnesisQuestion> {
    vec![
        AnamnesisQuestion {
            id: "prof_occupation".into(),
            category: "Dados Gerais".into(),
            question_text: "Profissão".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "chief_complaint".into(),
            category: "Queixa Principal".into(),
            question_text: "Queixa principal?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "brushing_frequency".into(),
            category: "Higiene Bucal".into(),
            question_text: "Quantas vezes por dia escova os dentes?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "under_medical_treatment".into(),
            category: "Saúde Geral".into(),
            question_text: "Está em tratamento médico?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "using_medication".into(),
            category: "Medicamentos".into(),
            question_text: "Está usando medicação?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "has_allergies".into(),
            category: "Alergias".into(),
            question_text: "Possui alguma alergia? (Como penicilinas, AAS ou outra)".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "diagnosed_hemorrhage".into(),
            category: "Histórico Clínico".into(),
            question_text: "Já teve hemorragia diagnosticada?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "cardiovascular_disorder".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Possui alguma alteração cardiovascular?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "hypertension".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Tem pressão alta?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "diabetes".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Possui diabetes?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "transmissible_disease".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Possui alguma doença transmissível? (HIV, Hepatite, outra)".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "teeth_grinding".into(),
            category: "Hábitos".into(),
            question_text: "Range os dentes?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "smoker".into(),
            category: "Hábitos".into(),
            question_text: "Fumante?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "alcohol_consumption".into(),
            category: "Hábitos".into(),
            question_text: "Ingere bebidas alcoólicas?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "oral_surgery_history".into(),
            category: "Histórico Odontológico".into(),
            question_text: "Já se submeteu à Cirurgia Oral (exodontia, freio labial, etc.)?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
    ]
}

/// Retorna as 12 perguntas padrão oficiais para menores / odontopediatria.
fn get_official_minor_questions() -> Vec<AnamnesisQuestion> {
    vec![
        AnamnesisQuestion {
            id: "ped_chief_complaint".into(),
            category: "Queixa Principal".into(),
            question_text: "Queixa principal dos pais / responsáveis?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_habits".into(),
            category: "Hábitos Infantis".into(),
            question_text: "Hábitos infantis (Chupeta, sucção de dedo, mamadeira noturna, roer unhas)?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_birth_nursing".into(),
            category: "Histórico Pediátrico".into(),
            question_text: "Histórico de parto e amamentação (Parto normal/cesárea, amamentação)?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_allergies".into(),
            category: "Alergias Pediátricas".into(),
            question_text: "Possui alguma alergia a medicamentos, alimentos ou substâncias?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_medication".into(),
            category: "Medicamentos".into(),
            question_text: "Faz uso contínuo de algum medicamento, vitamina ou xarope?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_complication".into(),
            category: "Histórico Clínico".into(),
            question_text: "Já teve alguma complicação em anestesia ou internação prévia?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_respiratory".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Possui problemas respiratórios (Asma, Bronquite, Rinite, respiração bucal)?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_bruxism".into(),
            category: "Hábitos Infantis".into(),
            question_text: "Range os dentes durante o sono (Bruxismo infantil)?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_trauma".into(),
            category: "Histórico Odontológico".into(),
            question_text: "Sofreu algum trauma dental ou queda recente na boca?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_sweets".into(),
            category: "Hábitos e Dieta".into(),
            question_text: "Frequência de ingestão de doces / açúcar entre as refeições?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_hygiene".into(),
            category: "Higiene Bucal".into(),
            question_text: "Como é a aceitação da escovação e uso de fio dental em casa?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_previous_visit".into(),
            category: "Histórico Odontológico".into(),
            question_text: "Já consultou odontopediatra anteriormente?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
    ]
}

#[component]
pub fn AnamneseTemplatesModal(
    is_open: Signal<bool>,
    token: String,
    clinic_id: String,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut selected_type = use_signal(|| "adult".to_string());
    let mut templates = use_signal(Vec::<AnamnesisTemplate>::new);
    let mut is_loading = use_signal(|| true);
    let mut is_saving = use_signal(|| false);

    // Form para nova pergunta
    let mut is_adding_new = use_signal(|| false);
    let mut new_q_text = use_signal(String::new);
    let mut new_q_category = use_signal(|| "Saúde Sistêmica".to_string());
    let mut new_q_type = use_signal(|| "yes_no".to_string());

    // Estado para edição de pergunta existente
    let mut editing_q_id = use_signal(|| None::<String>);
    let mut edit_q_text = use_signal(String::new);
    let mut edit_q_category = use_signal(String::new);
    let mut edit_q_type = use_signal(String::new);

    let tok = token.clone();
    let cid = clinic_id.clone();

    // Carregar templates ao abrir
    {
        let t = tok.clone();
        let c = cid.clone();
        use_effect(move || {
            let t_clone = t.clone();
            let c_clone = c.clone();
            is_loading.set(true);
            spawn(async move {
                match fetch_anamnesis_templates(&t_clone, &c_clone).await {
                    Ok(data) => {
                        templates.set(data);
                    }
                    Err(e) => {
                        error_toast.set(Some(format!("Erro ao carregar modelos: {}", e)));
                    }
                }
                is_loading.set(false);
            });
        });
    }

    let current_template = templates()
        .into_iter()
        .find(|t| t.template_type == selected_type())
        .unwrap_or_else(|| AnamnesisTemplate {
            id: String::new(),
            clinic_id: cid.clone(),
            template_type: selected_type(),
            title: if selected_type() == "minor" {
                "Ficha Padrão - Menor / Odontopediatria".into()
            } else {
                "Ficha Padrão - Adulto".into()
            },
            questions: if selected_type() == "minor" { get_official_minor_questions() } else { get_official_adult_questions() },
            created_at: String::new(),
            updated_at: String::new(),
        });

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal stock-custom-modal", style: "max-width: 900px;",
                div { class: "settings-header",
                    div {
                        h2 { class: "settings-title",
                            IconHeartPulse { size: 22, color: "#0052cc".to_string() }
                            span { " Modelos de Ficha de Anamnese" }
                        }
                        p { class: "text-muted font-xs mt-1",
                            "Configure as perguntas oficiais de histórico médico e bucal da clínica. As alterações serão aplicadas automaticamente para novos pacientes."
                        }
                    }
                    button {
                        class: "close-btn",
                        onclick: move |_| is_open.set(false),
                        "×"
                    }
                }

                div { class: "settings-content", style: "max-height: 70vh; overflow-y: auto; padding-right: 4px;",
                    // Seletor de Modelo e Ações de Topo
                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; border-bottom: 1px solid #e2e8f0; padding-bottom: 12px; flex-wrap: wrap; gap: 10px;",
                        div { class: "documents-tab-bar", style: "margin-bottom: 0;",
                            button {
                                class: if selected_type() == "adult" { "documents-tab-btn active" } else { "documents-tab-btn" },
                                onclick: move |_| {
                                    selected_type.set("adult".to_string());
                                    editing_q_id.set(None);
                                    is_adding_new.set(false);
                                },
                                "📋 Ficha Oficial: Adulto"
                            }
                            button {
                                class: if selected_type() == "minor" { "documents-tab-btn active" } else { "documents-tab-btn" },
                                onclick: move |_| {
                                    selected_type.set("minor".to_string());
                                    editing_q_id.set(None);
                                    is_adding_new.set(false);
                                },
                                "🧸 Ficha Oficial: Pediátrica / Menor"
                            }
                        }

                        div { style: "display: flex; align-items: center; gap: 8px;",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                style: "font-size: 12px; padding: 6px 12px; display: inline-flex; align-items: center; gap: 6px;",
                                onclick: {
                                    let cid_r = cid.clone();
                                    move |_| {
                                        let t_type = selected_type();
                                        let default_questions = if t_type == "minor" {
                                            get_official_minor_questions()
                                        } else {
                                            get_official_adult_questions()
                                        };

                                        let mut current_list = templates();
                                        if let Some(pos) = current_list.iter().position(|t| t.template_type == t_type) {
                                            current_list[pos].questions = default_questions;
                                        } else {
                                            current_list.push(AnamnesisTemplate {
                                                id: String::new(),
                                                clinic_id: cid_r.clone(),
                                                template_type: t_type.clone(),
                                                title: if t_type == "minor" {
                                                    "Ficha Padrão - Menor / Odontopediatria".into()
                                                } else {
                                                    "Ficha Padrão - Adulto".into()
                                                },
                                                questions: default_questions,
                                                created_at: String::new(),
                                                updated_at: String::new(),
                                            });
                                        }
                                        templates.set(current_list);
                                        toast_msg.set(Some("Perguntas restauradas para o padrão oficial da ficha da clínica!".into()));
                                    }
                                },
                                title: "Restaura o questionário padrão com 15 perguntas oficiais",
                                IconRefresh { size: 13, color: "#475569".to_string() }
                                span { "Restaurar Padrão da Clínica" }
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary",
                                style: "font-size: 12px; padding: 6px 14px;",
                                onclick: move |_| is_adding_new.set(!is_adding_new()),
                                IconPlus { size: 13, color: "currentColor".to_string() }
                                span { if is_adding_new() { " Fechar Formulário" } else { " + Nova Pergunta" } }
                            }
                        }
                    }

                    if is_loading() {
                        div { class: "loading-card",
                            div { class: "loading-spinner" }
                            p { "Carregando modelo de anamnese..." }
                        }
                    } else {
                        // Formulário de Adicionar Nova Pergunta (Destaque no Topo)
                        if is_adding_new() {
                            div { class: "agenda-resource-box", style: "margin-bottom: 18px; border: 2px solid #3b82f6; background: #eff6ff; border-radius: 10px; padding: 16px;",
                                div { class: "resource-section-header", style: "color: #1d4ed8; font-weight: 700; font-size: 13.5px;",
                                    span { "+ Inserir Nova Pergunta no Questionário" }
                                }

                                div { class: "form-grid-2", style: "margin-top: 12px;",
                                    div { class: "form-group full-width", style: "grid-column: 1 / -1;",
                                        label { "Texto da Pergunta *" }
                                        input {
                                            class: "form-input",
                                            placeholder: "Ex: Já se submeteu à Cirurgia Oral (exodontia, freio labial, etc.)?",
                                            value: "{new_q_text}",
                                            oninput: move |e| new_q_text.set(e.value())
                                        }
                                    }
                                    div { class: "form-group",
                                        label { "Categoria / Seção" }
                                        select {
                                            class: "form-input",
                                            value: "{new_q_category}",
                                            onchange: move |e| new_q_category.set(e.value()),
                                            option { value: "Dados Gerais", "Dados Gerais" }
                                            option { value: "Queixa Principal", "Queixa Principal" }
                                            option { value: "Higiene Bucal", "Higiene Bucal" }
                                            option { value: "Saúde Geral", "Saúde Geral" }
                                            option { value: "Medicamentos", "Medicamentos" }
                                            option { value: "Alergias", "Alergias" }
                                            option { value: "Histórico Clínico", "Histórico Clínico" }
                                            option { value: "Saúde Sistêmica", "Saúde Sistêmica" }
                                            option { value: "Hábitos", "Hábitos" }
                                            option { value: "Histórico Odontológico", "Histórico Odontológico" }
                                            option { value: "Hábitos Infantis", "Hábitos Infantis" }
                                            option { value: "Histórico Pediátrico", "Histórico Pediátrico" }
                                            option { value: "Alergias Pediátricas", "Alergias Pediátricas" }
                                            option { value: "Hábitos e Dieta", "Hábitos e Dieta" }
                                        }
                                    }
                                    div { class: "form-group",
                                        label { "Tipo de Resposta" }
                                        select {
                                            class: "form-input",
                                            value: "{new_q_type}",
                                            onchange: move |e| new_q_type.set(e.value()),
                                            option { value: "yes_no", "Sim / Não (com campo de observação)" }
                                            option { value: "text", "Texto Livre" }
                                        }
                                    }
                                }

                                div { style: "display: flex; justify-content: flex-end; gap: 8px; margin-top: 14px;",
                                    button {
                                        r#type: "button",
                                        class: "btn-secondary",
                                        style: "padding: 6px 12px; font-size: 13px;",
                                        onclick: move |_| is_adding_new.set(false),
                                        "Cancelar"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn-primary",
                                        style: "padding: 6px 16px; font-size: 13px;",
                                        onclick: {
                                            let cid_a = cid.clone();
                                            move |_| {
                                                let text = new_q_text().trim().to_string();
                                                if text.is_empty() {
                                                    error_toast.set(Some("Informe o texto da pergunta.".into()));
                                                    return;
                                                }

                                                let q_id = format!("q_{}", chrono::Utc::now().timestamp_millis());
                                                let new_q = AnamnesisQuestion {
                                                    id: q_id,
                                                    category: new_q_category(),
                                                    question_text: text,
                                                    question_type: new_q_type(),
                                                    options: None,
                                                    required: false,
                                                };

                                                let t_type = selected_type();
                                                let mut current_list = templates();
                                                if let Some(pos) = current_list.iter().position(|t| t.template_type == t_type) {
                                                    current_list[pos].questions.push(new_q);
                                                } else {
                                                    let t = AnamnesisTemplate {
                                                        id: String::new(),
                                                        clinic_id: cid_a.clone(),
                                                        template_type: t_type.clone(),
                                                        title: if t_type == "minor" {
                                                            "Ficha Padrão - Menor / Odontopediatria".into()
                                                        } else {
                                                            "Ficha Padrão - Adulto".into()
                                                        },
                                                        questions: vec![new_q],
                                                        created_at: String::new(),
                                                        updated_at: String::new(),
                                                    };
                                                    current_list.push(t);
                                                }
                                                templates.set(current_list);
                                                new_q_text.set(String::new());
                                                is_adding_new.set(false);
                                            }
                                        },
                                        IconPlus { size: 14, color: "currentColor".to_string() }
                                        span { " Inserir Pergunta" }
                                    }
                                }
                            }
                        }

                        // Tabela / Lista de perguntas atuais
                        div { class: "table-container", style: "margin-bottom: 20px;",
                            div { style: "padding: 12px 18px; background: #f8fafc; border-bottom: 1px solid #e2e8f0; display: flex; justify-content: space-between; align-items: center;",
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    strong { style: "font-size: 13.5px; color: #0f172a;", "Perguntas Configuradas na Ficha" }
                                    span { class: "badge-insurance-plan font-mono font-xs", "{current_template.questions.len()} perguntas" }
                                }
                                span { class: "text-muted font-xs", "Ordem de exibição idêntica à ficha de atendimento" }
                            }

                            if current_template.questions.is_empty() {
                                div { class: "empty-state-card", style: "padding: 30px 20px;",
                                    p { "Nenhuma pergunta configurada neste modelo." }
                                    button {
                                        r#type: "button",
                                        class: "btn-secondary",
                                        style: "margin-top: 10px;",
                                        onclick: {
                                            let cid_d = cid.clone();
                                            move |_| {
                                                let t_type = selected_type();
                                                let default_questions = if t_type == "minor" {
                                                    get_official_minor_questions()
                                                } else {
                                                    get_official_adult_questions()
                                                };
                                                let mut current_list = templates();
                                                if let Some(pos) = current_list.iter().position(|t| t.template_type == t_type) {
                                                    current_list[pos].questions = default_questions;
                                                } else {
                                                    current_list.push(AnamnesisTemplate {
                                                        id: String::new(),
                                                        clinic_id: cid_d.clone(),
                                                        template_type: t_type.clone(),
                                                        title: if t_type == "minor" {
                                                            "Ficha Padrão - Menor / Odontopediatria".into()
                                                        } else {
                                                            "Ficha Padrão - Adulto".into()
                                                        },
                                                        questions: default_questions,
                                                        created_at: String::new(),
                                                        updated_at: String::new(),
                                                    });
                                                }
                                                templates.set(current_list);
                                            }
                                        },
                                        "Carregar Perguntas Padrão da Clínica"
                                    }
                                }
                            } else {
                                table { class: "modern-table",
                                    thead {
                                        tr {
                                            th { style: "width: 50px; text-align: center;", "#" }
                                            th { "Pergunta" }
                                            th { style: "width: 170px;", "Categoria" }
                                            th { style: "width: 140px;", "Tipo" }
                                            th { style: "width: 90px; text-align: right;", "Ações" }
                                        }
                                    }
                                    tbody {
                                        for (idx, q) in current_template.questions.iter().enumerate() {
                                            {
                                                let q_id = q.id.clone();
                                                let q_clone = q.clone();
                                                let is_editing = editing_q_id().as_deref() == Some(&q_id);
                                                let q_type_badge = match q.question_type.as_str() {
                                                    "yes_no" => ("background: #f0fdf4; color: #166534; border: 1px solid #bbf7d0;", "Sim / Não"),
                                                    "text" => ("background: #eff6ff; color: #1e40af; border: 1px solid #bfdbfe;", "Texto Livre"),
                                                    _ => ("background: #f8fafc; color: #475569; border: 1px solid #e2e8f0;", "Múltipla"),
                                                };

                                                rsx! {
                                                    tr { key: "{q.id}", style: if is_editing { "background: #fefce8;" } else { "" },
                                                        if is_editing {
                                                            td { colspan: "5", style: "padding: 16px;",
                                                                div { style: "display: flex; flex-direction: column; gap: 10px;",
                                                                    div { style: "display: align-items: center; justify-content: space-between;",
                                                                        span { style: "font-weight: 700; font-size: 13px; color: #854d0e;", "Editando Pergunta #{idx + 1}" }
                                                                        div { style: "display: flex; gap: 6px;",
                                                                            button {
                                                                                r#type: "button",
                                                                                class: "btn-secondary",
                                                                                style: "padding: 4px 10px; font-size: 12px;",
                                                                                onclick: move |_| editing_q_id.set(None),
                                                                                IconX { size: 13, color: "currentColor".to_string() }
                                                                                span { " Cancelar" }
                                                                            }
                                                                            button {
                                                                                r#type: "button",
                                                                                class: "btn-primary",
                                                                                style: "padding: 4px 12px; font-size: 12px;",
                                                                                onclick: {
                                                                                    let target_q_id = q_id.clone();
                                                                                    move |_| {
                                                                                        let text = edit_q_text().trim().to_string();
                                                                                        if text.is_empty() {
                                                                                            error_toast.set(Some("O texto da pergunta não pode ser vazio.".into()));
                                                                                            return;
                                                                                        }

                                                                                        let mut current_list = templates();
                                                                                        if let Some(pos) = current_list.iter().position(|t| t.template_type == selected_type()) {
                                                                                            if let Some(q_pos) = current_list[pos].questions.iter().position(|q| q.id == target_q_id) {
                                                                                                                                                                                current_list[pos].questions[q_pos].question_text = text;
                                                                                                current_list[pos].questions[q_pos].category = edit_q_category();
                                                                                                current_list[pos].questions[q_pos].question_type = edit_q_type();
                                                                                                templates.set(current_list);
                                                                                            }
                                                                                        }
                                                                                        editing_q_id.set(None);
                                                                                    }
                                                                                },
                                                                                IconCheck { size: 13, color: "currentColor".to_string() }
                                                                                span { " Salvar" }
                                                                            }
                                                                        }
                                                                    }
                                                                    input {
                                                                        class: "form-input",
                                                                        value: "{edit_q_text}",
                                                                        oninput: move |e| edit_q_text.set(e.value())
                                                                    }
                                                                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                                                                        select {
                                                                            class: "form-input",
                                                                            value: "{edit_q_category}",
                                                                            onchange: move |e| edit_q_category.set(e.value()),
                                                                            option { value: "Dados Gerais", "Dados Gerais" }
                                                                            option { value: "Queixa Principal", "Queixa Principal" }
                                                                            option { value: "Higiene Bucal", "Higiene Bucal" }
                                                                            option { value: "Saúde Geral", "Saúde Geral" }
                                                                            option { value: "Medicamentos", "Medicamentos" }
                                                                            option { value: "Alergias", "Alergias" }
                                                                            option { value: "Histórico Clínico", "Histórico Clínico" }
                                                                            option { value: "Saúde Sistêmica", "Saúde Sistêmica" }
                                                                            option { value: "Hábitos", "Hábitos" }
                                                                            option { value: "Histórico Odontológico", "Histórico Odontológico" }
                                                                            option { value: "Hábitos Infantis", "Hábitos Infantis" }
                                                                            option { value: "Histórico Pediátrico", "Histórico Pediátrico" }
                                                                            option { value: "Alergias Pediátricas", "Alergias Pediátricas" }
                                                                            option { value: "Hábitos e Dieta", "Hábitos e Dieta" }
                                                                        }
                                                                        select {
                                                                            class: "form-input",
                                                                            value: "{edit_q_type}",
                                                                            onchange: move |e| edit_q_type.set(e.value()),
                                                                            option { value: "yes_no", "Sim / Não (com observação)" }
                                                                            option { value: "text", "Texto Livre" }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            td { style: "text-align: center; color: #94a3b8; font-weight: 700; font-size: 12px;", "#{idx + 1}" }
                                                            td { style: "font-weight: 500; color: #1e293b; font-size: 13.5px;", "{q.question_text}" }
                                                            td {
                                                                span { style: "background: #f1f5f9; color: #334155; padding: 3px 8px; border-radius: 6px; font-size: 12px; font-weight: 500;",
                                                                    "{q.category}"
                                                                }
                                                            }
                                                            td {
                                                                span { style: "padding: 3px 8px; border-radius: 6px; font-size: 11.5px; font-weight: 600; {q_type_badge.0}",
                                                                    "{q_type_badge.1}"
                                                                }
                                                            }
                                                            td { style: "text-align: right;",
                                                                div { style: "display: inline-flex; align-items: center; gap: 4px;",
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "btn-action-icon",
                                                                        onclick: {
                                                                            let q_c = q_clone.clone();
                                                                            move |_| {
                                                                                editing_q_id.set(Some(q_c.id.clone()));
                                                                                edit_q_text.set(q_c.question_text.clone());
                                                                                edit_q_category.set(q_c.category.clone());
                                                                                edit_q_type.set(q_c.question_type.clone());
                                                                            }
                                                                        },
                                                                        title: "Editar pergunta",
                                                                        IconEdit { size: 14, color: "#475569".to_string() }
                                                                    }
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "btn-action-icon text-danger",
                                                                        onclick: {
                                                                            let target_id = q_id.clone();
                                                                            move |_| {
                                                                                let mut current_list = templates();
                                                                                if let Some(pos) = current_list.iter().position(|t| t.template_type == selected_type()) {
                                                                                    current_list[pos].questions.retain(|item| item.id != target_id);
                                                                                    templates.set(current_list);
                                                                                }
                                                                            }
                                                                        },
                                                                        title: "Remover pergunta",
                                                                        IconTrash { size: 14, color: "#ef4444".to_string() }
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
                }

                div { class: "modal-footer-actions",
                    button {
                        r#type: "button",
                        class: "btn-secondary",
                        onclick: move |_| is_open.set(false),
                        "Fechar"
                    }
                    button {
                        r#type: "button",
                        class: "btn-primary",
                        disabled: is_saving(),
                        onclick: {
                            let t = tok.clone();
                            let cid_s = cid.clone();
                            move |_| {
                                let t_type = selected_type();
                                let target = templates()
                                    .into_iter()
                                    .find(|tmpl| tmpl.template_type == t_type)
                                    .unwrap_or_else(|| AnamnesisTemplate {
                                        id: String::new(),
                                        clinic_id: cid_s.clone(),
                                        template_type: t_type.clone(),
                                        title: if t_type == "minor" {
                                            "Ficha Padrão - Menor / Odontopediatria".into()
                                        } else {
                                            "Ficha Padrão - Adulto".into()
                                        },
                                        questions: if t_type == "minor" { get_official_minor_questions() } else { get_official_adult_questions() },
                                        created_at: String::new(),
                                        updated_at: String::new(),
                                    });

                                let req = SaveAnamnesisTemplateRequest {
                                    clinic_id: cid_s.clone(),
                                    template_type: t_type,
                                    title: target.title,
                                    questions: target.questions,
                                };

                                let t_clone = t.clone();
                                is_saving.set(true);
                                spawn(async move {
                                    match save_anamnesis_template(&t_clone, req).await {
                                        Ok(_) => {
                                            toast_msg.set(Some("Modelo de ficha de anamnese salvo com sucesso!".into()));
                                        }
                                        Err(e) => {
                                            error_toast.set(Some(format!("Erro ao salvar modelo: {}", e)));
                                        }
                                    }
                                    is_saving.set(false);
                                });
                            }
                        },
                        if is_saving() { "Salvando Modelo..." } else { "Salvar Alterações no Modelo" }
                    }
                }
            }
        }
    }
}
