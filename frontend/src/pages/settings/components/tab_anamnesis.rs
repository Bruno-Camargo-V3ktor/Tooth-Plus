use crate::api::anamnesis::AnamnesisApi;
use crate::components::modal::Modal;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconCheck, IconEdit, IconFileText, IconPlus, IconTrash};
use shared::anamnesis::{AnamnesisQuestion, AnamnesisTemplate, SaveAnamnesisTemplateRequest};
use dioxus::prelude::*;

#[component]
pub fn TabAnamnesis(clinic_id: String) -> Element {
    let toast = consume_context::<ToastState>();

    let mut templates = use_signal(Vec::<AnamnesisTemplate>::new);
    let mut reload_trigger = use_signal(|| 0);

    let mut show_template_modal = use_signal(|| false);
    let mut selected_template = use_signal(|| None::<AnamnesisTemplate>);

    // Campos do modelo em edição
    let mut tpl_title = use_signal(String::new);
    let mut tpl_type = use_signal(|| "adult".to_string());
    let mut tpl_questions = use_signal(Vec::<AnamnesisQuestion>::new);

    // Campos de nova pergunta
    let mut new_q_text = use_signal(String::new);
    let mut new_q_category = use_signal(|| "Histórico Médico".to_string());
    let mut new_q_type = use_signal(|| "yes_no".to_string());
    let mut new_q_required = use_signal(|| true);

    let cid_eff = clinic_id.clone();
    use_effect(move || {
        let _ = reload_trigger.read();
        let cid = cid_eff.clone();
        spawn(async move {
            if let Ok(resps) = AnamnesisApi::list_templates(&cid).await {
                templates.set(resps);
            }
        });
    });

    let handle_open_new = move |_| {
        selected_template.set(None);
        tpl_title.set(String::new());
        tpl_type.set("adult".to_string());
        tpl_questions.set(vec![
            AnamnesisQuestion {
                id: "q_1".to_string(),
                category: "Alergias".to_string(),
                question_text: "Possui alergia a medicamentos (anestésicos, penicilina)?".to_string(),
                question_type: "yes_no".to_string(),
                options: None,
                required: true,
            },
            AnamnesisQuestion {
                id: "q_2".to_string(),
                category: "Histórico Médico".to_string(),
                question_text: "Faz uso contínuo de algum medicamento?".to_string(),
                question_type: "yes_no".to_string(),
                options: None,
                required: true,
            },
        ]);
        show_template_modal.set(true);
    };



    let handle_add_question = move |_| {
        let q_txt = new_q_text.read().trim().to_string();
        if q_txt.is_empty() {
            return;
        }
        let mut list = tpl_questions.read().clone();
        let new_q = AnamnesisQuestion {
            id: format!("q_{}", list.len() + 1),
            category: new_q_category.read().clone(),
            question_text: q_txt,
            question_type: new_q_type.read().clone(),
            options: None,
            required: *new_q_required.read(),
        };
        list.push(new_q);
        tpl_questions.set(list);
        new_q_text.set(String::new());
    };

    let handle_save_template = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut reload_c = reload_trigger;
        let mut modal_c = show_template_modal;

        move |_| {
            let title_val = tpl_title.read().trim().to_string();
            if title_val.is_empty() {
                toast_c.show("Informe o título do modelo de anamnese.", ToastVariant::Error);
                return;
            }

            let type_val = tpl_type.read().clone();
            let q_list = tpl_questions.read().clone();

            let req = SaveAnamnesisTemplateRequest {
                clinic_id: cid.clone(),
                template_type: type_val,
                title: title_val,
                questions: q_list,
            };

            let mut toast_resp = toast_c.clone();
            let mut reload_resp = reload_c;
            let mut modal_resp = modal_c;

            spawn(async move {
                match AnamnesisApi::save_template(req).await {
                    Ok(_) => {
                        toast_resp.show("Modelo de anamnese salvo com sucesso!", ToastVariant::Success);
                        modal_resp.set(false);
                        reload_resp.set(reload_resp() + 1);
                    }
                    Err(err) => toast_resp.show(err, ToastVariant::Error),
                }
            });
        }
    };

    rsx! {
        div {
            // HEADER DO MÓDULO
            div { class: "settings-card", style: "padding: 16px 20px; margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center;",
                div {
                    h3 { style: "font-size: 15px; font-weight: 700; color: var(--text-main, #f8fafc); margin: 0;", "Modelos e Questionários de Anamnese" }
                    p { style: "font-size: 12.5px; color: var(--text-muted, #94a3b8); margin: 2px 0 0 0;", "Gerencie os questionários clínicos e perguntas aplicadas nas fichas de pacientes." }
                }
                button {
                    r#type: "button",
                    class: "btn-primary-blue",
                    style: "height: 38px; font-size: 13px; font-weight: 700; display: inline-flex; align-items: center; gap: 8px; padding: 0 18px;",
                    onclick: handle_open_new,
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                    span { "Novo Modelo de Anamnese" }
                }
            }

            // TABELA DE MODELOS
            div { class: "settings-table-card",
                table { class: "settings-table",
                    thead {
                        tr {
                            th { "Modelo de Anamnese" }
                            th { "Tipo de Ficha" }
                            th { "Perguntas Configuradas" }
                            th { style: "text-align: right; width: 100px;", "Ações" }
                        }
                    }
                    tbody {
                        if templates.read().is_empty() {
                            tr {
                                td { colspan: "4", style: "text-align: center; padding: 40px; color: var(--text-muted, #94a3b8);",
                                    "Nenhum modelo de anamnese cadastrado."
                                }
                            }
                        }
                        for tpl in templates() {
                            {
                                let tpl_c = tpl.clone();
                                let tpl_id = tpl.id.clone();
                                let mut toast_del = toast.clone();
                                let mut reload_del = reload_trigger;

                                rsx! {
                                    tr { key: "{tpl.id}",
                                        td {
                                            div { style: "display: flex; align-items: center; gap: 10px;",
                                                IconFileText { size: 18, color: "var(--primary, #00a0e4)".to_string() }
                                                strong { style: "font-size: 14px; color: var(--text-main, #f8fafc);", "{tpl.title}" }
                                            }
                                        }
                                        td {
                                            if tpl.template_type == "minor" {
                                                span { class: "badge badge-purple", style: "font-size: 11.5px;", "Infantil / Odontopediatria" }
                                            } else {
                                                span { class: "badge badge-blue", style: "font-size: 11.5px;", "Adulto / Geral" }
                                            }
                                        }
                                        td {
                                            span { class: "badge badge-gray", style: "font-size: 12px;",
                                                "{tpl.questions.len()} perguntas cadastradas"
                                            }
                                        }
                                        td { style: "text-align: right;",
                                            div { style: "display: inline-flex; align-items: center; gap: 8px;",
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Editar modelo e perguntas",
                                                    onclick: move |_| {
                                                        selected_template.set(Some(tpl_c.clone()));
                                                        tpl_title.set(tpl_c.title.clone());
                                                        tpl_type.set(tpl_c.template_type.clone());
                                                        tpl_questions.set(tpl_c.questions.clone());
                                                        show_template_modal.set(true);
                                                    },
                                                    IconEdit { size: 15, color: "var(--text-muted, #94a3b8)".to_string() }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Excluir modelo",
                                                    onclick: move |_| {
                                                        let tid = tpl_id.clone();
                                                        let mut t_d = toast_del.clone();
                                                        let mut r_d = reload_del;
                                                        spawn(async move {
                                                            if let Ok(_) = AnamnesisApi::delete_template(&tid).await {
                                                                t_d.show("Modelo de anamnese excluído.", ToastVariant::Success);
                                                                r_d.set(r_d() + 1);
                                                            }
                                                        });
                                                    },
                                                    IconTrash { size: 15, color: "#ef4444".to_string() }
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

            // MODAL DE EDIÇÃO E CONSTRUTOR DE PERGUNTAS DE ANAMNESE
            if show_template_modal() {
                Modal {
                    title: if selected_template().is_some() { "Editar Modelo de Anamnese & Perguntas".to_string() } else { "Criar Modelo de Anamnese & Perguntas".to_string() },
                    is_open: show_template_modal(),
                    on_close: move |_| show_template_modal.set(false),
                    footer: rsx! {
                        div { style: "display: flex; justify-content: flex-end; gap: 12px; width: 100%;",
                            button {
                                r#type: "button",
                                class: "btn-modal-ghost",
                                onclick: move |_| show_template_modal.set(false),
                                "CANCELAR"
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary-blue",
                                style: "font-weight: 700; padding: 0 24px; height: 38px;",
                                onclick: handle_save_template,
                                "SALVAR MODELO"
                            }
                        }
                    },

                    div { style: "display: flex; flex-direction: column; gap: 18px; max-height: 72vh; overflow-y: auto; padding-right: 6px;",
                        // IDENTIFICAÇÃO DO MODELO
                        div { style: "display: grid; grid-template-columns: 2fr 1fr; gap: 14px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Título do Modelo *" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: Anamnese Cirúrgica & Implante",
                                    value: "{tpl_title}",
                                    oninput: move |e| tpl_title.set(e.value()),
                                }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Perfil de Paciente" }
                                select {
                                    class: "form-select",
                                    value: "{tpl_type}",
                                    onchange: move |e| tpl_type.set(e.value()),
                                    option { value: "adult", "Adulto" }
                                    option { value: "minor", "Infantil (Pediátrico)" }
                                }
                            }
                        }

                        // CONSTRUTOR DE NOVA PERGUNTA
                        div { class: "settings-card", style: "margin: 0; padding: 16px; background: rgba(255,255,255,0.02); border: 1px solid var(--border-color, rgba(255,255,255,0.08));",
                            h4 { style: "font-size: 13.5px; font-weight: 800; color: var(--primary, #00a0e4); margin: 0 0 12px 0;", "Adicionar Nova Pergunta ao Questionário" }

                            div { style: "display: grid; grid-template-columns: 2fr 1fr 1fr; gap: 12px; margin-bottom: 10px;",
                                div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Texto da Pergunta" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Possui alergia a dipirona ou anti-inflamatórios?",
                                        value: "{new_q_text}",
                                        oninput: move |e| new_q_text.set(e.value()),
                                    }
                                }
                                div { class: "form-field", style: "margin: 0;",
                                    label { class: "form-label", "Categoria" }
                                    select {
                                        class: "form-select",
                                        value: "{new_q_category}",
                                        onchange: move |e| new_q_category.set(e.value()),
                                        option { value: "Alergias", "Alergias" }
                                        option { value: "Histórico Médico", "Histórico Médico" }
                                        option { value: "Condições Sistêmicas", "Condições Sistêmicas" }
                                        option { value: "Saúde Bucal", "Saúde Bucal" }
                                        option { value: "Hábitos", "Hábitos" }
                                        option { value: "Histórico Cirúrgico", "Histórico Cirúrgico" }
                                        option { value: "Queixa Principal", "Queixa Principal" }
                                    }
                                }
                                div { class: "form-field", style: "margin: 0;",
                                    label { class: "form-label", "Tipo de Resposta" }
                                    select {
                                        class: "form-select",
                                        value: "{new_q_type}",
                                        onchange: move |e| new_q_type.set(e.value()),
                                        option { value: "yes_no", "Sim / Não" }
                                        option { value: "text", "Texto Livre" }
                                    }
                                }
                            }

                            div { style: "display: flex; justify-content: space-between; align-items: center;",
                                label { class: "settings-checkbox-item", style: "padding: 0; background: none; border: none;",
                                    input {
                                        r#type: "checkbox",
                                        checked: *new_q_required.read(),
                                        onchange: move |e| new_q_required.set(e.value() == "true"),
                                    }
                                    span { "Resposta obrigatória" }
                                }
                                button {
                                    r#type: "button",
                                    class: "btn-primary-blue",
                                    style: "padding: 0 16px; height: 34px; font-size: 12.5px;",
                                    onclick: handle_add_question,
                                    IconPlus { size: 14, color: "#ffffff".to_string() }
                                    span { "Incluir Pergunta" }
                                }
                            }
                        }

                        // LISTA DE PERGUNTAS DO MODELO
                        div { style: "display: flex; flex-direction: column; gap: 10px;",
                            div { style: "display: flex; justify-content: space-between; align-items: center;",
                                h4 { style: "font-size: 13.5px; font-weight: 700; color: var(--text-main, #f8fafc); margin: 0;", "Perguntas Configuradas ({tpl_questions.read().len()})" }
                            }

                            for (idx, q) in tpl_questions.read().iter().enumerate() {
                                {
                                    let q_idx = idx;
                                    let q_cat = q.category.clone();
                                    let q_txt = q.question_text.clone();
                                    let q_type_lbl = if q.question_type == "yes_no" { "Sim/Não" } else { "Texto Livre" };

                                    rsx! {
                                        div { key: "{q.id}",
                                            style: "background: rgba(255,255,255,0.02); border: 1px solid var(--border-color, rgba(255,255,255,0.07)); border-radius: var(--radius-sm, 6px); padding: 12px 14px; display: flex; justify-content: space-between; align-items: center; gap: 12px;",
                                            div { style: "display: flex; align-items: center; gap: 10px; flex: 1;",
                                                span { style: "font-size: 12px; font-weight: 800; color: var(--primary, #00a0e4); width: 22px;", "#{idx + 1}" }
                                                div { style: "flex: 1;",
                                                    div { style: "font-size: 13.5px; font-weight: 600; color: var(--text-main, #f8fafc);", "{q_txt}" }
                                                    div { style: "display: flex; gap: 8px; margin-top: 2px;",
                                                        span { class: "badge badge-blue", style: "font-size: 10.5px; padding: 1px 6px;", "{q_cat}" }
                                                        span { class: "badge badge-gray", style: "font-size: 10.5px; padding: 1px 6px;", "{q_type_lbl}" }
                                                    }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "action-btn-icon",
                                                title: "Remover pergunta",
                                                onclick: move |_| {
                                                    let mut list = tpl_questions.read().clone();
                                                    if q_idx < list.len() {
                                                        list.remove(q_idx);
                                                        tpl_questions.set(list);
                                                    }
                                                },
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
