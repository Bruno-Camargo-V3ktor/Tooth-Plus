use crate::api;
use crate::components::icons::{IconBuilding, IconFlask, IconRefresh, IconSettings};
use crate::components::sidebar::Sidebar;
use crate::components::topbar::Topbar;
use crate::permissions;
use crate::router::Route;
use crate::{ActiveClinicState, SessionState};
use base64::{Engine as _, engine::general_purpose};
use dioxus::prelude::*;
use shared::clinics::UpdateClinicRequest;
use shared::files::FileUploadRequest;
use std::time::Duration;

#[component]
pub fn DashboardLayout() -> Element {
    let mut session = consume_context::<Signal<SessionState>>();
    let mut active_clinic = consume_context::<Signal<ActiveClinicState>>();
    let navigator = use_navigator();

    let mut is_collapsed = use_signal(|| false);
    let mut is_settings_open = use_signal(|| false);
    let mut active_tab = use_signal(|| "Perfil".to_string());
    let mut error_toast = use_signal(|| None::<String>);

    use_effect(move || {
        if error_toast().is_some() {
            spawn(async move {
                gloo_timers::future::sleep(Duration::from_secs(5)).await;
                error_toast.set(None);
            });
        }
    });

    if session().is_none() || active_clinic().is_none() {
        spawn(async move {
            navigator.replace(Route::LoginScreen {});
        });
        return rsx! { div {} };
    }

    let sess = session();
    let clinic = active_clinic();

    let can_read_clinics = permissions::has_permission(&sess, &clinic, "clinics:read");
    let can_write_clinics = permissions::has_permission(&sess, &clinic, "clinics:write");
    let can_read_wpp = permissions::has_permission(&sess, &clinic, "whatsapp:read");
    let can_write_wpp = permissions::has_permission(&sess, &clinic, "whatsapp:write");
    let can_read_adv = permissions::has_permission(&sess, &clinic, "advanced:read");
    let can_write_adv = permissions::has_permission(&sess, &clinic, "advanced:write");
    let can_delete = permissions::has_permission(&sess, &clinic, "clinics:delete");
    let can_read_users = permissions::has_permission(&sess, &clinic, "users:read");
    let can_read_finance = permissions::has_permission(&sess, &clinic, "finance:read_all")
        || permissions::has_permission(&sess, &clinic, "finance:read_income")
        || permissions::has_permission(&sess, &clinic, "finance:read_expense")
        || permissions::has_permission(&sess, &clinic, "finance:read_pending")
        || permissions::has_permission(&sess, &clinic, "finance:read");
    let can_read_agenda = permissions::has_any_permission(
        &sess,
        &clinic,
        &["appointments:read", "agenda:read"],
    );
    let can_read_patients = permissions::has_permission(&sess, &clinic, "patients:read");
    let can_read_stock = permissions::has_permission(&sess, &clinic, "stock:read");
    let can_read_documents = permissions::has_permission(&sess, &clinic, "documents:read");
    let can_see_settings = can_read_clinics || can_read_wpp || can_read_adv;

    let clinic_data = clinic.clone().unwrap();
    let session_data = sess.clone().unwrap();
    let clinic_id = clinic_data.clinic_id.clone();
    let token = session_data.token.clone();

    let collapsed_val = is_collapsed();
    let settings_open_val = is_settings_open();
    let tab_val = active_tab();

    rsx! {
        div {
            class: "dashboard-layout",
            style: "--clinic-primary: {clinic_data.theme_color};",

            if let Some(err_msg) = error_toast() {
                div { class: "toast-error",
                    span { "{err_msg}" }
                    button { class: "toast-close-btn", onclick: move |_| error_toast.set(None), "×" }
                }
            }

            Sidebar {
                theme_color: clinic_data.theme_color.clone(),
                logo_url: clinic_data.logo_url.clone(),
                is_collapsed: collapsed_val,
                can_see_agenda: can_read_agenda,
                can_see_patients: can_read_patients,
                can_see_finance: can_read_finance,
                can_see_stock: can_read_stock,
                can_see_documents: can_read_documents,
                can_see_users: can_read_users,
                can_see_settings,
                on_toggle: move |_| is_collapsed.set(!is_collapsed()),
                on_settings: move |_| is_settings_open.set(true),
                on_logout: move |_| { crate::utils::clear_session(); active_clinic.set(None); session.set(None); }
            }

            div { class: "main-area",
                Topbar { user_name: session_data.full_name.clone() }
                div { class: "content-wrapper", Outlet::<Route> {} }
            }

            if settings_open_val {
                div {
                    class: "modal-overlay",
                    onclick: move |_| is_settings_open.set(false),
                    div {
                        class: "settings-modal",
                        onclick: move |e| e.stop_propagation(),

                        div { class: "settings-header",
                            h2 { class: "settings-title", "Configurações da Clínica" }
                            button { class: "close-btn", onclick: move |_| is_settings_open.set(false), "×" }
                        }

                        div { class: "settings-body",
                            div { class: "settings-tabs-sidebar",
                                if can_read_clinics {
                                    button {
                                        class: if tab_val == "Perfil" { "settings-tab-btn active" } else { "settings-tab-btn" },
                                        onclick: move |_| active_tab.set("Perfil".to_string()),
                                        IconBuilding { size: 16, color: "currentColor".to_string() }
                                        span { "Perfil" }
                                    }
                                    button {
                                        class: if tab_val == "Identidade Visual" { "settings-tab-btn active" } else { "settings-tab-btn" },
                                        onclick: move |_| active_tab.set("Identidade Visual".to_string()),
                                        IconFlask { size: 16, color: "currentColor".to_string() }
                                        span { "Identidade Visual" }
                                    }
                                }
                                if can_read_wpp {
                                    button {
                                        class: if tab_val == "WhatsApp" { "settings-tab-btn active" } else { "settings-tab-btn" },
                                        onclick: move |_| active_tab.set("WhatsApp".to_string()),
                                        IconRefresh { size: 16, color: "currentColor".to_string() }
                                        span { "WhatsApp" }
                                    }
                                }
                                if can_read_adv {
                                    button {
                                        class: if tab_val == "Avançado" { "settings-tab-btn active" } else { "settings-tab-btn" },
                                        onclick: move |_| active_tab.set("Avançado".to_string()),
                                        IconSettings { size: 16, color: "currentColor".to_string() }
                                        span { "Avançado" }
                                    }
                                }
                            }

                            div { class: "settings-content",
                                match tab_val.as_str() {
                                    "Perfil" if can_read_clinics => rsx! {
                                        ProfileTab {
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            error_toast,
                                            can_write: can_write_clinics
                                        }
                                    },
                                    "Identidade Visual" if can_read_clinics => rsx! {
                                        BrandingTab {
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            error_toast,
                                            can_write: can_write_clinics
                                        }
                                    },
                                    "WhatsApp" if can_read_wpp => rsx! {
                                        WhatsAppTab {
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            error_toast,
                                            can_write: can_write_wpp
                                        }
                                    },
                                    "Avançado" if can_read_adv => rsx! {
                                        AdvancedTab {
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            error_toast,
                                            can_write: can_write_adv,
                                            can_delete
                                        }
                                    },
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
fn ProfileTab(
    clinic_id: String,
    token: String,
    mut error_toast: Signal<Option<String>>,
    can_write: bool,
) -> Element {
    let mut is_saving = use_signal(|| false);

    let id_res = clinic_id.clone();
    let t_res = token.clone();
    let mut et = error_toast;

    let clinic_resource = use_resource(move || {
        let id = id_res.clone();
        let t = t_res.clone();
        async move {
            let res = api::fetch_clinic(&t, &id).await;
            if let Err(ref e) = res {
                et.set(Some(e.clone()));
            }
            res
        }
    });

    match clinic_resource.read().as_ref() {
        None => rsx! { div { "Carregando dados..." } },
        Some(Err(_)) => {
            rsx! { div { class: "error-state-friendly", "Não foi possível carregar as informações." } }
        }
        Some(Ok(data)) => {
            let mut trading_name = use_signal(|| data.trading_name.clone());
            let mut corporate_name = use_signal(|| data.corporate_name.clone());
            let mut document_cnpj = use_signal(|| data.document_cnpj.clone());

            let id_sv = clinic_id.clone();
            let t_sv = token.clone();
            let handle_save = move |_| {
                if !can_write {
                    return;
                }
                is_saving.set(true);
                let id = id_sv.clone();
                let t = t_sv.clone();
                spawn(async move {
                    let req = UpdateClinicRequest {
                        trading_name: Some(trading_name()),
                        corporate_name: Some(corporate_name()),
                        document_cnpj: Some(document_cnpj()),
                        theme_color: None,
                        address: None,
                        auto_reminders: None,
                        require_esign: None,
                        ..Default::default()
                    };
                    if let Err(msg) = api::update_clinic(&t, &id, req).await {
                        error_toast.set(Some(msg));
                    }
                    is_saving.set(false);
                });
            };

            rsx! {
                div { class: "settings-pane-container",
                    h3 { class: "settings-pane-title", "Dados Cadastrais" }
                    div { class: "form-row-2",
                        div { class: "form-group",
                            label { class: "form-label", "Nome Fantasia" }
                            input { class: "modern-input-field", disabled: !can_write, value: "{trading_name}", oninput: move |e| trading_name.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Razão Social" }
                            input { class: "modern-input-field", disabled: !can_write, value: "{corporate_name}", oninput: move |e| corporate_name.set(e.value()) }
                        }
                    }
                    div { class: "form-group",
                        label { class: "form-label", "CNPJ" }
                        input { class: "modern-input-field", disabled: !can_write, value: "{document_cnpj}", oninput: move |e| document_cnpj.set(e.value()) }
                    }
                    div { class: "settings-action-bar",
                        if can_write {
                            button { class: "btn-primary", onclick: handle_save, disabled: is_saving(),
                                if is_saving() { "Salvando..." } else { "Salvar Alterações" }
                            }
                        } else {
                            p { class: "text-error-small", "Permissão de leitura apenas." }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BrandingTab(
    clinic_id: String,
    token: String,
    mut error_toast: Signal<Option<String>>,
    can_write: bool,
) -> Element {
    let mut active_clinic = consume_context::<Signal<ActiveClinicState>>();
    let mut is_uploading = use_signal(|| false);

    let current_color = active_clinic().as_ref().unwrap().theme_color.clone();
    let current_logo = active_clinic().as_ref().unwrap().logo_url.clone();

    let id_col = clinic_id.clone();
    let t_col = token.clone();
    let on_color_change = move |e: FormEvent| {
        if !can_write {
            return;
        }
        let new_color = e.value();
        if let Some(mut clinic) = active_clinic() {
            clinic.theme_color = new_color.clone();
            active_clinic.set(Some(clinic));
        }
        let id = id_col.clone();
        let t = t_col.clone();
        spawn(async move {
            let req = UpdateClinicRequest {
                theme_color: Some(new_color),
                trading_name: None,
                corporate_name: None,
                document_cnpj: None,
                address: None,
                auto_reminders: None,
                require_esign: None,
                ..Default::default()
            };
            if let Err(msg) = api::update_clinic(&t, &id, req).await {
                error_toast.set(Some(msg));
            }
        });
    };

    let id_up = clinic_id.clone();
    let t_up = token.clone();
    let on_file_drop = move |evt: FormEvent| {
        if !can_write {
            return;
        }
        for file in evt.files() {
            is_uploading.set(true);
            let id = id_up.clone();
            let t = t_up.clone();
            spawn(async move {
                if let Ok(bytes) = file.read_bytes().await {
                    let base64_content = general_purpose::STANDARD.encode(&bytes);
                    let req = FileUploadRequest {
                        filename: file.name(),
                        mime_type: "image/png".into(),
                        base64_content,
                        ..Default::default()
                    };
                    match api::upload_clinic_logo(&t, &id, req).await {
                        Ok(new_url) => {
                            if let Some(mut clinic) = active_clinic() {
                                clinic.logo_url = Some(new_url);
                                active_clinic.set(Some(clinic));
                            }
                        }
                        Err(msg) => error_toast.set(Some(msg)),
                    }
                }
                is_uploading.set(false);
            });
        }
    };

    rsx! {
        div { class: "settings-pane-container",
            h3 { class: "settings-pane-title", "Identidade Visual" }
            div { class: "branding-grid",
                div { class: "form-group",
                    label { class: "form-label", "Cor Principal" }
                    div { class: "color-picker-wrapper",
                        input { class: "color-input", r#type: "color", disabled: !can_write, value: "{current_color}", onchange: on_color_change }
                        span { class: "color-hex", "{current_color}" }
                    }
                }
                div { class: "form-group",
                    label { class: "form-label", "Logo da Unidade" }
                    div { class: "logo-upload-wrapper",
                        div { class: "logo-preview",
                            match current_logo {
                                Some(url) => rsx! { img { src: "{url}" } },
                                None => rsx! { span { "Sem Logo" } }
                            }
                        }
                        if can_write {
                            label { class: "btn-secondary",
                                if is_uploading() { "Enviando..." } else { "Trocar Logo" }
                                input { class: "hidden-input", r#type: "file", accept: "image/png, image/jpeg", onchange: on_file_drop }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WhatsAppTab(
    clinic_id: String,
    token: String,
    mut error_toast: Signal<Option<String>>,
    can_write: bool,
) -> Element {
    let mut qr_code = use_signal(|| None::<String>);
    let mut is_loading_qr = use_signal(|| false);

    let id_qr = clinic_id.clone();
    let t_qr = token.clone();
    let handle_connect = move |_| {
        is_loading_qr.set(true);
        let id = id_qr.clone();
        let t = t_qr.clone();
        spawn(async move {
            match api::fetch_whatsapp_qr_code(&t, &id).await {
                Ok(qr) => qr_code.set(Some(qr)),
                Err(msg) => error_toast.set(Some(msg)),
            }
            is_loading_qr.set(false);
        });
    };

    rsx! {
        div { class: "settings-pane-container",
            h3 { class: "settings-pane-title", "Conexão com WhatsApp" }
            p { class: "qr-description", "Habilite automações de mensagens e lembretes aos pacientes." }
            match qr_code() {
                Some(base64_str) => rsx! {
                    div { class: "qr-code-wrapper",
                        img { class: "qr-code-image", src: "data:image/png;base64,{base64_str}" }
                        p { class: "qr-status", "Aguardando leitura do QR Code..." }
                    }
                },
                None => rsx! {
                    div { class: "qr-placeholder",
                        if can_write {
                            button { class: "btn-primary", onclick: handle_connect, disabled: is_loading_qr(),
                                if is_loading_qr() { "Gerando..." } else { "Gerar QR Code" }
                            }
                        } else {
                            p { class: "text-error-small", "Apenas gestores podem gerar sessão WPP." }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AdvancedTab(
    clinic_id: String,
    token: String,
    mut error_toast: Signal<Option<String>>,
    can_write: bool,
    can_delete: bool,
) -> Element {
    let mut is_saving = use_signal(|| false);

    let id_res = clinic_id.clone();
    let t_res = token.clone();
    let mut et = error_toast;

    let clinic_resource = use_resource(move || {
        let id = id_res.clone();
        let t = t_res.clone();
        async move {
            let res = api::fetch_clinic(&t, &id).await;
            if let Err(ref e) = res {
                et.set(Some(e.clone()));
            }
            res
        }
    });

    match clinic_resource.read().as_ref() {
        None => rsx! { div { "Carregando configurações..." } },
        Some(Err(_)) => {
            rsx! { div { class: "error-state-friendly", "Não foi possível carregar as configurações." } }
        }
        Some(Ok(data)) => {
            let mut auto_reminders = use_signal(|| data.auto_reminders);
            let mut require_esign = use_signal(|| data.require_esign);
            let mut smtp_host = use_signal(|| data.smtp_host.clone().unwrap_or_default());
            let mut smtp_port = use_signal(|| data.smtp_port.map(|p| p.to_string()).unwrap_or_else(|| "587".into()));
            let mut smtp_user = use_signal(|| data.smtp_user.clone().unwrap_or_default());
            let mut smtp_pass = use_signal(String::new);
            let mut smtp_from = use_signal(|| data.smtp_from.clone().unwrap_or_default());
            let mut smtp_tls = use_signal(|| data.smtp_tls.unwrap_or(true));

            let id_sv = clinic_id.clone();
            let t_sv = token.clone();
            let handle_save = move |_| {
                if !can_write {
                    return;
                }
                is_saving.set(true);
                let id = id_sv.clone();
                let t = t_sv.clone();
                spawn(async move {
                    let port_val = smtp_port().trim().parse::<u16>().ok();
                    let req = UpdateClinicRequest {
                        trading_name: None,
                        corporate_name: None,
                        document_cnpj: None,
                        theme_color: None,
                        address: None,
                        auto_reminders: Some(auto_reminders()),
                        require_esign: Some(require_esign()),
                        smtp_host: if smtp_host().trim().is_empty() { None } else { Some(smtp_host()) },
                        smtp_port: port_val,
                        smtp_user: if smtp_user().trim().is_empty() { None } else { Some(smtp_user()) },
                        smtp_pass: if smtp_pass().trim().is_empty() { None } else { Some(smtp_pass()) },
                        smtp_from: if smtp_from().trim().is_empty() { None } else { Some(smtp_from()) },
                        smtp_tls: Some(smtp_tls()),
                    };
                    if let Err(msg) = api::update_clinic(&t, &id, req).await {
                        error_toast.set(Some(msg));
                    }
                    is_saving.set(false);
                });
            };

            let id_del = clinic_id.clone();
            let t_del = token.clone();
            let mut active_clinic = consume_context::<Signal<ActiveClinicState>>();
            let mut session = consume_context::<Signal<SessionState>>();
            let navigator = use_navigator();

            let handle_delete = move |_| {
                if !can_delete {
                    return;
                }
                let id = id_del.clone();
                let t = t_del.clone();
                spawn(async move {
                    match api::delete_clinic(&t, &id).await {
                        Ok(_) => {
                            active_clinic.set(None);
                            session.set(None);
                            navigator.replace(Route::LoginScreen {});
                        }
                        Err(msg) => error_toast.set(Some(msg)),
                    }
                });
            };

            rsx! {
                div { class: "settings-pane-container",
                    h3 { class: "settings-pane-title", "Configurações Avançadas" }
                    p { class: "tab-description", "Gerencie o comportamento global da unidade." }

                    div { class: "advanced-setting-row",
                        div { class: "advanced-setting-info",
                            h4 { "Lembretes Automáticos" }
                            p { "Dispara mensagens 24h antes do agendamento." }
                        }
                        div { class: "toggle-switch",
                            input { r#type: "checkbox", disabled: !can_write, checked: auto_reminders(), onchange: move |e| auto_reminders.set(e.checked()) }
                        }
                    }

                    div { class: "advanced-setting-row",
                        div { class: "advanced-setting-info",
                            h4 { "Exigir Assinatura Digital" }
                            p { "Bloqueia fluxos clínicos sem confirmação via OTP." }
                        }
                        div { class: "toggle-switch",
                            input { r#type: "checkbox", disabled: !can_write, checked: require_esign(), onchange: move |e| require_esign.set(e.checked()) }
                        }
                    }

                    // SMTP Custom Clinic Configuration
                    div { class: "settings-smtp-box",
                        div { class: "settings-smtp-header",
                            h4 { class: "settings-subtitle", "Servidor SMTP Próprio (E-mail da Clínica)" }
                            p { class: "tab-description", "Opcional. Se não preenchido, o sistema usará o SMTP padrão configurado no .ENV." }
                        }

                        div { class: "smtp-form-grid",
                            div { class: "input-group-wrapper",
                                label { "Host SMTP:" }
                                input {
                                    class: "modern-input-field",
                                    placeholder: "ex: smtp.sendgrid.net",
                                    disabled: !can_write,
                                    value: "{smtp_host}",
                                    oninput: move |e| smtp_host.set(e.value()),
                                }
                            }
                            div { class: "input-group-wrapper",
                                label { "Porta SMTP:" }
                                input {
                                    class: "modern-input-field",
                                    placeholder: "587",
                                    disabled: !can_write,
                                    value: "{smtp_port}",
                                    oninput: move |e| smtp_port.set(e.value()),
                                }
                            }
                            div { class: "input-group-wrapper",
                                label { "Usuário / E-mail SMTP:" }
                                input {
                                    class: "modern-input-field",
                                    placeholder: "ex: apikey ou seu@email.com",
                                    disabled: !can_write,
                                    value: "{smtp_user}",
                                    oninput: move |e| smtp_user.set(e.value()),
                                }
                            }
                            div { class: "input-group-wrapper",
                                label { "Senha SMTP (Preencha para alterar):" }
                                input {
                                    r#type: "password",
                                    class: "modern-input-field",
                                    placeholder: "••••••••",
                                    disabled: !can_write,
                                    value: "{smtp_pass}",
                                    oninput: move |e| smtp_pass.set(e.value()),
                                }
                            }
                            div { class: "input-group-wrapper full-width",
                                label { "E-mail de Remetente (From):" }
                                input {
                                    class: "modern-input-field",
                                    placeholder: "ex: Clinica Tooth Plus <contato@toothplus.com.br>",
                                    disabled: !can_write,
                                    value: "{smtp_from}",
                                    oninput: move |e| smtp_from.set(e.value()),
                                }
                            }
                            div { class: "input-group-wrapper full-width smtp-tls-row",
                                label { class: "smtp-toggle-label",
                                    input {
                                        r#type: "checkbox",
                                        class: "smtp-checkbox-input",
                                        disabled: !can_write,
                                        checked: smtp_tls(),
                                        onchange: move |e| smtp_tls.set(e.checked()),
                                    }
                                    span { class: "smtp-toggle-text", "Conexão Segura TLS / STARTTLS (Recomendado)" }
                                }
                            }
                        }
                    }

                    if can_write {
                        div { class: "settings-action-bar",
                            button { class: "btn-primary", onclick: handle_save, disabled: is_saving(),
                                if is_saving() { "Salvando..." } else { "Salvar Alterações" }
                            }
                        }
                    }

                    div { class: "danger-zone",
                        h4 { class: "danger-zone-title", "Zona de Perigo" }
                        p { class: "danger-zone-desc", "Ações irreversíveis no banco de dados." }
                        if can_delete {
                            button { class: "btn-danger", onclick: handle_delete, "Encerrar Clínica" }
                        } else {
                            p { class: "text-error-small", "Ação restrita ao Administrador." }
                        }
                    }
                }
            }
        }
    }
}
