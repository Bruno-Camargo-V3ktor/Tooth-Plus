use crate::icons::IconPlus;
use shared::patients::Patient;
use shared::treatments::PatientTreatmentPlan;
use dioxus::prelude::*;

#[component]
pub fn TabBudgets(
    patient: Patient,
    plans: Vec<PatientTreatmentPlan>,
    on_new_budget: EventHandler<()>,
) -> Element {
    let patient_plans: Vec<PatientTreatmentPlan> = plans
        .into_iter()
        .filter(|p| p.patient_id == patient.id)
        .collect();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { style: "display: flex; align-items: center; justify-content: space-between; background: #121a2c; padding: 12px 18px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);",
                div {
                    h3 { style: "font-size: 15px; font-weight: 700; color: #f8fafc; margin: 0;", "Orçamentos & Planos de Tratamento" }
                    p { style: "font-size: 12.5px; color: #94a3b8; margin: 2px 0 0 0;", "Propostas financeiras e aprovação de tratamentos do paciente." }
                }

                button {
                    r#type: "button",
                    class: "btn-new-patient-green",
                    style: "height: 38px;",
                    onclick: move |_| on_new_budget.call(()),
                    IconPlus { size: 15, color: "#ffffff".to_string() }
                    span { "NOVO ORÇAMENTO" }
                }
            }

            if patient_plans.is_empty() {
                div { class: "empty-debits-box",
                    h3 { class: "empty-debits-title", "Nenhum orçamento cadastrado" }
                    p { class: "empty-debits-desc", "Crie um orçamento para apresentar os procedimentos e valores ao paciente." }
                }
            } else {
                div { style: "display: flex; flex-direction: column; gap: 12px;",
                    for plan in patient_plans {
                        div {
                            key: "{plan.id}",
                            class: "patient-card",
                            div { class: "patient-card-header",
                                strong { style: "color: #f8fafc; font-size: 14px;", "{plan.title}" }
                                span {
                                    class: "badge badge-blue",
                                    "{plan.status.label()}"
                                }
                            }
                            div { class: "patient-card-body",
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    span { style: "color: #94a3b8; font-size: 13px;", "{plan.items.len()} procedimento(s) planejado(s)" }
                                    strong { style: "color: #38bdf8; font-size: 15px;", "R$ {plan.total_price_cents as f64 / 100.0:.2}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
