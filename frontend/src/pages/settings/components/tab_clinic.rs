use dioxus::prelude::*;

#[component]
pub fn TabClinic(
    trading_name: Signal<String>,
    cnpj: Signal<String>,
    phone: Signal<String>,
    street: Signal<String>,
    number: Signal<String>,
    on_save: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "settings-card",
            div { class: "settings-card-header",
                h3 { class: "settings-card-title", "Identificação e Contato da Clínica" }
            }
            div { class: "settings-card-body",
                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Nome Fantasia" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            value: "{trading_name}",
                            oninput: move |e| trading_name.set(e.value()),
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "CNPJ" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            value: "{cnpj}",
                            oninput: move |e| cnpj.set(e.value()),
                        }
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Telefone Comercial" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            value: "{phone}",
                            oninput: move |e| phone.set(e.value()),
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Logradouro & Número" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            value: "{street}, {number}",
                            readonly: true,
                        }
                    }
                }
            }
            div { class: "settings-card-footer",
                button {
                    class: "btn-modal-primary",
                    onclick: move |_| on_save.call(()),
                    "Salvar Dados da Clínica"
                }
            }
        }
    }
}
