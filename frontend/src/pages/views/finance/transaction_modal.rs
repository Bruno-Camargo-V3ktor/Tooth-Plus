//! # Modal de Lançamento Financeiro (Frontend)
//!
//! Formulário para criação e edição de entradas (receitas) e saídas (despesas),
//! com seleção de categorias odontológicas padronizadas, datas rápidas, status e formas de pagamento.

use crate::api::create_transaction;
use dioxus::prelude::*;
use shared::finance::{CreateTransactionRequest, TransactionDirection, TransactionStatus};

#[component]
pub fn TransactionModal(
    is_open: Signal<bool>,
    initial_direction: TransactionDirection,
    token: String,
    clinic_id: String,
    reload_counter: Signal<i32>,
    toast_msg: Signal<Option<String>>,
) -> Element {
    if !is_open() {
        return rsx! {};
    }

    let mut form_direction = use_signal(|| match initial_direction {
        TransactionDirection::Expense => "expense".to_string(),
        _ => "income".to_string(),
    });

    let is_expense = form_direction() == "expense";
    let modal_title = if is_expense {
        "Nova Saída / Despesa"
    } else {
        "Nova Entrada Financeira"
    };

    let mut form_category = use_signal(|| {
        if initial_direction == TransactionDirection::Expense {
            "Insumos & Estoque".to_string()
        } else {
            "Procedimento Clínico".to_string()
        }
    });

    let mut form_amount = use_signal(String::new);
    let mut form_desc = use_signal(String::new);
    let mut form_due_date = use_signal(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let mut is_paid_now = use_signal(|| true);
    let mut form_payment_method = use_signal(|| "Pix".to_string());
    let mut is_submitting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_submit = move |_| {
        let clean_amount = form_amount().replace(',', ".").trim().to_string();
        let amount_cents = match clean_amount.parse::<f64>() {
            Ok(v) if v > 0.0 => (v * 100.0).round() as i64,
            _ => {
                toast_msg.set(Some("Informe um valor numérico válido maior que zero.".into()));
                return;
            }
        };

        let desc = form_desc().trim().to_string();
        if desc.is_empty() {
            toast_msg.set(Some("Informe a descrição do lançamento financeiro.".into()));
            return;
        }

        let dir = if form_direction() == "expense" {
            TransactionDirection::Expense
        } else {
            TransactionDirection::Income
        };

        let status = if is_paid_now() {
            TransactionStatus::Paid
        } else {
            TransactionStatus::Pending
        };

        let today_str = chrono::Local::now().format("%Y-%m-%d").to_string();
        let paid_date_opt = if is_paid_now() {
            Some(today_str)
        } else {
            None
        };

        let req = CreateTransactionRequest {
            clinic_id: cid.clone(),
            appointment_id: None,
            patient_id: None,
            patient_name: None,
            user_id: None,
            direction: dir,
            amount_cents,
            description: desc,
            category: form_category(),
            due_date: form_due_date(),
            paid_date: paid_date_opt,
            payment_method: Some(form_payment_method()),
            status,
            installment_current: Some(1),
            installment_total: Some(1),
        };

        let t = tok.clone();
        let mut open_sig = is_open;
        let mut rel_sig = reload_counter;
        let mut sub_sig = is_submitting;
        let mut toast = toast_msg;

        sub_sig.set(true);
        spawn(async move {
            match create_transaction(&t, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Lançamento financeiro registrado com sucesso!".into()));
                }
                Err(e) => {
                    toast.set(Some(format!("Erro ao criar lançamento: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal",
                div { class: "settings-header",
                    h2 { class: "settings-title", "{modal_title}" }
                    button { class: "close-btn", onclick: move |_| is_open.set(false), "×" }
                }

                div { class: "settings-content",
                    div { class: "form-grid",
                        // 1. Tipo de Movimentação & Categoria
                        div { class: "input-group-wrapper",
                            label { "Tipo de Movimentação" }
                            select {
                                class: "modern-input-field modern-select",
                                value: "{form_direction}",
                                onchange: move |e: FormEvent| {
                                    let val = e.value();
                                    form_direction.set(val.clone());
                                    if val == "expense" {
                                        form_category.set("Insumos & Estoque".to_string());
                                    } else {
                                        form_category.set("Procedimento Clínico".to_string());
                                    }
                                },
                                option { value: "income", "Entrada (Receita)" }
                                option { value: "expense", "Saída (Despesa)" }
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Categoria" }
                            select {
                                class: "modern-input-field modern-select",
                                value: "{form_category}",
                                onchange: move |e: FormEvent| form_category.set(e.value()),
                                if form_direction() == "income" {
                                    optgroup { label: "Receitas Clínicas & Procedimentos",
                                        option { value: "Procedimento Clínico", "Procedimento Clínico" }
                                        option { value: "Tratamento Odontológico", "Tratamento Odontológico" }
                                        option { value: "Cirurgia", "Cirurgia" }
                                        option { value: "Retorno", "Retorno" }
                                        option { value: "Avaliação / Consulta", "Avaliação / Consulta" }
                                        option { value: "Ortodontia / Mensalidade", "Ortodontia / Mensalidade" }
                                        option { value: "Prótese & Implante", "Prótese & Implante" }
                                        option { value: "Outra Receita", "Outra Receita" }
                                    }
                                    optgroup { label: "Outras Categorias",
                                        option { value: "Insumos & Estoque", "Insumos & Estoque" }
                                        option { value: "Custos Fixos / Aluguel", "Custos Fixos / Aluguel" }
                                        option { value: "Salários & Repasses", "Salários & Repasses" }
                                        option { value: "Água / Luz / Internet", "Água / Luz / Internet" }
                                        option { value: "Manutenção & Equipamentos", "Manutenção & Equipamentos" }
                                        option { value: "Marketing & Divulgação", "Marketing & Divulgação" }
                                        option { value: "Impostos & Taxas", "Impostos & Taxas" }
                                        option { value: "Outra Despesa", "Outra Despesa" }
                                    }
                                } else {
                                    optgroup { label: "Despesas Operacionais & Custos",
                                        option { value: "Insumos & Estoque", "Insumos & Estoque" }
                                        option { value: "Custos Fixos / Aluguel", "Custos Fixos / Aluguel" }
                                        option { value: "Salários & Repasses", "Salários & Repasses" }
                                        option { value: "Água / Luz / Internet", "Água / Luz / Internet" }
                                        option { value: "Manutenção & Equipamentos", "Manutenção & Equipamentos" }
                                        option { value: "Marketing & Divulgação", "Marketing & Divulgação" }
                                        option { value: "Impostos & Taxas", "Impostos & Taxas" }
                                        option { value: "Outra Despesa", "Outra Despesa" }
                                    }
                                    optgroup { label: "Outras Categorias",
                                        option { value: "Procedimento Clínico", "Procedimento Clínico" }
                                        option { value: "Tratamento Odontológico", "Tratamento Odontológico" }
                                        option { value: "Cirurgia", "Cirurgia" }
                                        option { value: "Retorno", "Retorno" }
                                        option { value: "Avaliação / Consulta", "Avaliação / Consulta" }
                                        option { value: "Ortodontia / Mensalidade", "Ortodontia / Mensalidade" }
                                        option { value: "Prótese & Implante", "Prótese & Implante" }
                                        option { value: "Outra Receita", "Outra Receita" }
                                    }
                                }
                            }
                        }

                        // 2. Descrição do Lançamento
                        div { class: "input-group-wrapper full-width",
                            label { "Descrição do Lançamento" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: Clareamento a Laser, Dental Cremer, etc.",
                                value: "{form_desc}",
                                oninput: move |e| form_desc.set(e.value())
                            }
                        }

                        // 3. Valor (R$) & Data de Vencimento
                        div { class: "input-group-wrapper",
                            label { "Valor (R$)" }
                            input {
                                class: "modern-input-field font-mono",
                                placeholder: "0,00",
                                value: "{form_amount}",
                                oninput: move |e| form_amount.set(e.value())
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Data de Vencimento" }
                            input {
                                class: "modern-input-field",
                                r#type: "date",
                                value: "{form_due_date}",
                                oninput: move |e| form_due_date.set(e.value())
                            }
                            div { class: "date-quick-buttons",
                                button {
                                    r#type: "button",
                                    class: "quick-date-btn",
                                    onclick: move |_| {
                                        form_due_date.set(chrono::Local::now().format("%Y-%m-%d").to_string());
                                    },
                                    "Hoje"
                                }
                                button {
                                    r#type: "button",
                                    class: "quick-date-btn",
                                    onclick: move |_| {
                                        let tomorrow = chrono::Local::now() + chrono::Duration::days(1);
                                        form_due_date.set(tomorrow.format("%Y-%m-%d").to_string());
                                    },
                                    "Amanhã"
                                }
                                button {
                                    r#type: "button",
                                    class: "quick-date-btn",
                                    onclick: move |_| {
                                        let next_week = chrono::Local::now() + chrono::Duration::days(7);
                                        form_due_date.set(next_week.format("%Y-%m-%d").to_string());
                                    },
                                    "+7 dias"
                                }
                                button {
                                    r#type: "button",
                                    class: "quick-date-btn",
                                    onclick: move |_| {
                                        let next_month = chrono::Local::now() + chrono::Duration::days(30);
                                        form_due_date.set(next_month.format("%Y-%m-%d").to_string());
                                    },
                                    "+30 dias"
                                }
                            }
                        }

                        // 4. Status do Pagamento (Checkbox)
                        div { class: "input-group-wrapper full-width",
                            label { "Status do Pagamento" }
                            label { class: "perm-checkbox-item",
                                input {
                                    r#type: "checkbox",
                                    checked: is_paid_now(),
                                    onchange: move |e: FormEvent| is_paid_now.set(e.checked()),
                                }
                                span { "Já foi pago / liquidado agora" }
                            }
                        }

                        // 5. Forma de Pagamento
                        div { class: "input-group-wrapper full-width",
                            label { "Forma de Pagamento Utilizada" }
                            select {
                                class: "modern-input-field modern-select",
                                value: "{form_payment_method}",
                                onchange: move |e: FormEvent| form_payment_method.set(e.value()),
                                option { value: "Pix", "Pix" }
                                option { value: "Cartão de Crédito", "Cartão de Crédito" }
                                option { value: "Cartão de Débito", "Cartão de Débito" }
                                option { value: "Dinheiro", "Dinheiro" }
                                option { value: "Boleto", "Boleto Bancário" }
                                option { value: "Transferência", "Transferência TED/DOC" }
                            }
                        }
                    }
                }

                div { class: "modal-footer-actions",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| is_open.set(false),
                        "Cancelar"
                    }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        if is_submitting() { "Salvando..." } else { "Salvar Lançamento" }
                    }
                }
            }
        }
    }
}
