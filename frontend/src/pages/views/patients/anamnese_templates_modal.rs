//! # Modal de Configuração de Modelos de Anamnese (Frontend)
//!
//! Permite à clínica configurar e customizar as perguntas da ficha de anamnese
//! para pacientes Adultos e Menores de Idade (Odontopediatria).

use crate::api::{fetch_anamnesis_templates, save_anamnesis_template};
use crate::components::icons::{IconHeartPulse, IconPlus, IconTrash};
use dioxus::prelude::*;
use shared::anamnesis::{AnamnesisQuestion, AnamnesisTemplate, SaveAnamnesisTemplateRequest};

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
    let mut new_q_text = use_signal(String::new);
    let mut new_q_category = use_signal(|| "Saúde Sistêmica".to_string());
    let mut new_q_type = use_signal(|| "yes_no".to_string());

    let tok = token.clone();
    let cid = clinic_id.clone();

    // Carregar templates ao abrir
    let mut load_templates = {
        let t = tok.clone();
        let c = cid.clone();
        let mut t_sig = templates;
        let mut l_sig = is_loading;
        let mut err_sig = error_toast;
        move || {
            l_sig.set(true);
            let t_clone = t.clone();
            let c_clone = c.clone();
            spawn(async move {
                match fetch_anamnesis_templates(&t_clone, &c_clone).await {
                    Ok(data) => {
                        t_sig.set(data);
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("Erro ao carregar modelos: {}", e)));
                    }
                }
                l_sig.set(false);
            });
        }
    };

    use_effect(move || {
        load_templates();
    });

    let cid_for_add = cid.clone();
    let mut handle_add_question = move |_| {
        let text = new_q_text().trim().to_string();
        if text.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o texto da pergunta.".into()));
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
                clinic_id: cid_for_add.clone(),
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
    };

    let mut handle_remove_question = move |q_id: String| {
        let mut current_list = templates();
        if let Some(pos) = current_list.iter().position(|t| t.template_type == selected_type()) {
            current_list[pos].questions.retain(|q| q.id != q_id);
            templates.set(current_list);
        }
    };

    let cid_for_save = cid.clone();
    let mut handle_save_template = move |_| {
        let t_type = selected_type();
        let target = templates()
            .into_iter()
            .find(|t| t.template_type == t_type)
            .unwrap_or_else(|| AnamnesisTemplate {
                id: String::new(),
                clinic_id: cid_for_save.clone(),
                template_type: t_type.clone(),
                title: if t_type == "minor" {
                    "Ficha Padrão - Menor / Odontopediatria".into()
                } else {
                    "Ficha Padrão - Adulto".into()
                },
                questions: Vec::new(),
                created_at: String::new(),
                updated_at: String::new(),
            });

        let req = SaveAnamnesisTemplateRequest {
            clinic_id: cid_for_save.clone(),
            template_type: t_type,
            title: target.title,
            questions: target.questions,
        };

        let t = tok.clone();
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut save_sig = is_saving;

        save_sig.set(true);
        spawn(async move {
            match save_anamnesis_template(&t, req).await {
                Ok(_) => {
                    toast.set(Some("Modelo de anamnese salvo com sucesso!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao salvar modelo: {}", e)));
                }
            }
            save_sig.set(false);
        });
    };

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
            questions: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        });


    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal stock-custom-modal", style: "max-width: 820px;",
                div { class: "settings-header",
                    div {
                        h2 { class: "settings-title",
                            IconHeartPulse { size: 20, color: "#0052cc".to_string() }
                            span { " Modelos de Ficha de Anamnese" }
                        }
                        p { class: "text-muted font-xs mt-1",
                            "Personalize as perguntas padrão para novos pacientes. Alterações neste modelo não afetam retroativamente fichas já preenchidas sem consentimento."
                        }
                    }
                    button {
                        class: "close-btn",
                        onclick: move |_| {
                            let mut o = is_open;
                            o.set(false);
                        },
                        "×"
                    }
                }

                div { class: "settings-content", style: "max-height: 65vh; overflow-y: auto;",
                    // Seletor de Modelo: Adulto vs Menor de Idade
                    div { class: "documents-tab-bar", style: "margin-bottom: 20px;",
                        button {
                            class: if selected_type() == "adult" { "documents-tab-btn active" } else { "documents-tab-btn" },
                            onclick: move |_| selected_type.set("adult".to_string()),
                            "Modelo: Adulto"
                        }
                        button {
                            class: if selected_type() == "minor" { "documents-tab-btn active" } else { "documents-tab-btn" },
                            onclick: move |_| selected_type.set("minor".to_string()),
                            "Modelo: Menor de Idade / Pediátrico"
                        }
                    }

                    if is_loading() {
                        div { class: "loading-card",
                            div { class: "loading-spinner" }
                            p { "Carregando modelo de anamnese..." }
                        }
                    } else {
                        // Lista de perguntas atuais
                        div { class: "agenda-resource-box", style: "margin-bottom: 20px;",
                            div { class: "resource-section-header",
                                span { "Perguntas Configuradas ({current_template.questions.len()})" }
                            }

                            if current_template.questions.is_empty() {
                                div { class: "resource-empty-state",
                                    "Nenhuma pergunta configurada neste modelo. Adicione perguntas abaixo."
                                }
                            } else {
                                div { style: "display: flex; flex-direction: column; gap: 10px;",
                                    for q in current_template.questions.iter() {
                                        {
                                            let q_id = q.id.clone();
                                            let q_type_label = match q.question_type.as_str() {
                                                "yes_no" => "Sim / Não",
                                                "text" => "Texto Livre",
                                                _ => "Múltipla Escolha",
                                            };
                                            rsx! {
                                                div {
                                                    key: "{q.id}",
                                                    style: "display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; gap: 12px;",
                                                    div { style: "flex: 1;",
                                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                                            span { class: "badge-insurance-plan", "{q.category}" }
                                                            span { style: "font-size: 11px; color: #64748b; background: #e2e8f0; padding: 2px 6px; border-radius: 4px;", "{q_type_label}" }
                                                        }
                                                        p { style: "font-size: 13px; font-weight: 500; color: #1e293b; margin: 4px 0 0 0;", "{q.question_text}" }
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "btn-icon-danger",
                                                        onclick: move |_| handle_remove_question(q_id.clone()),
                                                        title: "Remover pergunta",
                                                        IconTrash { size: 15, color: "#ef4444".to_string() }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Adicionar Nova Pergunta
                        div { class: "agenda-resource-box",
                            div { class: "resource-section-header",
                                span { "+ Adicionar Nova Pergunta ao Modelo" }
                            }

                            div { class: "form-grid-2", style: "margin-top: 10px;",
                                div { class: "form-group full-width", style: "grid-column: 1 / -1;",
                                    label { "Texto da Pergunta *" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Apresenta alergia a algum medicamento ou substância?",
                                        value: "{new_q_text}",
                                        oninput: move |e| new_q_text.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Categoria" }
                                    select {
                                        class: "form-input",
                                        value: "{new_q_category}",
                                        onchange: move |e| new_q_category.set(e.value()),
                                        option { value: "Alergias", "Alergias" }
                                        option { value: "Saúde Sistêmica", "Saúde Sistêmica" }
                                        option { value: "Histórico Pediátrico", "Histórico Pediátrico" }
                                        option { value: "Hábitos", "Hábitos" }
                                        option { value: "Hábitos Infantis", "Hábitos Infantis" }
                                        option { value: "Medicamentos", "Medicamentos" }
                                        option { value: "Histórico Odontológico", "Histórico Odontológico" }
                                        option { value: "Queixa Principal", "Queixa Principal" }
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

                            div { style: "display: flex; justify-content: flex-end; margin-top: 12px;",
                                button {
                                    r#type: "button",
                                    class: "btn-secondary",
                                    onclick: move |e| handle_add_question(e),
                                    IconPlus { size: 14, color: "currentColor".to_string() }
                                    span { " Inserir Pergunta" }
                                }
                            }
                        }
                    }
                }

                div { class: "modal-footer-actions",
                    button {
                        r#type: "button",
                        class: "btn-secondary",
                        onclick: move |_| {
                            let mut o = is_open;
                            o.set(false);
                        },
                        "Fechar"
                    }
                    button {
                        r#type: "button",
                        class: "btn-primary",
                        disabled: is_saving(),
                        onclick: move |e| handle_save_template(e),
                        if is_saving() { "Salvando Modelo..." } else { "Salvar Alterações no Modelo" }
                    }
                }
            }
        }
    }
}
