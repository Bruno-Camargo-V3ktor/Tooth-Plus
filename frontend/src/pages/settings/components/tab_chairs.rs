use crate::components::modal::Modal;
use crate::icons::{IconEdit, IconPlus, IconTrash};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ChairItem {
    pub id: String,
    pub name: String,
    pub room: String,
    pub is_active: bool,
}

#[component]
pub fn TabChairs() -> Element {
    let mut chairs = use_signal(|| vec![
        ChairItem { id: "chair:1".to_string(), name: "Consultório 1 - Ortodontia".to_string(), room: "Sala 101".to_string(), is_active: true },
        ChairItem { id: "chair:2".to_string(), name: "Consultório 2 - Cirurgia & Implante".to_string(), room: "Sala 102".to_string(), is_active: true },
        ChairItem { id: "chair:3".to_string(), name: "Consultório 3 - Clínica Geral".to_string(), room: "Sala 103".to_string(), is_active: true },
    ]);

    let mut show_modal = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let mut chair_name = use_signal(String::new);
    let mut chair_room = use_signal(String::new);

    let handle_open_new = move |_| {
        editing_id.set(None);
        chair_name.set(String::new());
        chair_room.set(String::new());
        show_modal.set(true);
    };

    let handle_save = move |_| {
        let name = chair_name.read().trim().to_string();
        if name.is_empty() {
            return;
        }
        let room = chair_room.read().trim().to_string();
        let mut list = chairs.read().clone();

        if let Some(ref edit_id) = *editing_id.read() {
            if let Some(item) = list.iter_mut().find(|c| c.id == *edit_id) {
                item.name = name;
                item.room = room;
            }
        } else {
            let new_item = ChairItem {
                id: format!("chair:{}", list.len() + 1),
                name,
                room,
                is_active: true,
            };
            list.push(new_item);
        }
        chairs.set(list);
        show_modal.set(false);
    };

    rsx! {
        div {
            div { class: "settings-card", style: "padding: 16px 20px; margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center;",
                div {
                    h3 { style: "font-size: 15px; font-weight: 700; color: var(--text-main, #f8fafc); margin: 0;", "Cadeiras e Salas de Atendimento" }
                    p { style: "font-size: 12.5px; color: var(--text-muted, #94a3b8); margin: 2px 0 0 0;", "Organize os consultórios e equipamentos disponíveis na clínica." }
                }
                button {
                    r#type: "button",
                    class: "btn-primary-blue",
                    style: "height: 38px; font-size: 13px; font-weight: 700; display: inline-flex; align-items: center; gap: 8px; padding: 0 18px;",
                    onclick: handle_open_new,
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                    span { "Nova Cadeira" }
                }
            }

            div { class: "settings-table-card",
                table { class: "settings-table",
                    thead {
                        tr {
                            th { "Identificação da Cadeira / Sala" }
                            th { "Localização / Sala" }
                            th { "Status" }
                            th { style: "text-align: right; width: 100px;", "Ações" }
                        }
                    }
                    tbody {
                        for c in chairs() {
                            {
                                let cid = c.id.clone();
                                let cid_del = c.id.clone();
                                let cname = c.name.clone();
                                let croom = c.room.clone();

                                rsx! {
                                    tr { key: "{c.id}",
                                        td {
                                            strong { style: "font-size: 14px; color: var(--text-main, #f8fafc);", "{c.name}" }
                                        }
                                        td {
                                            span { style: "color: var(--text-muted, #94a3b8); font-size: 13px;", "{c.room}" }
                                        }
                                        td {
                                            span { class: "badge badge-green", style: "font-size: 11px;", "Ativa" }
                                        }
                                        td { style: "text-align: right;",
                                            div { style: "display: inline-flex; align-items: center; gap: 8px;",
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Editar cadeira",
                                                    onclick: move |_| {
                                                        editing_id.set(Some(cid.clone()));
                                                        chair_name.set(cname.clone());
                                                        chair_room.set(croom.clone());
                                                        show_modal.set(true);
                                                    },
                                                    IconEdit { size: 15, color: "var(--text-muted, #94a3b8)".to_string() }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Excluir cadeira",
                                                    onclick: move |_| {
                                                        let mut list = chairs.read().clone();
                                                        list.retain(|item| item.id != cid_del);
                                                        chairs.set(list);
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

            if show_modal() {
                Modal {
                    title: if editing_id().is_some() { "Editar Cadeira Clínica".to_string() } else { "Cadastrar Nova Cadeira Clínica".to_string() },
                    is_open: show_modal(),
                    on_close: move |_| show_modal.set(false),
                    footer: rsx! {
                        div { style: "display: flex; justify-content: flex-end; gap: 12px; width: 100%;",
                            button {
                                r#type: "button",
                                class: "btn-modal-ghost",
                                onclick: move |_| show_modal.set(false),
                                "CANCELAR"
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary-blue",
                                style: "font-weight: 700; padding: 0 24px; height: 38px;",
                                onclick: handle_save,
                                "SALVAR CADEIRA"
                            }
                        }
                    },
                    div { style: "display: flex; flex-direction: column; gap: 14px;",
                        div { class: "form-field",
                            label { class: "form-label", "Nome / Identificação da Cadeira *" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Consultório 1 - Ortodontia",
                                value: "{chair_name}",
                                oninput: move |e| chair_name.set(e.value()),
                            }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Sala / Andar" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Sala 101, 1º Andar",
                                value: "{chair_room}",
                                oninput: move |e| chair_room.set(e.value()),
                            }
                        }
                    }
                }
            }
        }
    }
}
