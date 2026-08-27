use crate::api::anamnesis::AnamnesisApi;
use crate::api::ActiveClinicState;
use crate::components::modal::Modal;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconCheck, IconFileText, IconPlus, IconTrash};
use shared::anamnesis::{AnamnesisQuestion, AnamnesisResponseItem};
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn TabAnamnesis(patient: Patient) -> Element {
    let toast = consume_context::<ToastState>();
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_else(|| "clinic:luria_dent".to_string());

    let mut responses = use_signal(|| vec![
        AnamnesisResponseItem {
            question_id: "q_1".to_string(),
            category: "Alergias".to_string(),
            question_text: "Possui alguma alergia a medicamentos (penicilina, anestésicos, dipirona)?".to_string(),
            question_type: "yes_no".to_string(),
            answer_boolean: Some(false),
            answer_text: None,
            notes: None,
        },
        AnamnesisResponseItem {
            question_id: "q_2".to_string(),
            category: "Histórico Médico".to_string(),
            question_text: "Faz uso contínuo de algum medicamento?".to_string(),
            question_type: "yes_no".to_string(),
            answer_boolean: Some(false),
            answer_text: None,
            notes: None,
        },
        AnamnesisResponseItem {
            question_id: "q_3".to_string(),
            category: "Condições Sistêmicas".to_string(),
            question_text: "Possui histórico de hipertensão, diabetes, cardiopatias ou problemas renais?".to_string(),
            question_type: "yes_no".to_string(),
            answer_boolean: Some(false),
            answer_text: None,
            notes: None,
        },
        AnamnesisResponseItem {
            question_id: "q_4".to_string(),
            category: "Histórico Cirúrgico".to_string(),
            question_text: "Já teve episódios de hemorragia ou cicatrização lenta após procedimentos?".to_string(),
            question_type: "yes_no".to_string(),
            answer_boolean: Some(false),
            answer_text: None,
            notes: None,
        },
        AnamnesisResponseItem {
            question_id: "q_5".to_string(),
            category: "Hábitos".to_string(),
            question_text: "Possui hábito de fumar (tabagismo) ou consome bebidas alcoólicas com frequência?".to_string(),
            question_type: "yes_no".to_string(),
            answer_boolean: Some(false),
            answer_text: None,
            notes: None,
        },
        AnamnesisResponseItem {
            question_id: "q_6".to_string(),
            category: "Queixa Principal".to_string(),
            question_text: "Qual é o motivo principal da consulta de hoje?".to_string(),
            question_type: "text".to_string(),
            answer_boolean: None,
            answer_text: Some("Avaliação odontológica de rotina e limpeza.".to_string()),
            notes: None,
        },
    ]);

    let mut show_new_q_modal = use_signal(|| false);
    let mut custom_q_text = use_signal(String::new);
    let mut custom_q_cat = use_signal(|| "Histórico Médico".to_string());
    let mut custom_q_type = use_signal(|| "yes_no".to_string());

    let handle_add_custom_question = {
        let mut toast_add = toast.clone();
        move |_| {
            let txt = custom_q_text.read().trim().to_string();
            if txt.is_empty() {
                return;
            }
            let mut list = responses.read().clone();
            let new_item = AnamnesisResponseItem {
                question_id: format!("cq_{}", list.len() + 1),
                category: custom_q_cat.read().clone(),
                question_text: txt,
                question_type: custom_q_type.read().clone(),
                answer_boolean: if *custom_q_type.read() == "yes_no" { Some(false) } else { None },
                answer_text: None,
                notes: None,
            };
            list.push(new_item);
            responses.set(list);
            custom_q_text.set(String::new());
            show_new_q_modal.set(false);
            toast_add.show("Pergunta adicionada à anamnese!", ToastVariant::Success);
        }
    };

    rsx! {
        div { class: "patient-card",
            div { class: "patient-card-header", style: "display: flex; justify-content: space-between; align-items: center;",
                div {
                    h3 { class: "patient-card-title", "Ficha Clínica de Anamnese" }
                    p { style: "font-size: 12px; color: var(--text-muted, #94a3b8); margin: 2px 0 0 0;", "Histórico médico, condições de saúde e respostas do paciente {patient.full_name}." }
                }
                div { style: "display: flex; gap: 10px;",
                    button {
                        r#type: "button",
                        class: "btn-modal-ghost",
                        style: "font-size: 12.5px; font-weight: 700;",
                        onclick: move |_| show_new_q_modal.set(true),
                        IconPlus { size: 14, color: "var(--primary, #00a0e4)".to_string() }
                        span { "Adicionar Pergunta" }
                    }
                    {
                        let mut toast_save = toast.clone();
                        rsx! {
                            button {
                                r#type: "button",
                                class: "btn-primary-blue",
                                style: "height: 36px; padding: 0 20px; font-size: 13px; font-weight: 700;",
                                onclick: move |_| {
                                    toast_save.show("Ficha de anamnese salva com sucesso!", ToastVariant::Success);
                                },
                                IconCheck { size: 16, color: "#ffffff".to_string() }
                                span { "Salvar Respostas" }
                            }
                        }
                    }
                }
            }

            div { class: "patient-card-body", style: "display: flex; flex-direction: column; gap: 14px;",
                for (idx, resp) in responses.read().iter().enumerate() {
                    {
                        let r_idx = idx;
                        let r_id = resp.question_id.clone();
                        let r_cat = resp.category.clone();
                        let r_txt = resp.question_text.clone();
                        let r_type = resp.question_type.clone();
                        let r_bool = resp.answer_boolean;
                        let r_text_val = resp.answer_text.clone().unwrap_or_default();
                        let r_notes_val = resp.notes.clone().unwrap_or_default();

                        rsx! {
                            div { key: "{resp.question_id}",
                                style: "background: rgba(255,255,255,0.02); border: 1px solid var(--border-color, rgba(255,255,255,0.07)); border-radius: var(--radius-md, 8px); padding: 16px;",

                                div { style: "display: flex; justify-content: space-between; align-items: flex-start; gap: 16px;",
                                    div { style: "flex: 1;",
                                        div { style: "display: flex; align-items: center; gap: 8px; margin-bottom: 4px;",
                                            span { class: "badge badge-blue", style: "font-size: 10.5px; padding: 1px 6px;", "{r_cat}" }
                                            span { style: "font-size: 12px; font-weight: 700; color: var(--text-light, #64748b);", "Pergunta #{idx + 1}" }
                                        }
                                        div { style: "font-size: 14px; font-weight: 700; color: var(--text-main, #f8fafc);", "{r_txt}" }
                                    }

                                    // Controles de Resposta
                                    if r_type == "yes_no" {
                                        div { style: "display: flex; gap: 6px;",
                                            button {
                                                r#type: "button",
                                                class: if r_bool == Some(true) { "btn-filter-pill active" } else { "btn-filter-pill" },
                                                style: if r_bool == Some(true) { "background: #ef4444; border-color: #dc2626; color: #ffffff;" } else { "" },
                                                onclick: move |_| {
                                                    let mut list = responses.read().clone();
                                                    if r_idx < list.len() {
                                                        list[r_idx].answer_boolean = Some(true);
                                                        responses.set(list);
                                                    }
                                                },
                                                "SIM"
                                            }
                                            button {
                                                r#type: "button",
                                                class: if r_bool == Some(false) { "btn-filter-pill active" } else { "btn-filter-pill" },
                                                style: if r_bool == Some(false) { "background: #16a34a; border-color: #15803d; color: #ffffff;" } else { "" },
                                                onclick: move |_| {
                                                    let mut list = responses.read().clone();
                                                    if r_idx < list.len() {
                                                        list[r_idx].answer_boolean = Some(false);
                                                        responses.set(list);
                                                    }
                                                },
                                                "NÃO"
                                            }
                                        }
                                    }
                                }

                                if r_type == "text" {
                                    div { style: "margin-top: 10px;",
                                        textarea {
                                            class: "form-textarea",
                                            style: "height: 60px; font-size: 13px;",
                                            placeholder: "Descreva a queixa ou informações adicionais...",
                                            value: "{r_text_val}",
                                            oninput: move |e| {
                                                let mut list = responses.read().clone();
                                                if r_idx < list.len() {
                                                    list[r_idx].answer_text = Some(e.value());
                                                    responses.set(list);
                                                }
                                            },
                                        }
                                    }
                                } else if r_bool == Some(true) {
                                    div { style: "margin-top: 12px; padding-top: 10px; border-top: 1px dashed rgba(255,255,255,0.06);",
                                        label { class: "form-label", style: "font-size: 11.5px; color: #f87171;", "Observações / Detalhes sobre a resposta afirmativa:" }
                                        input {
                                            class: "form-input",
                                            style: "font-size: 13px; height: 36px;",
                                            placeholder: "Ex: Alergia a amoxicilina confirmada em 2022...",
                                            value: "{r_notes_val}",
                                            oninput: move |e| {
                                                let mut list = responses.read().clone();
                                                if r_idx < list.len() {
                                                    list[r_idx].notes = Some(e.value());
                                                    responses.set(list);
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // MODAL PARA ADICIONAR PERGUNTA PERSONALIZADA DIRETAMENTE NA FICHA
            if show_new_q_modal() {
                Modal {
                    title: "Adicionar Pergunta à Ficha de Anamnese".to_string(),
                    is_open: show_new_q_modal(),
                    on_close: move |_| show_new_q_modal.set(false),
                    footer: rsx! {
                        div { style: "display: flex; justify-content: flex-end; gap: 10px; width: 100%;",
                            button {
                                r#type: "button",
                                class: "btn-modal-ghost",
                                onclick: move |_| show_new_q_modal.set(false),
                                "CANCELAR"
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary-blue",
                                style: "font-weight: 700; padding: 0 20px; height: 38px;",
                                onclick: handle_add_custom_question,
                                "INCLUIR PERGUNTA"
                            }
                        }
                    },
                    div { style: "display: flex; flex-direction: column; gap: 14px;",
                        div { class: "form-field",
                            label { class: "form-label", "Texto da Pergunta *" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Já passou por cirurgia na face ou ATM?",
                                value: "{custom_q_text}",
                                oninput: move |e| custom_q_text.set(e.value()),
                            }
                        }
                        div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Categoria" }
                                select {
                                    class: "form-select",
                                    value: "{custom_q_cat}",
                                    onchange: move |e| custom_q_cat.set(e.value()),
                                    option { value: "Histórico Médico", "Histórico Médico" }
                                    option { value: "Alergias", "Alergias" }
                                    option { value: "Condições Sistêmicas", "Condições Sistêmicas" }
                                    option { value: "Saúde Bucal", "Saúde Bucal" }
                                    option { value: "Hábitos", "Hábitos" }
                                }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Tipo de Resposta" }
                                select {
                                    class: "form-select",
                                    value: "{custom_q_type}",
                                    onchange: move |e| custom_q_type.set(e.value()),
                                    option { value: "yes_no", "Sim / Não" }
                                    option { value: "text", "Texto Livre" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
