use crate::icons::{IconCamera, IconInfo};
use shared::clinics::ClinicAddress;
use dioxus::prelude::*;

#[component]
pub fn TabClinic(
    trading_name: Signal<String>,
    cnpj: Signal<String>,
    comm_name: Signal<String>,
    manager_name: Signal<String>,
    opening_hour: Signal<u32>,
    closing_hour: Signal<u32>,
    timezone: Signal<String>,
    fiscal_issuer: Signal<String>,
    email: Signal<String>,
    phone: Signal<String>,
    cellphone: Signal<String>,
    print_letterhead: Signal<bool>,
    letterhead_options: Signal<Vec<String>>,
    address: Signal<ClinicAddress>,
    on_save: EventHandler<()>,
) -> Element {
    let t_name = trading_name.read().clone();
    let c_cnpj = cnpj.read().clone();
    let c_comm = comm_name.read().clone();
    let c_mgr = manager_name.read().clone();

    let addr = address.read().clone();
    let opts = letterhead_options.read().clone();



    rsx! {
        div { class: "settings-section",
            // 1. DADOS DA CLÍNICA
            h3 { class: "settings-section-title", "Dados da Clínica" }

            div { style: "display: grid; grid-template-columns: 1fr 140px; gap: 20px; align-items: flex-start;",
                div { style: "display: flex; flex-direction: column; gap: 14px;",
                    div { style: "display: grid; grid-template-columns: 2fr 1fr; gap: 14px;",
                        div { class: "form-field", style: "margin: 0;",
                            label { class: "form-label", "Nome da clínica*" }
                            input {
                                class: "form-input",
                                maxlength: "150",
                                value: "{trading_name}",
                                oninput: move |e| trading_name.set(e.value()),
                            }
                            div { class: "settings-field-counter", "{t_name.len()} / 150" }
                        }

                        div { class: "form-field", style: "margin: 0;",
                            label { class: "form-label", "CNPJ da clínica" }
                            input {
                                class: "form-input",
                                placeholder: "00.000.000/0000-00",
                                value: "{cnpj}",
                                oninput: move |e| cnpj.set(e.value()),
                            }
                        }
                    }

                    div { style: "display: grid; grid-template-columns: 1fr 1.5fr; gap: 14px;",
                        div { class: "form-field", style: "margin: 0;",
                            label { class: "form-label", "Nome utilizado nas comunicações*" }
                            input {
                                class: "form-input",
                                maxlength: "30",
                                value: "{comm_name}",
                                oninput: move |e| comm_name.set(e.value()),
                            }
                            div { class: "settings-field-counter", "{c_comm.len()} / 30" }
                        }

                        div { class: "form-field", style: "margin: 0;",
                            label { class: "form-label", "Responsável pela clínica*" }
                            input {
                                class: "form-input",
                                maxlength: "150",
                                value: "{manager_name}",
                                oninput: move |e| manager_name.set(e.value()),
                            }
                            div { class: "settings-field-counter", "{c_mgr.len()} / 150" }
                        }
                    }
                }

                // Box Lateral de Adicionar Logo
                div { class: "settings-logo-box",
                    IconCamera { size: 28, color: "#94a3b8".to_string() }
                    span { "Adicionar logo" }
                }
            }
        }

        // 2. HORÁRIO DE FUNCIONAMENTO
        div { class: "settings-section",
            h3 { class: "settings-section-title", "Horário de funcionamento" }

            div { style: "display: grid; grid-template-columns: 1fr 1fr 1.5fr; gap: 14px;",
                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Horário inicial da clínica*" }
                    input {
                        class: "form-input",
                        r#type: "number",
                        min: "0",
                        max: "23",
                        value: "{opening_hour}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<u32>() { opening_hour.set(v); }
                        },
                    }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Horário final da clínica*" }
                    input {
                        class: "form-input",
                        r#type: "number",
                        min: "0",
                        max: "23",
                        value: "{closing_hour}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<u32>() { closing_hour.set(v); }
                        },
                    }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Fuso horário*" }
                    select {
                        class: "form-select",
                        value: "{timezone}",
                        onchange: move |e| timezone.set(e.value()),
                        option { value: "Brasilia/São Paulo", "Brasilia/São Paulo" }
                        option { value: "Manaus", "Manaus" }
                        option { value: "Cuiaba", "Cuiabá" }
                        option { value: "Rio Branco", "Rio Branco" }
                    }
                }
            }
        }

        // 3. FISCAL
        div { class: "settings-section",
            h3 { class: "settings-section-title", "Fiscal" }

            div { style: "max-width: 480px;",
                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Emitir recibo em nome de" }
                    select {
                        class: "form-select",
                        value: "{fiscal_issuer}",
                        onchange: move |e| fiscal_issuer.set(e.value()),
                        option { value: "Clínica", "Clínica" }
                        option { value: "Profissional", "Profissional / Dentista" }
                    }
                }
            }
        }

        // 4. INFORMAÇÕES DA CLÍNICA
        div { class: "settings-section",
            div { style: "display: flex; align-items: center; gap: 6px; margin-bottom: 14px;",
                h3 { class: "settings-section-title", style: "margin: 0;", "Informações da clinica" }
                IconInfo { size: 16, color: "#38bdf8".to_string() }
            }

            div { style: "display: grid; grid-template-columns: 1.5fr 1fr 1fr; gap: 14px;",
                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Email*" }
                    input {
                        class: "form-input",
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Telefone" }
                    input {
                        class: "form-input",
                        placeholder: "(11) 0000-0000",
                        value: "{phone}",
                        oninput: move |e| phone.set(e.value()),
                    }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Celular*" }
                    input {
                        class: "form-input",
                        placeholder: "(11) 90000-0000",
                        value: "{cellphone}",
                        oninput: move |e| cellphone.set(e.value()),
                    }
                }
            }

            // Switch Papel Timbrado e Opções
            div { style: "margin-top: 18px;",
                div { style: "display: flex; align-items: center; gap: 8px;",
                    input {
                        r#type: "checkbox",
                        id: "letterhead-switch",
                        checked: *print_letterhead.read(),
                        onchange: move |e| print_letterhead.set(e.value() == "true"),
                    }
                    label {
                        r#for: "letterhead-switch",
                        style: "font-size: 13.5px; font-weight: 700; color: #f1f5f9; cursor: pointer; display: flex; align-items: center; gap: 6px;",
                        span { "Imprimir com papel timbrado" }
                        IconInfo { size: 14, color: "#64748b".to_string() }
                    }
                }

                div { class: "settings-checkboxes-row",
                    label { class: "settings-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: opts.contains(&"Orçamento".to_string()),
                            onchange: move |_| {
                            let mut cur = letterhead_options.read().clone();
                            if cur.iter().any(|s| s == "Orçamento") {
                                cur.retain(|s| s != "Orçamento");
                            } else {
                                cur.push("Orçamento".to_string());
                            }
                            letterhead_options.set(cur);
                        },
                        }
                        span { "Orçamento" }
                    }
                    label { class: "settings-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: opts.contains(&"Evoluções".to_string()),
                            onchange: move |_| {
                            let mut cur = letterhead_options.read().clone();
                            if cur.iter().any(|s| s == "Evoluções") {
                                cur.retain(|s| s != "Evoluções");
                            } else {
                                cur.push("Evoluções".to_string());
                            }
                            letterhead_options.set(cur);
                        },
                        }
                        span { "Evoluções" }
                    }
                    label { class: "settings-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: opts.contains(&"Anamnese".to_string()),
                            onchange: move |_| {
                            let mut cur = letterhead_options.read().clone();
                            if cur.iter().any(|s| s == "Anamnese") {
                                cur.retain(|s| s != "Anamnese");
                            } else {
                                cur.push("Anamnese".to_string());
                            }
                            letterhead_options.set(cur);
                        },
                        }
                        span { "Anamnese" }
                    }
                    label { class: "settings-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: opts.contains(&"Receituários".to_string()),
                            onchange: move |_| {
                            let mut cur = letterhead_options.read().clone();
                            if cur.iter().any(|s| s == "Receituários") {
                                cur.retain(|s| s != "Receituários");
                            } else {
                                cur.push("Receituários".to_string());
                            }
                            letterhead_options.set(cur);
                        },
                        }
                        span { "Receituários" }
                    }
                    label { class: "settings-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: opts.contains(&"Atestados".to_string()),
                            onchange: move |_| {
                            let mut cur = letterhead_options.read().clone();
                            if cur.iter().any(|s| s == "Atestados") {
                                cur.retain(|s| s != "Atestados");
                            } else {
                                cur.push("Atestados".to_string());
                            }
                            letterhead_options.set(cur);
                        },
                        }
                        span { "Atestados" }
                    }
                    label { class: "settings-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: opts.contains(&"Documentos personalizados".to_string()),
                            onchange: move |_| {
                            let mut cur = letterhead_options.read().clone();
                            if cur.iter().any(|s| s == "Documentos personalizados") {
                                cur.retain(|s| s != "Documentos personalizados");
                            } else {
                                cur.push("Documentos personalizados".to_string());
                            }
                            letterhead_options.set(cur);
                        },
                        }
                        span { "Documentos personalizados" }
                    }
                    label { class: "settings-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: opts.contains(&"Controle de prótese".to_string()),
                            onchange: move |_| {
                            let mut cur = letterhead_options.read().clone();
                            if cur.iter().any(|s| s == "Controle de prótese") {
                                cur.retain(|s| s != "Controle de prótese");
                            } else {
                                cur.push("Controle de prótese".to_string());
                            }
                            letterhead_options.set(cur);
                        },
                        }
                        span { "Controle de prótese" }
                    }
                    label { class: "settings-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: opts.contains(&"Recibos".to_string()),
                            onchange: move |_| {
                            let mut cur = letterhead_options.read().clone();
                            if cur.iter().any(|s| s == "Recibos") {
                                cur.retain(|s| s != "Recibos");
                            } else {
                                cur.push("Recibos".to_string());
                            }
                            letterhead_options.set(cur);
                        },
                        }
                        span { "Recibos" }
                    }
                }
            }
        }

        // 5. LOCALIZAÇÃO
        div { class: "settings-section",
            h3 { class: "settings-section-title", "Localização" }

            div { style: "display: grid; grid-template-columns: 1fr 2fr 0.8fr 1fr; gap: 14px; margin-bottom: 14px;",
                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "CEP" }
                    input {
                        class: "form-input",
                        placeholder: "00000-000",
                        value: "{addr.zip_code}",
                        oninput: move |e| {
                            let mut a = address.read().clone();
                            a.zip_code = e.value();
                            address.set(a);
                        },
                    }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Rua" }
                    input {
                        class: "form-input",
                        maxlength: "120",
                        value: "{addr.street}",
                        oninput: move |e| {
                            let mut a = address.read().clone();
                            a.street = e.value();
                            address.set(a);
                        },
                    }
                    div { class: "settings-field-counter", "{addr.street.len()} / 120" }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Número" }
                    input {
                        class: "form-input",
                        maxlength: "5",
                        value: "{addr.number}",
                        oninput: move |e| {
                            let mut a = address.read().clone();
                            a.number = e.value();
                            address.set(a);
                        },
                    }
                    div { class: "settings-field-counter", "{addr.number.len()} / 5" }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Complemento" }
                    input {
                        class: "form-input",
                        maxlength: "255",
                        value: if let Some(ref c) = addr.complement { "{c}" } else { "" },
                        oninput: move |e| {
                            let mut a = address.read().clone();
                            a.complement = Some(e.value());
                            address.set(a);
                        },
                    }
                    div { class: "settings-field-counter", "{addr.complement.as_ref().map(|c| c.len()).unwrap_or(0)} / 255" }
                }
            }

            div { style: "display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 14px;",
                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Bairro" }
                    input {
                        class: "form-input",
                        maxlength: "128",
                        value: "{addr.neighborhood}",
                        oninput: move |e| {
                            let mut a = address.read().clone();
                            a.neighborhood = e.value();
                            address.set(a);
                        },
                    }
                    div { class: "settings-field-counter", "{addr.neighborhood.len()} / 128" }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Cidade" }
                    input {
                        class: "form-input",
                        maxlength: "50",
                        value: "{addr.city}",
                        oninput: move |e| {
                            let mut a = address.read().clone();
                            a.city = e.value();
                            address.set(a);
                        },
                    }
                    div { class: "settings-field-counter", "{addr.city.len()} / 50" }
                }

                div { class: "form-field", style: "margin: 0;",
                    label { class: "form-label", "Estado" }
                    select {
                        class: "form-select",
                        value: "{addr.state}",
                        onchange: move |e| {
                            let mut a = address.read().clone();
                            a.state = e.value();
                            address.set(a);
                        },
                        option { value: "SP", "São Paulo" }
                        option { value: "RJ", "Rio de Janeiro" }
                        option { value: "MG", "Minas Gerais" }
                        option { value: "RS", "Rio Grande do Sul" }
                        option { value: "PR", "Paraná" }
                        option { value: "SC", "Santa Catarina" }
                        option { value: "BA", "Bahia" }
                        option { value: "PE", "Pernambuco" }
                        option { value: "CE", "Ceará" }
                        option { value: "GO", "Goiás" }
                        option { value: "DF", "Distrito Federal" }
                    }
                }
            }
        }

        // BARRA FIXA INFERIOR
        div { class: "settings-bottom-actions",
            button {
                r#type: "button",
                class: "settings-btn-save",
                onclick: move |_| on_save.call(()),
                "SALVAR"
            }
        }
    }
}
