//! # Modal de Transição de Status e Baixa de Insumos (Frontend)
//!
//! Controla as mudanças de status da consulta (Confirmar, Iniciar Atendimento, Concluir
//! ou Cancelar), permitindo confirmar quantidades reais de materiais consumidos
//! antes de disparar o lançamento financeiro e a dedução no estoque.

use crate::api::update_appointment_status;
use crate::components::icons::IconCheck;
use dioxus::prelude::*;
use shared::appointments::{
    AppointmentResponse, AppointmentStatus, ConsumedItemDto, UpdateAppointmentStatusRequest,
};

/// Modal para alteração do status da consulta com confirmação de materiais consumidos.
#[component]
pub fn AppointmentStatusModal(
    token: String,
    clinic_id: String,
    appointment: Option<AppointmentResponse>,
    is_open: Signal<bool>,
    on_success: EventHandler<String>,
    toast_msg: Signal<Option<String>>,
) -> Element {
    let Some(ref app) = appointment else {
        return rsx! {};
    };

    if !is_open() {
        return rsx! {};
    }

    let mut selected_status = use_signal(|| match app.status {
        AppointmentStatus::Pending => "confirmed".to_string(),
        AppointmentStatus::Confirmed => "in_progress".to_string(),
        AppointmentStatus::InProgress => "completed".to_string(),
        _ => "completed".to_string(),
    });
    let mut cancellation_reason = use_signal(String::new);
    let mut consumed_items = use_signal(|| app.consumed_items.clone());
    let mut is_submitting = use_signal(|| false);

    let app_id = app.id.clone();
    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_submit = move |_| {
        let status_enum = match selected_status().as_str() {
            "confirmed" => AppointmentStatus::Confirmed,
            "in_progress" => AppointmentStatus::InProgress,
            "completed" => AppointmentStatus::Completed,
            "canceled" => AppointmentStatus::Canceled,
            "no_show" => AppointmentStatus::NoShow,
            _ => AppointmentStatus::Pending,
        };

        let reason_opt = if status_enum == AppointmentStatus::Canceled {
            let r = cancellation_reason().trim().to_string();
            if r.is_empty() {
                None
            } else {
                Some(r)
            }
        } else {
            None
        };

        let items_opt = if status_enum == AppointmentStatus::Completed {
            Some(consumed_items())
        } else {
            None
        };

        let req = UpdateAppointmentStatusRequest {
            status: status_enum,
            cancellation_reason: reason_opt,
            consumed_items: items_opt,
        };

        let t = tok.clone();
        let c = cid.clone();
        let a_id = app_id.clone();
        let mut open_sig = is_open;
        let mut sub_sig = is_submitting;
        let mut toast = toast_msg;
        let on_succ = on_success.clone();

        sub_sig.set(true);
        spawn(async move {
            match update_appointment_status(&t, &a_id, &c, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    toast.set(Some("Status do agendamento atualizado com sucesso!".into()));
                    on_succ.call("Status atualizado".into());
                }
                Err(e) => {
                    toast.set(Some(format!("Erro ao atualizar status: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal modal-small",
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", "Atualizar Status do Atendimento" }
                        p { class: "modal-subtitle", "Avance as etapas do atendimento odontológico e confirme os insumos utilizados." }
                    }
                    button { class: "modal-close", onclick: move |_| { let mut o = is_open; o.set(false); }, "×" }
                }
                div { class: "modal-body",
                    div { class: "appointment-summary-box mb-4",
                        h4 { class: "font-weight-bold mb-1", "{app.title}" }
                        p { class: "text-muted font-xs",
                            "Paciente: {app.patient_name.as_deref().unwrap_or(\"Não informado\")}"
                        }
                    }

                    div { class: "form-group",
                        label { "Novo Status *" }
                        select {
                            class: "form-input",
                            value: "{selected_status}",
                            onchange: move |e| selected_status.set(e.value()),
                            option { value: "confirmed", "Confirmado" }
                            option { value: "in_progress", "Em Atendimento" }
                            option { value: "completed", "Concluído (Baixa no Estoque e Financeiro)" }
                            option { value: "canceled", "Cancelado" }
                            option { value: "no_show", "Não Compareceu" }
                        }
                    }

                    if selected_status() == "canceled" {
                        div { class: "form-group",
                            label { "Motivo do Cancelamento" }
                            textarea {
                                class: "form-textarea",
                                placeholder: "Ex: Paciente solicitou reagendamento por motivos pessoais...",
                                value: "{cancellation_reason}",
                                oninput: move |e| cancellation_reason.set(e.value())
                            }
                        }
                    }

                    if selected_status() == "completed" && !consumed_items().is_empty() {
                        div { class: "form-section-title mt-4", "Materiais Utilizados (Baixa de Estoque)" }
                        p { class: "text-muted font-xs mb-3", "Confirme as quantidades reais utilizadas neste atendimento:" }
                        div { class: "consumed-items-list",
                            for (idx, item) in consumed_items().iter().enumerate() {
                                {
                                    let item_name = item.item_name.clone().unwrap_or_else(|| "Item de estoque".into());
                                    let planned = item.quantity_planned;
                                    let used_val = item.quantity_used.unwrap_or(planned);

                                    rsx! {
                                        div { key: "{item.item_id}", class: "consumed-item-row",
                                            span { class: "consumed-item-name", "{item_name}" }
                                            div { class: "consumed-item-qty-wrap",
                                                span { class: "text-muted font-xs mr-2", "Previsto: {planned}" }
                                                input {
                                                    class: "form-input qty-input",
                                                    r#type: "number",
                                                    min: "0",
                                                    value: "{used_val}",
                                                    oninput: move |e| {
                                                        let val = e.value().parse::<i32>().unwrap_or(0);
                                                        let mut items = consumed_items();
                                                        if let Some(it) = items.get_mut(idx) {
                                                            it.quantity_used = Some(val);
                                                        }
                                                        consumed_items.set(items);
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
                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| { let mut o = is_open; o.set(false); }, "Cancelar" }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        IconCheck { size: 16, color: "currentColor".to_string() }
                        span { if is_submitting() { "Atualizando..." } else { "Confirmar Status" } }
                    }
                }
            }
        }
    }
}
