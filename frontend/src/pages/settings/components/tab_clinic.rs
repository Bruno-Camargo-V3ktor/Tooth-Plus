use crate::icons::{IconCamera, IconCheck, IconInfo};
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
        div { style: "display: flex; flex-direction: column; gap: 24px;",
            // 1. DADOS DA CLÍNICA
            div { class: "settings-card",
                div { class: "settings-card-header",
                    div {
                        h3 { class: "settings-card-title", "Identificação da Clínica" }
                        p { class: "settings-card-desc", "Informações principais exibidas para pacientes e em documentos clínicos." }
                    }
                    button {
                        r#type: "button",
                        class: "settings-btn-save",
                        onclick: move |_| on_save.call(()),
                        IconCheck { size: 16, color: "#ffffff".to_string() }
                        span { "Salvar Alterações" }
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 160px; gap: 24px; align-items: flex-start;",
                    div { style: "display: flex; flex-direction: column; gap: 16px;",
                        div { style: "display: grid; grid-template-columns: 2fr 1fr; gap: 16px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Nome da clínica *" }
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

                        div { style: "display: grid; grid-template-columns: 1fr 1.5fr; gap: 16px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Nome utilizado nas comunicações *" }
                                input {
                                    class: "form-input",
                                    maxlength: "30",
                                    value: "{comm_name}",
                                    oninput: move |e| comm_name.set(e.value()),
                                }
                                div { class: "settings-field-counter", "{c_comm.len()} / 30" }
                            }

                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Responsável técnico pela clínica *" }
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

                    // Box Lateral de Logotipo
                    div { class: "settings-logo-box",
                        IconCamera { size: 32, color: "var(--primary, #00a0e4)".to_string() }
                        span { "Logotipo da Clínica" }
                    }
                }
            }

            // 2. HORÁRIOS & CONFIGURAÇÕES FISCAIS
            div { class: "settings-card",
                div { class: "settings-card-header",
                    div {
                        h3 { class: "settings-card-title", "Funcionamento & Emissão Fiscal" }
                        p { class: "settings-card-desc", "Defina as faixas de atendimento da agenda e regras para recibos." }
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 1fr 1.5fr 1.5fr; gap: 16px;",
                    div { class: "form-field", style: "margin: 0;",
                        label { class: "form-label", "Horário inicial da agenda *" }
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
                        label { class: "form-label", "Horário final da agenda *" }
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
                        label { class: "form-label", "Fuso horário *" }
                        select {
                            class: "form-select",
                            value: "{timezone}",
                            onchange: move |e| timezone.set(e.value()),
                            option { value: "Brasilia/São Paulo", "Brasilia/São Paulo (UTC-3)" }
                            option { value: "Manaus", "Manaus (UTC-4)" }
                            option { value: "Cuiaba", "Cuiabá (UTC-4)" }
                            option { value: "Rio Branco", "Rio Branco (UTC-5)" }
                        }
                    }

                    div { class: "form-field", style: "margin: 0;",
                        label { class: "form-label", "Emitir recibo em nome de" }
                        select {
                            class: "form-select",
                            value: "{fiscal_issuer}",
                            onchange: move |e| fiscal_issuer.set(e.value()),
                            option { value: "Clínica", "Clínica (Pessoa Jurídica)" }
                            option { value: "Profissional", "Dentista Responsável (Pessoa Física)" }
                        }
                    }
                }
            }

            // 3. CONTATOS & PAPEL TIMBRADO
            div { class: "settings-card",
                div { class: "settings-card-header",
                    div {
                        h3 { class: "settings-card-title", "Contatos & Personalização de Impressão" }
                        p { class: "settings-card-desc", "Contatos institucionais e documentos onde o cabeçalho timbrado é aplicado." }
                    }
                }

                div { style: "display: grid; grid-template-columns: 1.5fr 1fr 1fr; gap: 16px;",
                    div { class: "form-field", style: "margin: 0;",
                        label { class: "form-label", "E-mail de Contato *" }
                        input {
                            class: "form-input",
                            r#type: "email",
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                        }
                    }

                    div { class: "form-field", style: "margin: 0;",
                        label { class: "form-label", "Telefone Fixo" }
                        input {
                            class: "form-input",
                            placeholder: "(11) 0000-0000",
                            value: "{phone}",
                            oninput: move |e| phone.set(e.value()),
                        }
                    }

                    div { class: "form-field", style: "margin: 0;",
                        label { class: "form-label", "WhatsApp / Celular *" }
                        input {
                            class: "form-input",
                            placeholder: "(11) 90000-0000",
                            value: "{cellphone}",
                            oninput: move |e| cellphone.set(e.value()),
                        }
                    }
                }

                // Switch Papel Timbrado e Checkboxes
                div { style: "margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, rgba(255,255,255,0.06));",
                    div { style: "display: flex; align-items: center; gap: 10px; margin-bottom: 12px;",
                        input {
                            r#type: "checkbox",
                            id: "letterhead-switch",
                            checked: *print_letterhead.read(),
                            onchange: move |e| print_letterhead.set(e.value() == "true"),
                        }
                        label {
                            r#for: "letterhead-switch",
                            style: "font-size: 14px; font-weight: 700; color: var(--text-main, #f8fafc); cursor: pointer; display: flex; align-items: center; gap: 6px;",
                            span { "Imprimir cabeçalho e papel timbrado nos documentos" }
                            IconInfo { size: 14, color: "var(--text-muted, #94a3b8)".to_string() }
                        }
                    }

                    div { class: "settings-checkboxes-grid",
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
                            span { "Orçamentos e Planos" }
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
                            span { "Evoluções Clínicas" }
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
                            span { "Fichas de Anamnese" }
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
                            span { "Receituários e Prescrições" }
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
                            span { "Atestados Odontológicos" }
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
                            span { "Contratos e Modelos" }
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
                            span { "Ordens de Prótese" }
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
                            span { "Recibos e Comprovantes" }
                        }
                    }
                }
            }

            // 4. ENDEREÇO & LOCALIZAÇÃO
            div { class: "settings-card",
                div { class: "settings-card-header",
                    div {
                        h3 { class: "settings-card-title", "Localização da Clínica" }
                        p { class: "settings-card-desc", "Endereço completo exibido no cabeçalho e rodapé dos impressos." }
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 2fr 0.8fr 1fr; gap: 16px; margin-bottom: 16px;",
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
                        label { class: "form-label", "Logradouro / Rua" }
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
                    }
                }

                div { style: "display: grid; grid-template-columns: 1.2fr 1.2fr 1fr; gap: 16px;",
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
                    }

                    div { class: "form-field", style: "margin: 0;",
                        label { class: "form-label", "Estado (UF)" }
                        select {
                            class: "form-select",
                            value: "{addr.state}",
                            onchange: move |e| {
                                let mut a = address.read().clone();
                                a.state = e.value();
                                address.set(a);
                            },
                            option { value: "SP", "São Paulo (SP)" }
                            option { value: "RJ", "Rio de Janeiro (RJ)" }
                            option { value: "MG", "Minas Gerais (MG)" }
                            option { value: "RS", "Rio Grande do Sul (RS)" }
                            option { value: "PR", "Paraná (PR)" }
                            option { value: "SC", "Santa Catarina (SC)" }
                            option { value: "BA", "Bahia (BA)" }
                            option { value: "PE", "Pernambuco (PE)" }
                            option { value: "CE", "Ceará (CE)" }
                            option { value: "GO", "Goiás (GO)" }
                            option { value: "DF", "Distrito Federal (DF)" }
                        }
                    }
                }
            }
        }
    }
}
