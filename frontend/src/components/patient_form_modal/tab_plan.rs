use dioxus::prelude::*;

#[component]
pub fn TabPlan(insurance_plan: Signal<String>) -> Element {
    rsx! {
        div { class: "form-field",
            label { class: "form-label", "Plano Odontológico / Convênio" }
            input {
                class: "form-input",
                r#type: "text",
                placeholder: "Ex: Unimed Odonto, Amil Dental, Bradesco...",
                value: "{insurance_plan}",
                oninput: move |e| insurance_plan.set(e.value()),
            }
        }
    }
}
