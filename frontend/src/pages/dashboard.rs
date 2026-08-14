use crate::api;
use crate::components::sidebar::Sidebar;
use crate::components::topbar::Topbar;
use crate::router::Route;
use crate::{ActiveClinicState, SessionState};
use base64::{Engine as _, engine::general_purpose};
use dioxus::prelude::*;
use shared::clinics::{ClinicResponse, UpdateClinicRequest};
use shared::files::FileUploadRequest;

#[component]
pub fn DashboardLayout() -> Element {
    let mut session = consume_context::<Signal<SessionState>>();
    let mut active_clinic = consume_context::<Signal<ActiveClinicState>>();
    let navigator = use_navigator();

    let mut is_collapsed = use_signal(|| false);
    let mut is_settings_open = use_signal(|| false);
    let mut active_tab = use_signal(|| "Perfil".to_string());

    if session().is_none() || active_clinic().is_none() {
        spawn(async move {
            navigator.replace(Route::LoginScreen {});
        });
        return rsx! { div {} };
    }

    let clinic = active_clinic().as_ref().unwrap().clone();
    let user_name = session().as_ref().unwrap().full_name.clone();

    let collapsed_val = is_collapsed();
    let settings_open_val = is_settings_open();
    let tab_val = active_tab();

    rsx! {
        div { class: "dashboard-layout",
            Sidebar {
                theme_color: clinic.theme_color.clone(),
                logo_url: clinic.logo_url.clone(),
                is_collapsed: collapsed_val,
                on_toggle: move |_| is_collapsed.set(!is_collapsed()),
                on_settings: move |_| is_settings_open.set(true),
                on_logout: move |_| {
                    active_clinic.set(None);
                    session.set(None);
                }
            }

            div { class: "main-area",
                Topbar { user_name: user_name }

                div { class: "content-wrapper",
                    Outlet::<Route> {}
                }
            }

            if settings_open_val {
                div { class: "modal-overlay",
                    div { class: "settings-modal",


                        div { class: "settings-header",
                            h2 { class: "settings-title", "Configurações da Clínica" }
                            button {
                                class: "close-btn",
                                onclick: move |_| is_settings_open.set(false),
                                "×"
                            }
                        }


                        div { class: "settings-body",


                            div { class: "settings-tabs-sidebar",
                                for tab in ["Perfil", "Identidade Visual", "WhatsApp"] {
                                    button {
                                        class: if tab_val == tab { "settings-tab-btn active" } else { "settings-tab-btn" },
                                        onclick: move |_| active_tab.set(tab.to_string()),
                                        "{tab}"
                                    }
                                }
                            }

                            // Conteúdo
                            div { class: "settings-content",
                                match tab_val.as_str() {
                                    "Perfil" => rsx! { ProfileTab { clinic_id: clinic.clinic_id.clone() } },
                                    "Identidade Visual" => rsx! { BrandingTab { clinic_id: clinic.clinic_id.clone() } },
                                    "WhatsApp" => rsx! { WhatsAppTab { clinic_id: clinic.clinic_id.clone() } },
                                    _ => rsx! { div {} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProfileTab(clinic_id: String) -> Element {
    let mut is_saving = use_signal(|| false);

    let id_for_resource = clinic_id.clone();
    let clinic_resource = use_resource(move || {
        let id = id_for_resource.clone();
        async move { api::fetch_clinic(&id).await.unwrap() }
    });

    match clinic_resource.read().as_ref() {
        None => rsx! { div { "Carregando dados..." } },
        Some(clinic_data) => {
            let mut trading_name = use_signal(|| clinic_data.trading_name.clone());
            let mut corporate_name = use_signal(|| clinic_data.corporate_name.clone());
            let mut document_cnpj = use_signal(|| clinic_data.document_cnpj.clone());

            // 2. Cópia segura para o botão de salvar
            let id_for_save = clinic_id.clone();
            let handle_save = move |_| {
                is_saving.set(true);
                let id = id_for_save.clone();

                spawn(async move {
                    let _ = api::update_clinic(
                        &id,
                        UpdateClinicRequest {
                            trading_name: Some(trading_name()),
                            corporate_name: Some(corporate_name()),
                            document_cnpj: Some(document_cnpj()),
                            theme_color: None,
                            address: None,
                        },
                    )
                    .await;
                    is_saving.set(false);
                });
            };

            rsx! {
                div { class: "tab-pane",
                    h3 { "Dados Cadastrais" }
                    div { class: "form-grid",
                        div { class: "input-group-wrapper",
                            label { "Nome Fantasia" }
                            input { class: "modern-input-field", value: "{trading_name}", oninput: move |e| trading_name.set(e.value()) }
                        }
                        div { class: "input-group-wrapper",
                            label { "Razão Social" }
                            input { class: "modern-input-field", value: "{corporate_name}", oninput: move |e| corporate_name.set(e.value()) }
                        }
                        div { class: "input-group-wrapper full-width",
                            label { "CNPJ" }
                            input { class: "modern-input-field", value: "{document_cnpj}", oninput: move |e| document_cnpj.set(e.value()) }
                        }
                    }
                    button {
                        class: "btn-primary",
                        onclick: handle_save,
                        disabled: is_saving(),
                        if is_saving() { "Salvando..." } else { "Salvar Alterações" }
                    }
                }
            }
        }
    }
}

#[component]
fn BrandingTab(clinic_id: String) -> Element {
    let mut active_clinic = consume_context::<Signal<ActiveClinicState>>();
    let mut is_uploading = use_signal(|| false);

    let current_color = active_clinic().as_ref().unwrap().theme_color.clone();
    let current_logo = active_clinic().as_ref().unwrap().logo_url.clone();

    let id_for_color = clinic_id.clone();
    let on_color_change = move |e: FormEvent| {
        let new_color = e.value();
        if let Some(mut clinic) = active_clinic() {
            clinic.theme_color = new_color.clone();
            active_clinic.set(Some(clinic));
        }

        let id = id_for_color.clone();
        spawn(async move {
            let _ = api::update_clinic(
                &id,
                UpdateClinicRequest {
                    theme_color: Some(new_color),
                    trading_name: None,
                    corporate_name: None,
                    document_cnpj: None,
                    address: None,
                },
            )
            .await;
        });
    };

    let id_for_upload = clinic_id.clone();
    let on_file_drop = move |evt: FormEvent| {
        for file in evt.files() {
            is_uploading.set(true);
            let id = id_for_upload.clone();

            spawn(async move {
                if let Ok(bytes) = file.read_bytes().await {
                    let base64_content = general_purpose::STANDARD.encode(&bytes);
                    let req = FileUploadRequest {
                        filename: file.name(),
                        mime_type: "image/png".into(),
                        base64_content,
                    };

                    if let Ok(new_url) = api::upload_clinic_logo(&id, req).await {
                        if let Some(mut clinic) = active_clinic() {
                            clinic.logo_url = Some(new_url);
                            active_clinic.set(Some(clinic));
                        }
                    }
                }
                is_uploading.set(false);
            });
        }
    };

    rsx! {
        div { class: "tab-pane",
            h3 { "Identidade Visual" }
            div { class: "branding-grid",

                div {
                    label { class: "branding-label", "Cor Principal do Sistema" }
                    div { class: "color-picker-wrapper",
                        input {
                            class: "color-input",
                            r#type: "color",
                            value: "{current_color}",
                            onchange: on_color_change
                        }
                        span { class: "color-hex", "{current_color}" }
                    }
                    p { class: "branding-hint", "Altere para ver a Sidebar mudando em tempo real." }
                }

                div {
                    label { class: "branding-label", "Logo da Unidade" }
                    div { class: "logo-upload-wrapper",
                        div { class: "logo-preview",
                            match current_logo {
                                Some(url) => rsx! { img { src: "{url}" } },
                                None => rsx! { span { "Sem Logo" } }
                            }
                        }
                        label {
                            class: "btn-secondary",
                            if is_uploading() { "Enviando..." } else { "Trocar Logo" }
                            input { r#type: "file", accept: "image/png, image/jpeg", style: "display: none;", onchange: on_file_drop }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WhatsAppTab(clinic_id: String) -> Element {
    let mut qr_code = use_signal(|| None::<String>);
    let mut is_loading_qr = use_signal(|| false);

    let id_for_qr = clinic_id.clone();
    let handle_connect = move |_| {
        is_loading_qr.set(true);
        let id = id_for_qr.clone();

        spawn(async move {
            if let Ok(qr_base64) = api::fetch_whatsapp_qr_code(&id).await {
                qr_code.set(Some(qr_base64));
            }
            is_loading_qr.set(false);
        });
    };

    rsx! {
        div { class: "tab-pane qr-container",
            h3 { "Conexão com WhatsApp" }
            p { class: "qr-description", "Conecte o WhatsApp para habilitar automações de mensagens." }

            match qr_code() {
                Some(base64_str) => rsx! {
                    div { class: "qr-code-wrapper",
                        img { class: "qr-code-image", src: "data:image/png;base64,{base64_str}" }
                        p { class: "qr-status", "Aguardando leitura do QR Code..." }
                    }
                },
                None => rsx! {
                    div { class: "qr-placeholder",
                        button {
                            class: "btn-primary",
                            onclick: handle_connect,
                            disabled: is_loading_qr(),
                            if is_loading_qr() { "Gerando..." } else { "Gerar QR Code" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AdvancedTab(clinic_id: String) -> Element {
    let mut auto_reminders = use_signal(|| true);
    let mut require_esign = use_signal(|| true);
    let mut webhook_url = use_signal(|| "".to_string());

    rsx! {
        div { class: "tab-pane",
            h3 { style: "margin-top: 0;", "Configurações Avançadas e Automação" }
            p { style: "color: #64748b; margin-bottom: 24px;", "Gerencie comportamentos globais do servidor, integrações e segurança da clínica." }

            // Bloco de Automações
            div { class: "advanced-setting-row",
                div { class: "advanced-setting-info",
                    h4 { "Lembretes Automáticos de Consulta" }
                    p { "Dispara mensagens no WhatsApp 24h antes do agendamento." }
                }
                div { class: "toggle-switch",
                    input {
                        r#type: "checkbox",
                        checked: auto_reminders(),
                        onchange: move |e| auto_reminders.set(e.checked())
                    }
                }
            }

            // Bloco de Segurança/Documentos
            div { class: "advanced-setting-row",
                div { class: "advanced-setting-info",
                    h4 { "Exigir Assinatura Digital (E-Sign)" }
                    p { "Bloqueia o início do tratamento até que os contratos sejam assinados via OTP." }
                }
                div { class: "toggle-switch",
                    input {
                        r#type: "checkbox",
                        checked: require_esign(),
                        onchange: move |e| require_esign.set(e.checked())
                    }
                }
            }

            // Bloco de Integração (Webhooks)
            div { style: "margin-top: 32px;",
                h4 { style: "color: #0f172a; margin-bottom: 8px;", "Webhook de Integração" }
                p { style: "color: #64748b; font-size: 13px; margin-bottom: 16px;", "Envie eventos (como novo paciente ou pagamento aprovado) para sistemas externos." }
                div { class: "input-group-wrapper",
                    input {
                        class: "modern-input-field",
                        placeholder: "https://seu-sistema.com.br/api/webhook",
                        value: "{webhook_url}",
                        oninput: move |e| webhook_url.set(e.value())
                    }
                }
            }


            div { style: "margin-top: 48px; padding: 24px; border: 1px solid #fecaca; border-radius: 12px; background: #fef2f2;",
                h4 { style: "color: #ef4444; margin: 0 0 8px 0;", "Zona de Perigo" }
                p { style: "color: #991b1b; font-size: 13px; margin-bottom: 16px;", "Ações irreversíveis relacionadas ao banco de dados desta clínica." }
                button {
                    style: "background: #ef4444; color: white; border: none; padding: 10px 16px; border-radius: 6px; font-weight: 500; cursor: pointer;",
                    onclick: move |_| { },
                    "Encerrar e Apagar Clínica"
                }
            }
        }
    }
}
