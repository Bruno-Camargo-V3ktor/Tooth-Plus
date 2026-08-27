use crate::api::users::UsersApi;
use crate::components::modal::Modal;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconEdit, IconPlus, IconTrash, IconUser};
use shared::users::{CreateUserRequest, UpdateUserRequest, UserResponse};
use dioxus::prelude::*;

#[component]
pub fn TabTeam(clinic_id: String) -> Element {
    let toast = consume_context::<ToastState>();

    let mut users_list = use_signal(Vec::<UserResponse>::new);
    let mut reload_trigger = use_signal(|| 0);

    let mut show_modal = use_signal(|| false);
    let mut editing_user_id = use_signal(|| None::<String>);

    // Campos do formulário
    let mut full_name = use_signal(String::new);
    let mut username = use_signal(String::new);
    let mut password_plain = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut document_cpf = use_signal(String::new);
    let mut role = use_signal(|| "dentist".to_string());
    let mut professional_registry = use_signal(String::new);
    let mut permissions = use_signal(|| vec![
        "agenda".to_string(),
        "patients".to_string(),
        "finance".to_string(),
        "treatments".to_string(),
    ]);

    let cid_eff = clinic_id.clone();
    use_effect(move || {
        let _ = reload_trigger.read();
        let cid = cid_eff.clone();
        spawn(async move {
            if let Ok(resps) = UsersApi::list_users(&cid).await {
                users_list.set(resps);
            }
        });
    });



    let handle_open_new = move |_| {
        editing_user_id.set(None);
        full_name.set(String::new());
        username.set(String::new());
        password_plain.set(String::new());
        email.set(String::new());
        phone.set(String::new());
        document_cpf.set(String::new());
        role.set("dentist".to_string());
        professional_registry.set(String::new());
        permissions.set(vec![
            "agenda".to_string(),
            "patients".to_string(),
            "treatments".to_string(),
        ]);
        show_modal.set(true);
    };

    let handle_save = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut reload_c = reload_trigger;
        let mut modal_c = show_modal;

        move |_| {
            let fn_val = full_name.read().trim().to_string();
            let un_val = username.read().trim().to_string();
            let cpf_val = document_cpf.read().trim().to_string();

            if fn_val.is_empty() || un_val.is_empty() {
                toast_c.show("Informe o nome completo e o nome de usuário.", ToastVariant::Error);
                return;
            }

            let edit_id_opt = editing_user_id.read().clone();
            let email_val = if email.read().trim().is_empty() { None } else { Some(email.read().trim().to_string()) };
            let phone_val = if phone.read().trim().is_empty() { None } else { Some(phone.read().trim().to_string()) };
            let cro_val = if professional_registry.read().trim().is_empty() { None } else { Some(professional_registry.read().trim().to_string()) };
            let role_val = role.read().clone();
            let perms_val = permissions.read().clone();

            let mut toast_resp = toast_c.clone();
            let mut reload_resp = reload_c;
            let mut modal_resp = modal_c;
            let cid_call = cid.clone();

            spawn(async move {
                if let Some(ref uid) = edit_id_opt {
                    let req = UpdateUserRequest {
                        full_name: Some(fn_val),
                        email: email_val,
                        phone: phone_val,
                        new_password: None,
                        document_cpf: Some(cpf_val),
                        professional_registry: cro_val,
                        role: Some(role_val),
                        permissions: Some(perms_val),
                        clinic_ids: Some(vec![cid_call]),
                    };
                    match UsersApi::update_user(uid, req).await {
                        Ok(_) => {
                            toast_resp.show("Membro da equipe atualizado com sucesso!", ToastVariant::Success);
                            modal_resp.set(false);
                            reload_resp.set(reload_resp() + 1);
                        }
                        Err(err) => toast_resp.show(err, ToastVariant::Error),
                    }
                } else {
                    let req = CreateUserRequest {
                        username: un_val,
                        password_plain: "123456".to_string(),
                        full_name: fn_val,
                        email: email_val,
                        phone: phone_val,
                        document_cpf: cpf_val,
                        professional_registry: cro_val,
                        role: role_val,
                        permissions: perms_val,
                        clinic_ids: vec![cid_call],
                    };
                    match UsersApi::create_user(req).await {
                        Ok(_) => {
                            toast_resp.show("Novo membro cadastrado com sucesso!", ToastVariant::Success);
                            modal_resp.set(false);
                            reload_resp.set(reload_resp() + 1);
                        }
                        Err(err) => toast_resp.show(err, ToastVariant::Error),
                    }
                }
            });
        }
    };

    let perms_cur = permissions.read().clone();

    rsx! {
        div {
            div { class: "settings-list-header",
                h2 { class: "settings-list-title", "Membros da Equipe & Acessos" }
                button {
                    r#type: "button",
                    class: "settings-btn-new-green",
                    onclick: handle_open_new,
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                    span { "NOVO USUÁRIO" }
                }
            }

            div { class: "settings-table-card",
                table { class: "settings-table",
                    thead {
                        tr {
                            th { "Membro / Profissional" }
                            th { "Função / Cargo" }
                            th { "Permissões de Acesso" }
                            th { "Contato" }
                            th { style: "text-align: right; width: 100px;", "Ações" }
                        }
                    }
                    tbody {
                        for u in users_list() {
                            {
                                let uid = u.id.clone();
                                let uid_del = u.id.clone();
                                let u_fn = u.full_name.clone();
                                let u_un = u.username.clone();
                                let u_em = u.email.clone().unwrap_or_default();
                                let u_ph = u.phone.clone().unwrap_or_default();
                                let u_cpf = u.document_cpf.clone();
                                let u_role = u.role.clone();
                                let u_cro = u.professional_registry.clone().unwrap_or_default();
                                let u_perms = u.permissions.clone();
                                let u_perms_edit = u.permissions.clone();

                                let initials = u.full_name.split_whitespace().take(2).map(|w| w.chars().next().unwrap_or('U')).collect::<String>();

                                let role_label = match u.role.as_str() {
                                    "dentist" => "Dentista Clínico",
                                    "admin" => "Administrador Geral",
                                    "receptionist" => "Recepcionista",
                                    "assistant" => "Auxiliar Odontológico (ASB)",
                                    _ => "Equipe",
                                };

                                let mut toast_del = toast.clone();
                                let mut reload_del = reload_trigger;

                                rsx! {
                                    tr { key: "{u.id}",
                                        td {
                                            div { style: "display: flex; align-items: center; gap: 12px;",
                                                div { style: "width: 36px; height: 36px; border-radius: 50%; background: #0284c7; color: #fff; font-size: 13px; font-weight: 800; display: flex; align-items: center; justify-content: center; flex-shrink: 0;",
                                                    "{initials}"
                                                }
                                                div {
                                                    strong { style: "font-size: 14px; color: #f1f5f9; display: block;", "{u.full_name}" }
                                                    if let Some(ref reg) = u.professional_registry {
                                                        span { style: "font-size: 11.5px; color: #38bdf8;", "{reg}" }
                                                    }
                                                }
                                            }
                                        }
                                        td {
                                            span { class: "badge badge-blue", style: "font-size: 11.5px;", "{role_label}" }
                                        }
                                        td {
                                            div { style: "display: flex; flex-wrap: wrap; gap: 4px; max-width: 320px;",
                                                for p in u_perms {
                                                    span { class: "badge badge-gray", style: "font-size: 10.5px; padding: 1px 6px;",
                                                        match p.as_str() {
                                                            "agenda" => "📅 Agenda",
                                                            "patients" => "👥 Prontuário",
                                                            "finance" => "💵 Financeiro",
                                                            "stock" => "📦 Estoque",
                                                            "documents" => "📄 Documentos",
                                                            "settings" => "⚙️ Ajustes",
                                                            _ => "Acesso",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        td {
                                            div { style: "font-size: 12px; color: #94a3b8;",
                                                if let Some(ref em) = u.email {
                                                    div { "{em}" }
                                                }
                                                if let Some(ref ph) = u.phone {
                                                    div { "{ph}" }
                                                }
                                            }
                                        }
                                        td { style: "text-align: right;",
                                            div { style: "display: inline-flex; align-items: center; gap: 8px;",
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Editar permissões e dados",
                                                    onclick: move |_| {
                                                        editing_user_id.set(Some(uid.clone()));
                                                        full_name.set(u_fn.clone());
                                                        username.set(u_un.clone());
                                                        email.set(u_em.clone());
                                                        phone.set(u_ph.clone());
                                                        document_cpf.set(u_cpf.clone());
                                                        role.set(u_role.clone());
                                                        professional_registry.set(u_cro.clone());
                                                        permissions.set(u_perms_edit.clone());
                                                        show_modal.set(true);
                                                    },
                                                    IconEdit { size: 14, color: "#94a3b8".to_string() }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Excluir membro da equipe",
                                                    onclick: move |_| {
                                                        let del_id = uid_del.clone();
                                                        let mut t_d = toast_del.clone();
                                                        let mut r_d = reload_del;
                                                        spawn(async move {
                                                            if let Ok(_) = UsersApi::delete_user(&del_id).await {
                                                                t_d.show("Membro excluído com sucesso.", ToastVariant::Success);
                                                                r_d.set(r_d() + 1);
                                                            }
                                                        });
                                                    },
                                                    IconTrash { size: 14, color: "#ef4444".to_string() }
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

            if show_modal() {
                Modal {
                    title: if editing_user_id().is_some() { "Editar Membro da Equipe".to_string() } else { "Cadastrar Membro da Equipe".to_string() },
                    is_open: show_modal(),
                    on_close: move |_| show_modal.set(false),
                    footer: rsx! {
                        div { style: "display: flex; justify-content: flex-end; gap: 10px; width: 100%;",
                            button {
                                r#type: "button",
                                class: "btn-modal-ghost",
                                onclick: move |_| show_modal.set(false),
                                "CANCELAR"
                            }
                            button {
                                r#type: "button",
                                class: "settings-btn-save",
                                onclick: handle_save,
                                "SALVAR USUÁRIO"
                            }
                        }
                    },

                    div { style: "display: flex; flex-direction: column; gap: 14px; max-height: 70vh; overflow-y: auto; padding-right: 4px;",
                        div { style: "display: grid; grid-template-columns: 2fr 1fr; gap: 12px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Nome Completo *" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: Dra. Mariana Vasconcelos",
                                    value: "{full_name}",
                                    oninput: move |e| full_name.set(e.value()),
                                }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Usuário (login) *" }
                                input {
                                    class: "form-input",
                                    placeholder: "ex: mariana.vasc",
                                    value: "{username}",
                                    oninput: move |e| username.set(e.value()),
                                }
                            }
                        }

                        div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "E-mail" }
                                input {
                                    class: "form-input",
                                    r#type: "email",
                                    placeholder: "mariana@clinica.com",
                                    value: "{email}",
                                    oninput: move |e| email.set(e.value()),
                                }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Celular / WhatsApp" }
                                input {
                                    class: "form-input",
                                    placeholder: "(11) 98888-7777",
                                    value: "{phone}",
                                    oninput: move |e| phone.set(e.value()),
                                }
                            }
                        }

                        div { style: "display: grid; grid-template-columns: 1fr 1fr 1.2fr; gap: 12px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "CPF *" }
                                input {
                                    class: "form-input",
                                    placeholder: "000.000.000-00",
                                    value: "{document_cpf}",
                                    oninput: move |e| document_cpf.set(e.value()),
                                }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Cargo / Função" }
                                select {
                                    class: "form-select",
                                    value: "{role}",
                                    onchange: move |e| role.set(e.value()),
                                    option { value: "dentist", "Dentista" }
                                    option { value: "admin", "Administrador Geral" }
                                    option { value: "receptionist", "Recepcionista" }
                                    option { value: "assistant", "Auxiliar (ASB)" }
                                }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "CRO / Registro (se dentista)" }
                                input {
                                    class: "form-input",
                                    placeholder: "CRO-SP 12345",
                                    value: "{professional_registry}",
                                    oninput: move |e| professional_registry.set(e.value()),
                                }
                            }
                        }

                        // DEFINIÇÃO DE ACESSOS E PERMISSÕES
                        div { style: "margin-top: 10px; background: rgba(255,255,255,0.02); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; padding: 14px;",
                            h4 { style: "font-size: 13.5px; font-weight: 700; color: #38bdf8; margin: 0 0 10px 0;", "Acessos e Permissões no Sistema" }

                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                                label { class: "settings-checkbox-item",
                                    input {
                                        r#type: "checkbox",
                                        checked: perms_cur.contains(&"agenda".to_string()),
                                        onchange: move |_| {
                                        let mut perms = permissions.read().clone();
                                        if perms.contains(&"agenda".to_string()) {
                                            perms.retain(|x| x != "agenda");
                                        } else {
                                            perms.push("agenda".to_string());
                                        }
                                        permissions.set(perms);
                                    },
                                    }
                                    span { "📅 Agenda e Agendamentos" }
                                }
                                label { class: "settings-checkbox-item",
                                    input {
                                        r#type: "checkbox",
                                        checked: perms_cur.contains(&"patients".to_string()),
                                        onchange: move |_| {
                                        let mut perms = permissions.read().clone();
                                        if perms.contains(&"patients".to_string()) {
                                            perms.retain(|x| x != "patients");
                                        } else {
                                            perms.push("patients".to_string());
                                        }
                                        permissions.set(perms);
                                    },
                                    }
                                    span { "👥 Prontuário e Pacientes" }
                                }
                                label { class: "settings-checkbox-item",
                                    input {
                                        r#type: "checkbox",
                                        checked: perms_cur.contains(&"finance".to_string()),
                                        onchange: move |_| {
                                        let mut perms = permissions.read().clone();
                                        if perms.contains(&"finance".to_string()) {
                                            perms.retain(|x| x != "finance");
                                        } else {
                                            perms.push("finance".to_string());
                                        }
                                        permissions.set(perms);
                                    },
                                    }
                                    span { "💵 Módulo Financeiro" }
                                }
                                label { class: "settings-checkbox-item",
                                    input {
                                        r#type: "checkbox",
                                        checked: perms_cur.contains(&"stock".to_string()),
                                        onchange: move |_| {
                                        let mut perms = permissions.read().clone();
                                        if perms.contains(&"stock".to_string()) {
                                            perms.retain(|x| x != "stock");
                                        } else {
                                            perms.push("stock".to_string());
                                        }
                                        permissions.set(perms);
                                    },
                                    }
                                    span { "📦 Gestão de Estoque" }
                                }
                                label { class: "settings-checkbox-item",
                                    input {
                                        r#type: "checkbox",
                                        checked: perms_cur.contains(&"treatments".to_string()),
                                        onchange: move |_| {
                                        let mut perms = permissions.read().clone();
                                        if perms.contains(&"treatments".to_string()) {
                                            perms.retain(|x| x != "treatments");
                                        } else {
                                            perms.push("treatments".to_string());
                                        }
                                        permissions.set(perms);
                                    },
                                    }
                                    span { "🦷 Catálogo de Procedimentos" }
                                }
                                label { class: "settings-checkbox-item",
                                    input {
                                        r#type: "checkbox",
                                        checked: perms_cur.contains(&"documents".to_string()),
                                        onchange: move |_| {
                                        let mut perms = permissions.read().clone();
                                        if perms.contains(&"documents".to_string()) {
                                            perms.retain(|x| x != "documents");
                                        } else {
                                            perms.push("documents".to_string());
                                        }
                                        permissions.set(perms);
                                    },
                                    }
                                    span { "📄 Modelos e Assinatura Digital" }
                                }
                                label { class: "settings-checkbox-item",
                                    input {
                                        r#type: "checkbox",
                                        checked: perms_cur.contains(&"settings".to_string()),
                                        onchange: move |_| {
                                        let mut perms = permissions.read().clone();
                                        if perms.contains(&"settings".to_string()) {
                                            perms.retain(|x| x != "settings");
                                        } else {
                                            perms.push("settings".to_string());
                                        }
                                        permissions.set(perms);
                                    },
                                    }
                                    span { "⚙️ Ajustes da Clínica" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
