use super::tab_odontogram::TabOdontogram;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::IconPlus;
use shared::patients::{Patient, PatientTreatment};
use dioxus::prelude::*;

#[component]
pub fn TabTreatments(
    patient: Patient,
    treatments: Vec<PatientTreatment>,
    on_add_treatment: EventHandler<(String, String, String, i64)>,
) -> Element {
    let mut plan_name = use_signal(|| "Particular".to_string());
    let mut treatment_name = use_signal(String::new);
    let mut selected_teeth = use_signal(Vec::<String>::new);
    let mut value_str = use_signal(String::new);
    let toast = consume_context::<ToastState>();

    let handle_save = {
        let mut toast_c = toast.clone();
        let plan_n = plan_name.clone();
        let treat_n = treatment_name.clone();
        let sel_t = selected_teeth.clone();
        let val_s = value_str.clone();

        move |_| {
            let t_name = treat_n.read().trim().to_string();
            let val_num: f64 = val_s.read().replace(',', ".").parse().unwrap_or(0.0);

            if t_name.is_empty() {
                toast_c.show("Informe o nome do tratamento.", ToastVariant::Error);
                return;
            }

            let cost_cents = (val_num * 100.0) as i64;
            let teeth_str = sel_t.read().join(", ");

            on_add_treatment.call((
                plan_n.read().clone(),
                t_name,
                teeth_str,
                cost_cents,
            ));

            let mut t_mut = treat_n;
            let mut s_mut = sel_t;
            let mut v_mut = val_s;
            t_mut.set(String::new());
            s_mut.set(vec![]);
            v_mut.set(String::new());
            toast_c.show("Tratamento adicionado com sucesso!", ToastVariant::Success);
        }
    };

    let mut toast_evo = toast.clone();

    rsx! {
        div { class: "patient-tab-grid-2",
            // Coluna Esquerda: Formulário de Adicionar + Odontograma
            div { style: "display: flex; flex-direction: column; gap: 16px;",
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Adicionar tratamento" }
                    }
                    div { class: "patient-card-body",
                        div { class: "form-row-2 form-row",
                            div { class: "form-field",
                                label { class: "form-label", "Plano *" }
                                select {
                                    class: "form-select",
                                    value: "{plan_name}",
                                    onchange: move |e| plan_name.set(e.value()),
                                    option { value: "Particular", "Particular" }
                                    option { value: "Amil Dental", "Amil Dental" }
                                    option { value: "Unimed Odonto", "Unimed Odonto" }
                                    option { value: "Bradesco Dental", "Bradesco Dental" }
                                }
                            }
                            div { class: "form-field",
                                label { class: "form-label", "Tratamento *" }
                                select {
                                    class: "form-select",
                                    value: "{treatment_name}",
                                    onchange: move |e| {
                                        let v = e.value();
                                        treatment_name.set(v.clone());
                                        if v.contains("Restauração") { value_str.set("250.00".to_string()); }
                                        else if v.contains("Profilaxia") { value_str.set("180.00".to_string()); }
                                        else if v.contains("Canal") { value_str.set("850.00".to_string()); }
                                        else if v.contains("Extração") { value_str.set("350.00".to_string()); }
                                        else if v.contains("Clareamento") { value_str.set("900.00".to_string()); }
                                    },
                                    option { value: "", "Selecione o procedimento..." }
                                    option { value: "Restauração em Resina Composta", "Restauração em Resina Composta" }
                                    option { value: "Profilaxia & Raspagem Supragengival", "Profilaxia & Raspagem Supragengival" }
                                    option { value: "Tratamento Endodôntico (Canal)", "Tratamento Endodôntico (Canal)" }
                                    option { value: "Exodontia Simples", "Exodontia Simples" }
                                    option { value: "Clareamento Dental Caseiro", "Clareamento Dental Caseiro" }
                                }
                            }
                        }

                        div { class: "form-row-2 form-row",
                            div { class: "form-field",
                                label { class: "form-label", "Dentes/Região" }
                                input {
                                    class: "form-input",
                                    r#type: "text",
                                    placeholder: "Ex: 18, 21 ou selecione no Odontograma",
                                    value: "{selected_teeth.read().join(\", \")}",
                                    readonly: true,
                                }
                            }
                            div { class: "form-field",
                                label { class: "form-label", "Valor (R$) *" }
                                input {
                                    class: "form-input",
                                    r#type: "number",
                                    step: "0.01",
                                    placeholder: "0.00",
                                    value: "{value_str}",
                                    oninput: move |e| value_str.set(e.value()),
                                }
                            }
                        }

                        div { style: "display: flex; justify-content: flex-end; margin-top: 8px;",
                            button {
                                r#type: "button",
                                class: "btn-primary",
                                onclick: handle_save,
                                "Salvar Procedimento"
                            }
                        }
                    }
                }

                // Odontograma
                TabOdontogram {
                    selected_teeth,
                    on_toggle_tooth: move |t_num: String| {
                        let mut curr = selected_teeth.read().clone();
                        if curr.contains(&t_num) {
                            curr.retain(|x| x != &t_num);
                        } else {
                            curr.push(t_num);
                        }
                        selected_teeth.set(curr);
                    },
                }
            }

            // Coluna Direita: Evoluções Clínicas
            div { style: "display: flex; flex-direction: column; gap: 16px;",
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Evoluções" }
                        button {
                            r#type: "button",
                            class: "btn-modal-ghost",
                            style: "padding: 4px 8px; font-size: 13px;",
                            title: "Adicionar Nova Evolução",
                            IconPlus { size: 16, color: "#00a0e4".to_string() }
                        }
                    }
                    div { class: "patient-card-body",
                        if treatments.is_empty() {
                            div { style: "padding: 48px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; gap: 12px;",
                                span { style: "font-size: 14px; color: #94a3b8;", "O paciente não possui evoluções" }
                                button {
                                    r#type: "button",
                                    class: "btn-new-patient-green",
                                    style: "font-size: 12px; height: 36px;",
                                    onclick: move |_| {
                                        toast_evo.show("Selecione um procedimento para registrar evolução.", ToastVariant::Info);
                                    },
                                    "ADICIONAR EVOLUÇÃO"
                                }
                            }
                        } else {
                            div { style: "display: flex; flex-direction: column; gap: 10px;",
                                for treat in treatments {
                                    div {
                                        key: "{treat.id}",
                                        style: "background: #162035; border: 1px solid rgba(255,255,255,0.06); padding: 12px 14px; border-radius: 8px; display: flex; flex-direction: column; gap: 4px;",
                                        div { style: "display: flex; align-items: center; justify-content: space-between;",
                                            strong { style: "color: #f8fafc; font-size: 13.5px;", "{treat.procedure_name}" }
                                            span { style: "font-size: 11.5px; color: #38bdf8; font-weight: 700;", "R$ {treat.cost_cents as f64 / 100.0:.2}" }
                                        }
                                        div { style: "display: flex; align-items: center; justify-content: space-between; font-size: 12px; color: #94a3b8;",
                                            span { "Dente: {treat.tooth_number.clone().unwrap_or_else(|| \"Geral\".to_string())}" }
                                            span { "Status: {treat.status}" }
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
