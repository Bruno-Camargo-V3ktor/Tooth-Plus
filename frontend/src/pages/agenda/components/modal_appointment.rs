use crate::components::modal::Modal;
use dioxus::prelude::*;

#[component]
pub fn ModalAppointment(
    is_open: bool,
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

    rsx! {
        Modal {
            title: if *is_compromisso.read() { "Novo Compromisso".to_string() } else { "Nova Consulta".to_string() },
            is_open,
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                button {
                    r#type: "button",
                    class: "btn-modal-ghost",
                    onclick: move |_| on_close.call(()),
                    "Cancelar"
                }
                button {
                    r#type: "button",
                    class: "btn-modal-primary",
                    onclick: move |_| on_submit.call(()),
                    "Confirmar Agendamento"
                }
            },

            div { class: "form-row-2 form-row",
                div { class: "form-field",
                    label { class: "form-label", "Data *" }
                    input {
                        class: "form-input",
                        r#type: "date",
                        value: "{appt_date}",
                        oninput: move |e| appt_date.set(e.value()),
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Horário *" }
                    input {
                        class: "form-input",
                        r#type: "time",
                        value: "{appt_time}",
                        oninput: move |e| appt_time.set(e.value()),
                    }
                }
            }

            div { class: "form-row-2 form-row",
                div { class: "form-field",
                    label { class: "form-label", "Duração (minutos) *" }
                    select {
                        class: "form-select",
                        value: "{duration}",
                        onchange: move |e| {
                            if let Ok(v) = e.value().parse::<u32>() { duration.set(v); }
                        },
                        option { value: "15", "15 minutos" }
                        option { value: "30", "30 minutos (Padrão)" }
                        option { value: "45", "45 minutos" }
                        option { value: "60", "1 hora" }
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Dentista Responsável" }
                    select {
                        class: "form-select",
                        value: "{assigned_user_id}",
                        onchange: move |e| assigned_user_id.set(e.value()),
                        option { value: "usr-1", "Dr. Roberto Alencar" }
                        option { value: "usr-2", "Dr. Lucas Mendes" }
                    }
                }
            }

            if !*is_compromisso.read() {
                div { class: "form-field",
                    label { class: "form-label", "Paciente *" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Nome do paciente...",
                        value: "{patient_query}",
                        oninput: move |e| patient_query.set(e.value()),
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Procedimento Previsto" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Avaliação Inicial, Restauração...",
                        value: "{procedure_name}",
                        oninput: move |e| procedure_name.set(e.value()),
                    }
                }
            } else {
                div { class: "form-field",
                    label { class: "form-label", "Título do Compromisso *" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Reunião de equipe, Manutenção...",
                        value: "{patient_query}",
                        oninput: move |e| patient_query.set(e.value()),
                    }
                }
            }

            div { class: "form-field",
                label { class: "form-label", "Observações" }
                textarea {
                    class: "form-textarea",
                    placeholder: "Anotações adicionais...",
                    value: "{notes}",
                    oninput: move |e| notes.set(e.value()),
                }
            }
        }
    }
}
