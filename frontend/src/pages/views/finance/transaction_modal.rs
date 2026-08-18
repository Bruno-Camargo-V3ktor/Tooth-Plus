//! # Modal de Lançamento de Transações Financeiras (Frontend)
//!
//! Controla os lançamentos de receitas de consultas/procedimentos,
//! despesas operacionais da clínica, comissões de dentistas e formas de pagamento.

use crate::api::create_transaction;
use crate::components::icons::IconCheck;
use dioxus::prelude::*;
use shared::finance::{CreateTransactionRequest, TransactionDirection, TransactionStatus};

/// Modal para inserção de receitas ou despesas no fluxo financeiro da clínica.
#[component]
pub fn TransactionModal(
    token: String,
    clinic_id: String,
    initial_direction: TransactionDirection,
    is_open: Signal<bool>,
    reload_counter: Signal<usize>,
    toast_msg: Signal<Option<String>>,
) -> Element {
    if !is_open() {
        return rsx! {};
    }

    let mut form_direction = use_signal(|| match initial_direction {
        TransactionDirection::Expense => "expense".to_string(),
        _ => "income".to_string(),
    });
    let mut form_category = use_signal(|| "consultation".to_string());
    let mut form_amount = use_signal(|| "0,00".to_string());
    let mut form_desc = use_signal(String::new);
    let mut form_payment_method = use_signal(|| "Pix".to_string());
    let mut form_status = use_signal(|| "paid".to_string());
    let mut form_due_date = use_signal(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let mut is_submitting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_submit = move |_| {
        let desc = form_desc().trim().to_string();
        if desc.is_empty() {
            let mut toast = toast_msg;
            toast.set(Some("Informe a descrição do lançamento financeiro.".into()));
            return;
        }

        let amount_clean = form_amount().replace("R$", "").replace(".", "").replace(",", "").trim().to_string();
        let amount_cents = amount_clean.parse::<i64>().unwrap_or(0);
        if amount_cents <= 0 {
            let mut toast = toast_msg;
            toast.set(Some("O valor deve ser maior que zero.".into()));
            return;
        }

        let dir = match form_direction().as_str() {
            "expense" => TransactionDirection::Expense,
            _ => TransactionDirection::Income,
        };

        let status = match form_status().as_str() {
            "pending" => TransactionStatus::Pending,
            "canceled" => TransactionStatus::Canceled,
            _ => TransactionStatus::Paid,
        };

        let today_str = chrono::Local::now().format("%Y-%m-%d").to_string();
        let paid_date_opt = if status == TransactionStatus::Paid {
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
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", "Novo Lançamento Financeiro" }
                        p { class: "modal-subtitle", "Registre receitas de consultas ou despesas operacionais da clínica." }
                    }
                    button { class: "modal-close", onclick: move |_| { let mut o = is_open; o.set(false); }, "×" }
                }
                div { class: "modal-body",
                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Tipo de Movimentação *" }
                            select {
                                class: "form-input",
                                value: "{form_direction}",
                                onchange: move |e| form_direction.set(e.value()),
                                option { value: "income", "Receita (Entrada +)" }
                                option { value: "expense", "Despesa (Saída -)" }
                            }
                        }
                        div { class: "form-group",
                            label { "Categoria" }
                            select {
                                class: "form-input",
                                value: "{form_category}",
                                onchange: move |e| form_category.set(e.value()),
                                if form_direction() == "income" {
                                    option { value: "consultation", "Receita de Atendimento" }
                                    option { value: "procedure", "Procedimento / Cirurgia" }
                                    option { value: "other_income", "Outra Receita" }
                                } else {
                                    option { value: "commission", "Comissão de Dentista" }
                                    option { value: "supplies", "Compra de Materiais" }
                                    option { value: "rent", "Aluguel / Condomínio" }
                                    option { value: "utilities", "Água / Luz / Internet" }
                                    option { value: "other_expense", "Outra Despesa" }
                                }
                            }
                        }
                    }

                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Valor (R$) *" }
                            input {
                                class: "form-input font-mono",
                                placeholder: "0,00",
                                value: "{form_amount}",
                                oninput: move |e| form_amount.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Forma de Pagamento" }
                            select {
                                class: "form-input",
                                value: "{form_payment_method}",
                                onchange: move |e| form_payment_method.set(e.value()),
                                option { value: "Pix", "Pix" }
                                option { value: "Cartão de Crédito", "Cartão de Crédito" }
                                option { value: "Cartão de Débito", "Cartão de Débito" }
                                option { value: "Dinheiro", "Dinheiro em Espécie" }
                                option { value: "Boleto", "Boleto Bancário" }
                                option { value: "Transferência", "Transferência TED/DOC" }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Descrição do Lançamento *" }
                        input {
                            class: "form-input",
                            placeholder: "Ex: Pagamento do tratamento ortodôntico - Paciente João",
                            value: "{form_desc}",
                            oninput: move |e| form_desc.set(e.value())
                        }
                    }

                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Status do Lançamento" }
                            select {
                                class: "form-input",
                                value: "{form_status}",
                                onchange: move |e| form_status.set(e.value()),
                                option { value: "paid", "Liquidado / Pago Agora" }
                                option { value: "pending", "Pendente / A Receber / A Pagar" }
                            }
                        }
                        div { class: "form-group",
                            label { "Data de Vencimento / Competência *" }
                            input {
                                class: "form-input",
                                r#type: "date",
                                value: "{form_due_date}",
                                oninput: move |e| form_due_date.set(e.value())
                            }
                        }
                    }
                }
                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| { let mut o = is_open; o.set(false); }, "Cancelar" }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        IconCheck { size: 16, color: "currentColor".to_string() }
                        span { if is_submitting() { "Salvando..." } else { "Salvar Lançamento" } }
                    }
                }
            }
        }
    }
}
