use crate::components::modal::Modal;
use crate::icons::{IconEdit, IconPlus, IconTrash};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct AnamnesisTemplateItem {
    pub id: String,
    pub name: String,
}

#[component]
pub fn TabAnamnesis() -> Element {
    let mut templates = use_signal(|| vec![
        AnamnesisTemplateItem { id: "anam:1".to_string(), name: "Anamnese adulta".to_string() },
        AnamnesisTemplateItem { id: "anam:2".to_string(), name: "Anamnese adulta resumida".to_string() },
        AnamnesisTemplateItem { id: "anam:3".to_string(), name: "Anamnese HOF".to_string() },
        AnamnesisTemplateItem { id: "anam:4".to_string(), name: "Anamnese infantil".to_string() },
        AnamnesisTemplateItem { id: "anam:5".to_string(), name: "Anamnese infantil resumida".to_string() },
        AnamnesisTemplateItem { id: "anam:6".to_string(), name: "Anamnese ortodôntica".to_string() },
        AnamnesisTemplateItem { id: "anam:7".to_string(), name: "Anamnese ortodôntica resumida".to_string() },
    ]);

    let mut show_modal = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let mut model_name = use_signal(String::new);

    let handle_open_new = move |_| {
        editing_id.set(None);
        model_name.set(String::new());
        show_modal.set(true);
    };

    let handle_save = move |_| {
        let name = model_name.read().trim().to_string();
        if name.is_empty() {
            return;
        }
        let mut list = templates.read().clone();

        if let Some(ref edit_id) = *editing_id.read() {
            if let Some(item) = list.iter_mut().find(|a| a.id == *edit_id) {
                item.name = name;
            }
        } else {
            let new_item = AnamnesisTemplateItem {
                id: format!("anam:{}", list.len() + 1),
                name,
            };
            list.push(new_item);
        }
        templates.set(list);
        show_modal.set(false);
    };

    rsx! {
        div {
            div { class: "settings-list-header",
                h2 { class: "settings-list-title", "Anamnese" }
                button {
                    r#type: "button",
                    class: "settings-btn-new-green",
                    onclick: handle_open_new,
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                    span { "NOVO MODELO" }
                }
            }

            div { class: "settings-table-card",
                table { class: "settings-table",
                    thead {
                        tr {
                            th { style: "display: flex; align-items: center; gap: 4px;",
                                span { "Nome" }
                                span { style: "font-size: 11px;", "↑" }
                            }
                            th { style: "text-align: right; width: 120px;", "Ações" }
                        }
                    }
                    tbody {
                        for tpl in templates() {
                            {
                                let tid = tpl.id.clone();
                                let tid_del = tpl.id.clone();
                                let tname = tpl.name.clone();

                                rsx! {
                                    tr { key: "{tpl.id}",
                                        td {
                                            span { style: "font-weight: 600; color: #f1f5f9; font-size: 13.5px;", "{tpl.name}" }
                                        }
                                        td { style: "text-align: right;",
                                            div { style: "display: inline-flex; align-items: center; gap: 8px;",
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Editar modelo",
                                                    onclick: move |_| {
                                                        editing_id.set(Some(tid.clone()));
                                                        model_name.set(tname.clone());
                                                        show_modal.set(true);
                                                    },
                                                    IconEdit { size: 14, color: "#94a3b8".to_string() }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Excluir modelo",
                                                    onclick: move |_| {
                                                        let mut list = templates.read().clone();
                                                        list.retain(|item| item.id != tid_del);
                                                        templates.set(list);
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

            div { class: "settings-help-footer",
                span { "🎓" }
                span { "Dúvidas? Saiba tudo sobre " }
                a { href: "#", "Anamnese" }
            }

            if show_modal() {
                Modal {
                    title: if editing_id().is_some() { "Editar Modelo de Anamnese".to_string() } else { "Novo Modelo de Anamnese".to_string() },
                    is_open: show_modal(),
                    on_close: move |_| show_modal.set(false),
                    footer: rsx! {
                        div { style: "display: flex; justify-content: flex-end; gap: 10px; width: 100%;",
                            button {
                                r#type: "button",
                                class: "btn-modal-ghost",
                                onclick: move |_| show_modal.set(false),
                                "CANCELAR"
                            }
                            button {
                                r#type: "button",
                                class: "settings-btn-save",
                                onclick: handle_save,
                                "SALVAR MODELO"
                            }
                        }
                    },
                    div { style: "display: flex; flex-direction: column; gap: 14px;",
                        div { class: "form-field",
                            label { class: "form-label", "Nome do modelo de anamnese *" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Anamnese Cirúrgica, Anamnese Endodôntica...",
                                value: "{model_name}",
                                oninput: move |e| model_name.set(e.value()),
                            }
                        }
                    }
                }
            }
        }
    }
}
