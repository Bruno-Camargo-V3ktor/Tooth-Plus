use crate::components::modal::Modal;
use shared::documents::ContractTemplate;
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn IssueDocumentModal(
    is_open: bool,
    templates: Vec<ContractTemplate>,
    patients: Vec<Patient>,
    selected_patient_id: Signal<String>,
    selected_template_id: Signal<String>,
    document_title: Signal<String>,
    document_type: Signal<String>,
    requires_patient_sig: Signal<bool>,
    requires_doctor_sig: Signal<bool>,
    is_already_signed: Signal<bool>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let templates_options = templates.clone();

    rsx! {
        Modal {
            title: "Emitir Novo Documento / Contrato Clínico".to_string(),
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
                    "Emitir Documento"
                }
            },

            div { style: "display: flex; flex-direction: column; gap: 14px;",
                div { class: "form-field",
                    label { class: "form-label", "Paciente *" }
                    select {
                        class: "form-select",
                        value: "{selected_patient_id}",
                        onchange: move |e| selected_patient_id.set(e.value()),
                        option { value: "", "Selecione o paciente..." }
                        for p in patients {
                            option { value: "{p.id}", "{p.full_name} ({p.phone})" }
                        }
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Modelo / Template (Opcional)" }
                        select {
                            class: "form-select",
                            value: "{selected_template_id}",
                            onchange: move |e| {
                                let val = e.value();
                                selected_template_id.set(val.clone());
                                if let Some(t) = templates_options.iter().find(|t| t.id == val) {
                                    document_title.set(t.title.clone());
                                }
                            },
                            option { value: "", "Documento Avulso / Personalizado" }
                            for t in templates {
                                option { value: "{t.id}", "{t.title} ({t.category})" }
                            }
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Tipo do Documento" }
                        select {
                            class: "form-select",
                            value: "{document_type}",
                            onchange: move |e| document_type.set(e.value()),
                            option { value: "contract", "Contrato Odontológico" }
                            option { value: "consent", "Termo de Consentimento (TCLE)" }
                            option { value: "certificate", "Atestado de Repouso / Comparecimento" }
                            option { value: "prescription", "Receituário & Prescrição" }
                            option { value: "custom", "Outro Documento" }
                        }
                    }
                }

                div { class: "form-field",
                    label { class: "form-label", "Título do Documento *" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Contrato de Prestação de Serviços - Prótese Fixa...",
                        value: "{document_title}",
                        oninput: move |e| document_title.set(e.value()),
                    }
                }

                div { style: "background: #0b1120; border: 1px solid rgba(255,255,255,0.08); padding: 12px; border-radius: 8px; display: flex; flex-direction: column; gap: 8px;",
                    span { style: "font-size: 12px; font-weight: 700; color: #94a3b8; text-transform: uppercase;", "Configurações de Assinatura" }

                    label { style: "display: flex; align-items: center; gap: 8px; font-size: 13px; color: #cbd5e1; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: "{requires_patient_sig}",
                            onchange: move |e| requires_patient_sig.set(e.checked()),
                        }
                        span { "Exigir assinatura digital do paciente (via portal / QR Code)" }
                    }

                    label { style: "display: flex; align-items: center; gap: 8px; font-size: 13px; color: #cbd5e1; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: "{requires_doctor_sig}",
                            onchange: move |e| requires_doctor_sig.set(e.checked()),
                        }
                        span { "Exigir assinatura digital do dentista responsável" }
                    }

                    label { style: "display: flex; align-items: center; gap: 8px; font-size: 13px; color: #cbd5e1; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: "{is_already_signed}",
                            onchange: move |e| is_already_signed.set(e.checked()),
                        }
                        span { "Documento já assinado fisicamente em papel" }
                    }
                }
            }
        }
    }
}
