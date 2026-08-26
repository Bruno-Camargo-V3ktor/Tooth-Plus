use dioxus::prelude::*;

#[component]
pub fn TabHours(
    opening_hour: Signal<u32>,
    closing_hour: Signal<u32>,
    on_save: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "settings-card",
            div { class: "settings-card-header",
                h3 { class: "settings-card-title", "Horário de Atendimento e Grade da Agenda" }
            }
            div { class: "settings-card-body",
                p { style: "font-size: 13px; color: #94a3b8; margin: 0 0 8px 0;",
                    "Defina os horários em que a clínica realiza consultas. A grade da página Agenda será ajustada automaticamente para iniciar e terminar nestes horários."
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Hora de Abertura (Início da Agenda) *" }
                        select {
                            class: "form-select",
                            value: "{opening_hour}",
                            onchange: move |e| {
                                if let Ok(v) = e.value().parse::<u32>() { opening_hour.set(v); }
                            },
                            option { value: "6", "06:00" }
                            option { value: "7", "07:00" }
                            option { value: "8", "08:00 (Padrão)" }
                            option { value: "9", "09:00" }
                            option { value: "10", "10:00" }
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Hora de Fechamento (Fim da Agenda) *" }
                        select {
                            class: "form-select",
                            value: "{closing_hour}",
                            onchange: move |e| {
                                if let Ok(v) = e.value().parse::<u32>() { closing_hour.set(v); }
                            },
                            option { value: "17", "17:00" }
                            option { value: "18", "18:00" }
                            option { value: "19", "19:00 (Padrão)" }
                            option { value: "20", "20:00" }
                            option { value: "21", "21:00" }
                            option { value: "22", "22:00" }
                        }
                    }
                }
            }
            div { class: "settings-card-footer",
                button {
                    class: "btn-modal-primary",
                    onclick: move |_| on_save.call(()),
                    "Salvar Alterações de Horário"
                }
            }
        }
    }
}
