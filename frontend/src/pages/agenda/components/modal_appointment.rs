use crate::components::modal::Modal;
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn ModalAppointment(
    is_open: bool,
    patients: Vec<Patient>,
    selected_patient_id: Signal<String>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
    is_compromisso: Signal<bool>,
    patient_query: Signal<String>,
    appt_date: Signal<String>,
    appt_time: Signal<String>,
    duration: Signal<u32>,
    procedure_name: Signal<String>,
    notes: Signal<String>,
    assigned_user_id: Signal<String>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let is_comp = *is_compromisso.read();
    let current_notes = notes.read().clone();
    let patients_options = patients.clone();
    let current_patient = patient_query.read().clone();
    let current_pid = selected_patient_id.read().clone();

    rsx! {
        Modal {
            title: "".to_string(),
            is_open,
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                div { style: "display: flex; align-items: center; justify-content: space-between; width: 100%;",
                    if !is_comp {
                        select {
                            class: "form-select",
                            style: "max-width: 180px; height: 36px; font-size: 12.5px; background: rgba(255,255,255,0.05);",
                            option { value: "", "Sem rótulo" }
                            option { value: "urgencia", "Urgência" }
                            option { value: "primeira", "Primeira Consulta" }
                            option { value: "retorno", "Retorno" }
                            option { value: "cirurgia", "Cirurgia" }
                            option { value: "avaliacao", "Avaliação" }
                        }
                    } else {
                        div {}
                    }

                    div { style: "display: flex; align-items: center; gap: 10px;",
                        button {
                            r#type: "button",
                            class: "btn-modal-ghost",
                            style: "font-weight: 700; font-size: 12.5px; text-transform: uppercase;",
                            onclick: move |_| on_close.call(()),
                            "FECHAR"
                        }
                        button {
                            r#type: "button",
                            class: "btn-new-patient-green",
                            style: "font-weight: 700; font-size: 13px; text-transform: uppercase; padding: 0 20px; height: 38px;",
                            onclick: move |_| on_submit.call(()),
                            "MARCAR"
                        }
                    }
                }
            },

            div { style: "display: flex; flex-direction: column; gap: 14px;",
                // Top Tab Bar (Consulta / Compromisso)
                div { style: "display: flex; align-items: center; border-bottom: 1px solid rgba(255,255,255,0.08); padding-bottom: 12px; margin-bottom: 4px;",
                    div { style: "display: flex; gap: 8px;",
                        button {
                            r#type: "button",
                            class: if !is_comp { "btn-filter-pill active" } else { "btn-filter-pill" },
                            style: "padding: 6px 18px; font-size: 13px; font-weight: 700;",
                            onclick: move |_| is_compromisso.set(false),
                            "Consulta"
                        }
                        button {
                            r#type: "button",
                            class: if is_comp { "btn-filter-pill active" } else { "btn-filter-pill" },
                            style: "padding: 6px 18px; font-size: 13px; font-weight: 700;",
                            onclick: move |_| is_compromisso.set(true),
                            "Compromisso"
                        }
                    }
                }

                if !is_comp {
                    // MODO CONSULTA
                    div { class: "form-field",
                        div { style: "display: flex; justify-content: space-between; align-items: center;",
                            label { class: "form-label", style: "color: #ef4444;", "Paciente *" }
                            a { href: "/patients", style: "font-size: 12px; color: #00a0e4; text-decoration: none; font-weight: 600;", "Cadastrar novo paciente" }
                        }
                        select {
                            class: "form-select",
                            value: "{current_pid}",
                            onchange: move |e| {
                                let val = e.value();
                                selected_patient_id.set(val.clone());
                                if let Some(p) = patients.iter().find(|p| p.id == val) {
                                    patient_query.set(p.full_name.clone());
                                }
                            },
                            option { value: "", "Selecione um paciente cadastrado..." }
                            for p in patients_options {
                                option { value: "{p.id}", "{p.full_name} ({p.phone})" }
                            }
                        }
                    }

                    div { class: "form-field",
                        label { class: "form-label", "Profissional *" }
                        select {
                            class: "form-select",
                            value: "{assigned_user_id}",
                            onchange: move |e| assigned_user_id.set(e.value()),
                            option { value: "usr:dr_lucas", "Dr(a). Lucas Mendes - CRO 12345" }
                            option { value: "usr:dra_fernanda", "Dra. Fernanda Ramos - CRO 54321" }
                            option { value: "usr:dra_luria", "Dra. Luria Silva - CRO 98765" }
                        }
                    }

                    div { style: "display: grid; grid-template-columns: 1.5fr 1fr 1fr; gap: 10px;",
                        div { class: "form-field",
                            label { class: "form-label", "Data da consulta *" }
                            input {
                                class: "form-input",
                                r#type: "date",
                                value: "{appt_date}",
                                oninput: move |e| appt_date.set(e.value()),
                            }
                            a { href: "#", style: "font-size: 11.5px; color: #00a0e4; text-decoration: none; margin-top: 4px; display: inline-block;", "Encontrar horário livre" }
                        }

                        div { class: "form-field",
                            label { class: "form-label", "Hora de início *" }
                            input {
                                class: "form-input",
                                r#type: "time",
                                value: "{appt_time}",
                                oninput: move |e| appt_time.set(e.value()),
                            }
                        }

                        div { class: "form-field",
                            label { class: "form-label", "Duração (min) *" }
                            select {
                                class: "form-select",
                                value: "{duration}",
                                onchange: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() { duration.set(v); }
                                },
                                option { value: "15", "15" }
                                option { value: "30", "30" }
                                option { value: "45", "45" }
                                option { value: "60", "60" }
                                option { value: "90", "90" }
                            }
                        }
                    }

                    div { class: "form-field",
                        div { style: "display: flex; justify-content: space-between;",
                            label { class: "form-label", "Observação" }
                            span { style: "font-size: 11px; color: #64748b;", "{current_notes.len()} / 500" }
                        }
                        textarea {
                            class: "form-textarea",
                            style: "height: 64px;",
                            maxlength: "500",
                            placeholder: "Observações clínicas, recomendações ou queixas principais...",
                            value: "{notes}",
                            oninput: move |e| notes.set(e.value()),
                        }
                    }

                    div { class: "form-field",
                        label { class: "form-label", "Retornar em" }
                        select {
                            class: "form-select",
                            option { value: "none", "Sem retorno" }
                            option { value: "7d", "7 dias" }
                            option { value: "15d", "15 dias" }
                            option { value: "30d", "30 dias" }
                            option { value: "6m", "6 meses" }
                        }
                    }

                    div { style: "display: flex; align-items: center; gap: 8px; font-size: 13px; color: #cbd5e1; padding: 4px 0;",
                        input { r#type: "checkbox", id: "send-reminder", checked: true }
                        label { r#for: "send-reminder", style: "cursor: pointer;", "Enviar confirmação e lembrete de consulta automático" }
                    }
                } else {
                    // MODO COMPROMISSO
                    div { class: "form-field",
                        div { style: "display: flex; justify-content: space-between;",
                            label { class: "form-label", style: "color: #ef4444;", "Título do compromisso *" }
                            span { style: "font-size: 11px; color: #64748b;", "{current_patient.len()} / 255" }
                        }
                        input {
                            class: "form-input",
                            r#type: "text",
                            maxlength: "255",
                            placeholder: "Ex: Reunião clínica, Manutenção do compressor...",
                            value: "{patient_query}",
                            oninput: move |e| patient_query.set(e.value()),
                        }
                    }

                    div { class: "form-field",
                        div { style: "display: flex; justify-content: space-between;",
                            label { class: "form-label", "Descrição" }
                            span { style: "font-size: 11px; color: #64748b;", "{current_notes.len()} / 500" }
                        }
                        textarea {
                            class: "form-textarea",
                            style: "height: 60px;",
                            maxlength: "500",
                            placeholder: "Detalhes do compromisso...",
                            value: "{notes}",
                            oninput: move |e| notes.set(e.value()),
                        }
                    }

                    div { class: "form-field",
                        label { class: "form-label", "Agenda de *" }
                        select {
                            class: "form-select",
                            value: "{assigned_user_id}",
                            onchange: move |e| assigned_user_id.set(e.value()),
                            option { value: "usr:dr_lucas", "Dr(a). Lucas Mendes" }
                            option { value: "usr:dra_fernanda", "Dra. Fernanda Ramos" }
                            option { value: "usr:dra_luria", "Dra. Luria Silva" }
                        }
                    }

                    div { style: "border-top: 1px solid rgba(255,255,255,0.06); padding-top: 10px;",
                        h4 { style: "font-size: 13.5px; font-weight: 700; color: #38bdf8; margin: 0 0 10px 0;", "Data e hora" }

                        div { style: "display: flex; align-items: center; gap: 8px; font-size: 13px; color: #cbd5e1; margin-bottom: 10px;",
                            input { r#type: "checkbox", id: "allday" }
                            label { r#for: "allday", style: "cursor: pointer;", "Dia inteiro" }
                        }

                        div { class: "form-row-2 form-row",
                            div { class: "form-field",
                                label { class: "form-label", "Começa em *" }
                                input { class: "form-input", r#type: "date", value: "{appt_date}" }
                            }
                            div { class: "form-field",
                                label { class: "form-label", "Horário início *" }
                                input { class: "form-input", r#type: "time", value: "{appt_time}" }
                            }
                        }

                        div { class: "form-row-2 form-row",
                            div { class: "form-field",
                                label { class: "form-label", "Termina em *" }
                                input { class: "form-input", r#type: "date", value: "{appt_date}" }
                            }
                            div { class: "form-field",
                                label { class: "form-label", "Horário fim *" }
                                input { class: "form-input", r#type: "time", value: "11:15" }
                            }
                        }

                        div { style: "display: flex; align-items: center; gap: 8px; font-size: 13px; color: #cbd5e1; margin-top: 10px;",
                            input { r#type: "checkbox", id: "repeat-appt" }
                            label { r#for: "repeat-appt", style: "cursor: pointer;", "Repetir compromisso" }
                        }
                    }
                }
            }
        }
    }
}
