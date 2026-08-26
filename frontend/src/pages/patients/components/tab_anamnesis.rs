use crate::components::toast::{ToastState, ToastVariant};
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn TabAnamnesis(patient: Patient) -> Element {
    let mut allergies = use_signal(|| "Nenhuma alergia relatada".to_string());
    let mut systemic = use_signal(|| "Pressão arterial controlada".to_string());
    let mut medications = use_signal(|| "Nenhum medicamento de uso contínuo".to_string());
    let mut is_smoker = use_signal(|| false);
    let mut is_bleeder = use_signal(|| false);
    let mut toast = consume_context::<ToastState>();

    rsx! {
        div { class: "patient-card",
            div { class: "patient-card-header",
                h3 { class: "patient-card-title", "Questionário Clínico de Anamnese" }
            }
            div { class: "patient-card-body",
                div { class: "form-field",
                    label { class: "form-label", "Alergias a Medicamentos / Alimentos *" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        value: "{allergies}",
                        oninput: move |e| allergies.set(e.value()),
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Doenças Crônicas / Condições Sistêmicas" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            value: "{systemic}",
                            oninput: move |e| systemic.set(e.value()),
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Medicamentos de Uso Contínuo" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            value: "{medications}",
                            oninput: move |e| medications.set(e.value()),
                        }
                    }
                }

                div { style: "display: flex; gap: 24px; margin-top: 8px;",
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        input {
                            r#type: "checkbox",
                            id: "chk-smoker",
                            checked: "{is_smoker}",
                            onchange: move |e| is_smoker.set(e.checked()),
                        }
                        label { r#for: "chk-smoker", style: "font-size: 13.5px; color: #cbd5e1;", "Paciente fumante" }
                    }
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        input {
                            r#type: "checkbox",
                            id: "chk-bleed",
                            checked: "{is_bleeder}",
                            onchange: move |e| is_bleeder.set(e.checked()),
                        }
                        label { r#for: "chk-bleed", style: "font-size: 13.5px; color: #cbd5e1;", "Histórico de hemorragia / cicatrização lenta" }
                    }
                }

                div { style: "display: flex; justify-content: flex-end; margin-top: 12px;",
                    button {
                        r#type: "button",
                        class: "btn-primary",
                        onclick: move |_| {
                            toast.show("Ficha de anamnese atualizada!", ToastVariant::Success);
                        },
                        "Salvar Anamnese"
                    }
                }
            }
        }
    }
}
