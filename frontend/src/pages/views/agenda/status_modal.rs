//! # Modal de Atualização de Status do Atendimento (Frontend)
//!
//! Permite alterar o status do agendamento (Confirmado, Em Atendimento, Concluído, etc.)
//! registrando o motivo de cancelamento (pelo Doutor ou pelo Paciente) ou confirmando os insumos consumidos.

use crate::api::update_appointment_status;
use crate::components::icons::IconCheck;
use dioxus::prelude::*;
use shared::appointments::{AppointmentResponse, AppointmentStatus, UpdateAppointmentStatusRequest};

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
        AppointmentStatus::Pending => "pending".to_string(),
        AppointmentStatus::Confirmed => "confirmed".to_string(),
        AppointmentStatus::InProgress => "in_progress".to_string(),
        AppointmentStatus::Completed => "completed".to_string(),
        AppointmentStatus::CanceledByDoctor => "canceled_by_doctor".to_string(),
        AppointmentStatus::CanceledByPatient => "canceled_by_patient".to_string(),
        AppointmentStatus::Canceled => "canceled_by_doctor".to_string(),
        AppointmentStatus::NoShow => "no_show".to_string(),
    });
    let mut cancellation_reason = use_signal(|| app.cancellation_reason.clone().unwrap_or_default());
    let mut consumed_items = use_signal(|| app.consumed_items.clone());
    let mut is_submitting = use_signal(|| false);

    let app_id = app.id.clone();
    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_submit = move |_| {
        let status_enum = match selected_status().as_str() {
            "pending" => AppointmentStatus::Pending,
            "confirmed" => AppointmentStatus::Confirmed,
            "in_progress" => AppointmentStatus::InProgress,
            "completed" => AppointmentStatus::Completed,
            "canceled_by_doctor" => AppointmentStatus::CanceledByDoctor,
            "canceled_by_patient" => AppointmentStatus::CanceledByPatient,
            "canceled" => AppointmentStatus::Canceled,
            "no_show" => AppointmentStatus::NoShow,
            _ => AppointmentStatus::Pending,
        };

        let reason_opt = if status_enum.is_canceled() {
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

    let is_canceled_selected = selected_status() == "canceled_by_doctor" || selected_status() == "canceled_by_patient" || selected_status() == "canceled";

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal modal-small",
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", "Atualizar Status do Atendimento" }
                        p { class: "modal-subtitle", "Avance as etapas do atendimento ou registre o cancelamento com justificativa." }
                    }
                    button { class: "modal-close", onclick: move |_| { let mut o = is_open; o.set(false); }, "×" }
                }

                div { class: "modal-body",
                    div { class: "appointment-summary-box mb-3",
                        h4 { class: "font-weight-bold mb-1", "{app.title}" }
                        p { class: "text-muted font-xs",
                            "Paciente: {app.patient_name.as_deref().unwrap_or(\"Não informado\")}"
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Novo Status *" }
                        select {
                            class: "select-field",
                            value: "{selected_status}",
                            onchange: move |e| selected_status.set(e.value()),
                            option { value: "pending", "⏳ Pendente / Aguardando" }
                            option { value: "confirmed", "✓ Confirmado pelo Paciente" }
                            option { value: "in_progress", "🦷 Em Atendimento no Consultório" }
                            option { value: "completed", "★ Atendimento Concluído" }
                            option { value: "canceled_by_doctor", "✕ Cancelado pelo Doutor / Clínica" }
                            option { value: "canceled_by_patient", "✕ Cancelado pelo Paciente" }
                            option { value: "no_show", "⚠ Não Compareceu (Falta)" }
                        }
                    }

                    if is_canceled_selected {
                        div { class: "form-group mt-2",
                            label { class: "form-label text-danger font-semibold",
                                if selected_status() == "canceled_by_doctor" {
                                    "Motivo / Observações do Cancelamento pelo Doutor *"
                                } else {
                                    "Motivo / Observações do Cancelamento pelo Paciente *"
                                }
                            }
                            textarea {
                                class: "form-textarea",
                                placeholder: if selected_status() == "canceled_by_doctor" {
                                    "Ex: Profissional em procedimento de emergência / Reagendado a pedido da clínica..."
                                } else {
                                    "Ex: Paciente solicitou reagendamento por motivos de viagem / trabalho..."
                                },
                                value: "{cancellation_reason}",
                                oninput: move |e| cancellation_reason.set(e.value())
                            }
                        }
                    }

                    if selected_status() == "completed" && !consumed_items().is_empty() {
                        div { class: "form-group mt-3",
                            h4 { class: "form-section-title", "Materiais Utilizados (Baixa de Estoque)" }
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
                }

                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| { let mut o = is_open; o.set(false); },
                        "Cancelar"
                    }
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
