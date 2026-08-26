use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn PatientKpis(patients: Vec<Patient>) -> Element {
    let total_patients = patients.len();
    let particular_count = patients.iter().filter(|p| p.insurance_plan.is_none()).count();
    let plan_count = total_patients.saturating_sub(particular_count);
    let with_guardians = patients.iter().filter(|p| !p.legal_guardians.is_empty()).count();

    rsx! {
        div { class: "patients-kpi-grid",
            div { class: "kpi-card",
                span { class: "kpi-card-label", "Total de Pacientes" }
                span { class: "kpi-card-value", "{total_patients}" }
                span { class: "kpi-card-sub", "Base ativa na clínica" }
            }
            div { class: "kpi-card",
                span { class: "kpi-card-label", "Pacientes Particulares" }
                span { class: "kpi-card-value", "{particular_count}" }
                span { class: "kpi-card-sub", "Atendimentos diretos" }
            }
            div { class: "kpi-card",
                span { class: "kpi-card-label", "Convênios & Planos" }
                span { class: "kpi-card-value", "{plan_count}" }
                span { class: "kpi-card-sub", "Amil, Uniodonto, Bradesco" }
            }
            div { class: "kpi-card",
                span { class: "kpi-card-label", "Com Responsável Legal" }
                span { class: "kpi-card-value", "{with_guardians}" }
                span { class: "kpi-card-sub", "Menores / Dependentes" }
            }
        }
    }
}
