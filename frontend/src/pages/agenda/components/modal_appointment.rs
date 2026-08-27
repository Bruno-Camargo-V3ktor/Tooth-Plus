use crate::components::modal::Modal;
use crate::icons::{IconCheck, IconClose, IconPlus, IconSearch, IconUser};
use shared::patients::Patient;
use shared::treatments::TreatmentTemplate;
use dioxus::prelude::*;

#[component]
pub fn ModalAppointment(
    is_open: bool,
    patients: Vec<Patient>,
    treatments: Vec<TreatmentTemplate>,
    labels: Vec<String>,
    selected_patient_id: Signal<String>,
    selected_label: Signal<String>,
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
    let current_patient = patient_query.read().clone();
    let current_pid = selected_patient_id.read().clone();
    let current_proc = procedure_name.read().clone();

    let mut is_patient_dropdown_open = use_signal(|| false);

    let treatments_lookup = treatments.clone();
    let treatments_options = treatments.clone();
    let labels_options = if labels.is_empty() {
        vec![
            "Primeira Consulta".to_string(),
            "Retorno".to_string(),
            "Avaliação".to_string(),
            "Urgência".to_string(),
            "Cirurgia".to_string(),
            "Manutenção".to_string(),
        ]
    } else {
        labels.clone()
    };

    // Filtra pacientes digitados
    let filtered_patients: Vec<Patient> = {
        let q = current_patient.to_lowercase();
        if q.is_empty() {
            patients.clone()
        } else {
            patients
                .iter()
                .filter(|p| {
                    p.full_name.to_lowercase().contains(&q)
                        || p.phone.contains(&q)
                        || p.document_cpf.as_ref().map(|c| c.contains(&q)).unwrap_or(false)
                })
                .cloned()
                .collect()
        }
    };

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
                            value: "{selected_label}",
                            onchange: move |e| selected_label.set(e.value()),
                            option { value: "", "Sem rótulo" }
                            for lbl in labels_options {
                                option { value: "{lbl}", "{lbl}" }
                            }
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
                    // Campo de Paciente com Busca em Tempo Real e Dropdown Fluido
                    div { class: "form-field", style: "position: relative;",
                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;",
                            label { class: "form-label", style: "color: #ef4444; margin: 0;", "Paciente *" }
                            a { href: "/patients", style: "font-size: 12px; color: #00a0e4; text-decoration: none; font-weight: 600;", "Cadastrar novo paciente" }
                        }

                        div { style: "position: relative; display: flex; align-items: center;",
                            input {
                                class: "form-input",
                                style: "padding-left: 36px;",
                                placeholder: "Digite o nome, telefone ou CPF para pesquisar...",
                                value: "{current_patient}",
                                onfocus: move |_| is_patient_dropdown_open.set(true),
                                oninput: move |e| {
                                    patient_query.set(e.value());
                                    selected_patient_id.set(String::new());
                                    is_patient_dropdown_open.set(true);
                                },
                            }
                            div { style: "position: absolute; left: 10px; pointer-events: none;",
                                IconSearch { size: 16, color: "#64748b".to_string() }
                            }
                        }

                        if *is_patient_dropdown_open.read() && !filtered_patients.is_empty() {
                            div {
                                style: "position: absolute; top: calc(100% + 4px); left: 0; right: 0; max-height: 200px; overflow-y: auto; background: #1a2236; border: 1px solid rgba(255,255,255,0.12); border-radius: 8px; box-shadow: 0 10px 25px rgba(0,0,0,0.5); z-index: 999;",
                                for p in filtered_patients {
                                    {
                                        let p_name = p.full_name.clone();
                                        let p_phone = p.phone.clone();
                                        let p_id = p.id.clone();
                                        let is_sel = current_pid == p_id;

                                        rsx! {
                                            div {
                                                key: "{p_id}",
                                                style: if is_sel { "padding: 8px 12px; background: rgba(0,160,228,0.15); cursor: pointer; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid rgba(255,255,255,0.05);" } else { "padding: 8px 12px; cursor: pointer; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid rgba(255,255,255,0.05);" },
                                                onclick: move |_| {
                                                    selected_patient_id.set(p_id.clone());
                                                    patient_query.set(p_name.clone());
                                                    is_patient_dropdown_open.set(false);
                                                },
                                                div { style: "display: flex; align-items: center; gap: 8px;",
                                                    div { style: "width: 26px; height: 26px; border-radius: 50%; background: #0284c7; color: #fff; font-size: 11px; font-weight: 700; display: flex; align-items: center; justify-content: center;",
                                                        "{p_name.chars().next().unwrap_or('P')}"
                                                    }
                                                    div {
                                                        div { style: "font-size: 13px; font-weight: 700; color: #f1f5f9;", "{p_name}" }
                                                        div { style: "font-size: 11px; color: #94a3b8;", "{p_phone}" }
                                                    }
                                                }
                                                if is_sel {
                                                    IconCheck { size: 14, color: "#22c55e".to_string() }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Campo de Tratamento / Procedimento (Opcional)
                    div { class: "form-field",
                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;",
                            label { class: "form-label", style: "margin: 0;", "Tratamento / Procedimento (Opcional)" }
                            a { href: "/treatments", style: "font-size: 12px; color: #00a0e4; text-decoration: none; font-weight: 600;", "Cadastrar novo tratamento" }
                        }
                        select {
                            class: "form-select",
                            value: "{current_proc}",
                            onchange: move |e| {
                                let val = e.value();
                                procedure_name.set(val.clone());
                                if let Some(t) = treatments_lookup.iter().find(|t| t.name == val) {
                                    duration.set(t.estimated_duration_minutes.unwrap_or(30).max(15) as u32);
                                }
                            },
                            option { value: "", "Nenhum procedimento selecionado..." }
                            for t in treatments_options {
                                {
                                    let t_cat = t.category.clone().unwrap_or_else(|| "Geral".to_string());
                                    let t_dur = t.estimated_duration_minutes.unwrap_or(30);
                                    rsx! {
                                        option { value: "{t.name}", "{t.name} ({t_cat}) • {t_dur} min" }
                                    }
                                }
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
                                option { value: "120", "120" }
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

                    // Card estilizado para Data e Hora no Compromisso
                    div {
                        style: "background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; padding: 14px; display: flex; flex-direction: column; gap: 12px;",

                        div { style: "display: flex; justify-content: space-between; align-items: center;",
                            h4 { style: "font-size: 13.5px; font-weight: 700; color: #38bdf8; margin: 0;", "Data e hora" }
                            div { style: "display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: #94a3b8;",
                                input { r#type: "checkbox", id: "allday" }
                                label { r#for: "allday", style: "cursor: pointer;", "Dia inteiro" }
                            }
                        }

                        div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Começa em *" }
                                input { class: "form-input", r#type: "date", value: "{appt_date}" }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Horário início *" }
                                input { class: "form-input", r#type: "time", value: "{appt_time}" }
                            }
                        }

                        div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Termina em *" }
                                input { class: "form-input", r#type: "date", value: "{appt_date}" }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Horário fim *" }
                                input { class: "form-input", r#type: "time", value: "11:15" }
                            }
                        }

                        div { style: "display: flex; align-items: center; gap: 8px; font-size: 12.5px; color: #cbd5e1; padding-top: 4px; border-top: 1px solid rgba(255,255,255,0.05);",
                            input { r#type: "checkbox", id: "repeat-appt" }
                            label { r#for: "repeat-appt", style: "cursor: pointer;", "Repetir compromisso" }
                        }
                    }
                }
            }
        }
    }
}
