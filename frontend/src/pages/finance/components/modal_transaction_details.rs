use crate::components::modal::Modal;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconFileText, IconTrash};
use shared::finance::Transaction;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Copy)]
pub enum DetailModalTab {
    Edit,
    Documents,
}

#[component]
pub fn ModalTransactionDetails(
    is_open: bool,
    transaction: Option<Transaction>,
    on_close: EventHandler<()>,
    on_save: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let tx = match transaction {
        Some(t) => t,
        None => return rsx! {},
    };

    let mut active_tab = use_signal(|| DetailModalTab::Documents);
    let mut desc = use_signal(|| tx.description.clone());
    let mut amount_str = use_signal(|| format!("{:.2}", tx.amount_cents as f64 / 100.0));
    let mut category = use_signal(|| tx.category.clone());
    let mut due_date = use_signal(|| tx.due_date.split('T').next().unwrap_or(&tx.due_date).to_string());
    let toast = consume_context::<ToastState>();

    let receipt_name_display = tx.receipt_name.clone().unwrap_or_else(|| format!("recibo_{}", tx.description.replace(' ', "_")));
    let receipt_date_display = tx.receipt_date.clone().unwrap_or_else(|| "24/08/2026".to_string());

    let mut toast_del = toast.clone();
    let mut toast_att = toast.clone();

    rsx! {
        Modal {
            title: "".to_string(),
            is_open,
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                button {
                    r#type: "button",
                    class: "btn-modal-ghost",
                    onclick: move |_| on_close.call(()),
                    "FECHAR"
                }
                button {
                    r#type: "button",
                    class: "btn-new-patient-green",
                    onclick: move |_| on_save.call(()),
                    "SALVAR"
                }
            },

            div { style: "display: flex; flex-direction: column; gap: 16px;",
                // Barra de Abas do Modal
                div { class: "tab-underline-bar", style: "margin: 0; padding-bottom: 8px;",
                    button {
                        class: if *active_tab.read() == DetailModalTab::Edit { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                        onclick: move |_| active_tab.set(DetailModalTab::Edit),
                        "EDITAR RECEBIMENTO"
                    }
                    button {
                        class: if *active_tab.read() == DetailModalTab::Documents { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                        onclick: move |_| active_tab.set(DetailModalTab::Documents),
                        "DOCUMENTOS"
                    }
                }

                if *active_tab.read() == DetailModalTab::Edit {
                    div { style: "display: flex; flex-direction: column; gap: 12px;",
                        div { class: "form-field",
                            label { class: "form-label", "Descrição / Paciente *" }
                            input {
                                class: "form-input",
                                r#type: "text",
                                value: "{desc}",
                                oninput: move |e| desc.set(e.value()),
                            }
                        }
                        div { class: "form-row-2 form-row",
                            div { class: "form-field",
                                label { class: "form-label", "Valor Total (R$) *" }
                                input {
                                    class: "form-input",
                                    r#type: "number",
                                    step: "0.01",
                                    value: "{amount_str}",
                                    oninput: move |e| amount_str.set(e.value()),
                                }
                            }
                            div { class: "form-field",
                                label { class: "form-label", "Categoria" }
                                input {
                                    class: "form-input",
                                    r#type: "text",
                                    value: "{category}",
                                    oninput: move |e| category.set(e.value()),
                                }
                            }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Data de Vencimento" }
                            input {
                                class: "form-input",
                                r#type: "date",
                                value: "{due_date}",
                                oninput: move |e| due_date.set(e.value()),
                            }
                        }
                    }
                } else {
                    div { style: "display: flex; flex-direction: column; gap: 16px;",
                        // Card do Recibo Anexado
                        div { style: "background: #0b1120; border: 1px solid rgba(255,255,255,0.08); padding: 12px 16px; border-radius: 8px; display: flex; align-items: center; justify-content: space-between;",
                            div { style: "display: flex; align-items: center; gap: 12px;",
                                IconFileText { size: 24, color: "#38bdf8".to_string() }
                                div {
                                    strong { style: "font-size: 13.5px; color: #38bdf8; display: block;", "{receipt_name_display}" }
                                    span { style: "font-size: 12px; color: #94a3b8;", "Enviado em {receipt_date_display}" }
                                }
                            }

                            div { style: "display: flex; align-items: center; gap: 8px;",
                                button {
                                    r#type: "button",
                                    class: "action-btn-icon",
                                    title: "Baixar Comprovante",
                                    onclick: move |_| {
                                        let _ = web_sys::window().map(|w| w.print());
                                    },
                                    "⬇"
                                }
                                button {
                                    r#type: "button",
                                    class: "action-btn-icon",
                                    title: "Excluir Comprovante",
                                    onclick: move |_| {
                                        toast_del.show("Comprovante desvinculado.", ToastVariant::Info);
                                    },
                                    IconTrash { size: 14, color: "#ef4444".to_string() }
                                }
                            }
                        }

                        div { style: "display: flex; justify-content: center; padding: 12px 0;",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                style: "color: #38bdf8; border-color: rgba(56, 189, 248, 0.3); font-weight: 700;",
                                onclick: move |_| {
                                    toast_att.show("Selecione o arquivo de comprovante.", ToastVariant::Info);
                                },
                                "ANEXAR COMPROVANTE"
                            }
                        }
                    }
                }
            }
        }
    }
}
