use crate::api::users::UsersApi;
use crate::components::modal::Modal;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconCheck, IconClose, IconEdit, IconPlus, IconSearch, IconTrash, IconUser, IconUsers};
use shared::users::{CreateUserRequest, UpdateUserRequest, UserResponse};
use dioxus::prelude::*;

#[component]
pub fn TabTeam(clinic_id: String) -> Element {
    let toast = consume_context::<ToastState>();

    let mut users_list = use_signal(Vec::<UserResponse>::new);
    let mut reload_trigger = use_signal(|| 0);
    let mut search_query = use_signal(String::new);
    let mut role_filter = use_signal(|| "ALL".to_string());

    let mut show_modal = use_signal(|| false);
    let mut editing_user_id = use_signal(|| None::<String>);

    // Campos do formulário
    let mut full_name = use_signal(String::new);
    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut document_cpf = use_signal(String::new);
    let mut role = use_signal(|| "dentist".to_string());
    let mut professional_registry = use_signal(String::new);
    let mut is_active_sig = use_signal(|| true);
    let mut permissions = use_signal(|| vec![
        "agenda".to_string(),
        "patients".to_string(),
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
        email.set(String::new());
        phone.set(String::new());
        document_cpf.set(String::new());
        role.set("dentist".to_string());
        professional_registry.set(String::new());
        is_active_sig.set(true);
        permissions.set(vec![
            "agenda".to_string(),
            "patients".to_string(),
            "treatments".to_string(),
            "documents".to_string(),
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
                toast_c.show("Informe o nome completo e o nome de usuário (login).", ToastVariant::Error);
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

    let filtered_users: Vec<UserResponse> = {
        let q = search_query.read().to_lowercase();
        let rf = role_filter.read().clone();

        users_list
            .read()
            .iter()
            .filter(|u| {
                if rf != "ALL" && u.role != rf {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                u.full_name.to_lowercase().contains(&q)
                    || u.username.to_lowercase().contains(&q)
                    || u.email.as_ref().map(|e| e.to_lowercase().contains(&q)).unwrap_or(false)
                    || u.document_cpf.contains(&q)
            })
            .cloned()
            .collect()
    };

    let perms_cur = permissions.read().clone();
    let current_role = role.read().clone();

    rsx! {
        div {
            // HEADER COM BARRA DE PESQUISA, FILTROS E NOVO USUÁRIO
            div { class: "settings-card", style: "padding: 16px 20px; margin-bottom: 20px;",
                div { style: "display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;",
                    div { style: "display: flex; align-items: center; gap: 12px; flex: 1; max-width: 400px; position: relative;",
                        div { style: "position: relative; width: 100%; display: flex; align-items: center;",
                            input {
                                class: "form-input",
                                style: "padding-left: 36px; height: 38px;",
                                placeholder: "Buscar membro por nome, login ou e-mail...",
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value()),
                            }
                            div { style: "position: absolute; left: 10px; pointer-events: none;",
                                IconSearch { size: 16, color: "var(--text-muted, #94a3b8)".to_string() }
                            }
                        }
                    }

                    div { style: "display: flex; align-items: center; gap: 10px;",
                        select {
                            class: "form-select",
                            style: "height: 38px; min-width: 160px; font-size: 13px;",
                            value: "{role_filter}",
                            onchange: move |e| role_filter.set(e.value()),
                            option { value: "ALL", "Todos os Cargos" }
                            option { value: "dentist", "Dentistas" }
                            option { value: "admin", "Administradores" }
                            option { value: "receptionist", "Recepcionistas" }
                            option { value: "assistant", "Auxiliares (ASB)" }
                        }

                        button {
                            r#type: "button",
                            class: "btn-primary-blue",
                            style: "height: 38px; font-size: 13px; font-weight: 700; display: inline-flex; align-items: center; gap: 8px; padding: 0 18px;",
                            onclick: handle_open_new,
                            IconPlus { size: 16, color: "#ffffff".to_string() }
                            span { "Novo Usuário" }
                        }
                    }
                }
            }

            // TABELA DE MEMBROS
            div { class: "settings-table-card",
                table { class: "settings-table",
                    thead {
                        tr {
                            th { "Membro / Profissional" }
                            th { "Cargo / Perfil" }
                            th { "Módulos com Acesso" }
                            th { "Contato & Documento" }
                            th { style: "text-align: right; width: 100px;", "Ações" }
                        }
                    }
                    tbody {
                        if filtered_users.is_empty() {
                            tr {
                                td { colspan: "5", style: "text-align: center; padding: 40px; color: var(--text-muted, #94a3b8);",
                                    "Nenhum membro encontrado com os filtros informados."
                                }
                            }
                        }
                        for u in filtered_users {
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

                                let (role_label, role_color) = match u.role.as_str() {
                                    "dentist" => ("Dentista Clínico", "#00a0e4"),
                                    "admin" => ("Administrador Geral", "#a855f7"),
                                    "receptionist" => ("Recepcionista", "#22c55e"),
                                    "assistant" => ("Auxiliar (ASB)", "#eab308"),
                                    _ => ("Equipe", "#64748b"),
                                };

                                let mut toast_del = toast.clone();
                                let mut reload_del = reload_trigger;

                                rsx! {
                                    tr { key: "{u.id}",
                                        td {
                                            div { style: "display: flex; align-items: center; gap: 12px;",
                                                div { style: format!("width: 38px; height: 38px; border-radius: 50%; background: {}; color: #fff; font-size: 13px; font-weight: 800; display: flex; align-items: center; justify-content: center; flex-shrink: 0; box-shadow: 0 2px 6px rgba(0,0,0,0.3);", role_color),
                                                    "{initials}"
                                                }
                                                div {
                                                    strong { style: "font-size: 14px; color: var(--text-main, #f8fafc); display: block;", "{u.full_name}" }
                                                    div { style: "display: flex; align-items: center; gap: 6px; font-size: 11.5px;",
                                                        span { style: "color: var(--text-muted, #94a3b8);", "@{u.username}" }
                                                        if let Some(ref reg) = u.professional_registry {
                                                            span { style: "color: var(--primary, #00a0e4); font-weight: 600;", "• {reg}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        td {
                                            span {
                                                class: "badge",
                                                style: format!("background: rgba(255,255,255,0.06); color: {}; border: 1px solid {}; font-weight: 700; font-size: 12px; padding: 4px 10px; border-radius: 6px;", role_color, role_color),
                                                "{role_label}"
                                            }
                                        }
                                        td {
                                            div { style: "display: flex; flex-wrap: wrap; gap: 4px; max-width: 340px;",
                                                for p in u_perms.iter() {
                                                    span {
                                                        class: "badge",
                                                        style: "background: rgba(255,255,255,0.04); color: var(--text-muted, #94a3b8); font-size: 11px; padding: 2px 7px; border-radius: 4px; border: 1px solid var(--border-color, rgba(255,255,255,0.06));",
                                                        match p.as_str() {
                                                            "agenda" => "📅 Agenda",
                                                            "patients" => "👥 Prontuário",
                                                            "finance" => "💵 Financeiro",
                                                            "stock" => "📦 Estoque",
                                                            "treatments" => "🦷 Procedimentos",
                                                            "documents" => "📄 Assinatura & Docs",
                                                            "settings" => "⚙️ Ajustes",
                                                            _ => "Acesso",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        td {
                                            div { style: "font-size: 12.5px; color: var(--text-muted, #94a3b8); display: flex; flex-direction: column; gap: 2px;",
                                                if let Some(ref em) = u.email {
                                                    div { "{em}" }
                                                }
                                                if let Some(ref ph) = u.phone {
                                                    div { "{ph}" }
                                                }
                                                span { style: "font-size: 11px; color: var(--text-light, #64748b);", "CPF: {u_cpf}" }
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
                                                    IconEdit { size: 15, color: "var(--text-muted, #94a3b8)".to_string() }
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
                                                                t_d.show("Membro da equipe excluído com sucesso.", ToastVariant::Success);
                                                                r_d.set(r_d() + 1);
                                                            }
                                                        });
                                                    },
                                                    IconTrash { size: 15, color: "#ef4444".to_string() }
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

            // MODAL DE CADASTRO E CONTROLE DE PERMISSÕES
            if show_modal() {
                Modal {
                    title: if editing_user_id().is_some() { "Editar Membro da Equipe".to_string() } else { "Cadastrar Novo Membro da Equipe".to_string() },
                    is_open: show_modal(),
                    on_close: move |_| show_modal.set(false),
                    footer: rsx! {
                        div { style: "display: flex; justify-content: flex-end; gap: 12px; width: 100%;",
                            button {
                                r#type: "button",
                                class: "btn-modal-ghost",
                                onclick: move |_| show_modal.set(false),
                                "CANCELAR"
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary-blue",
                                style: "font-weight: 700; padding: 0 24px; height: 38px;",
                                onclick: handle_save,
                                "SALVAR USUÁRIO"
                            }
                        }
                    },

                    div { style: "display: flex; flex-direction: column; gap: 18px; max-height: 72vh; overflow-y: auto; padding-right: 6px;",
                        // SELETOR RÁPIDO DE PERFIL (PRESETS)
                        div { style: "background: rgba(255,255,255,0.02); border: 1px solid var(--border-color, rgba(255,255,255,0.08)); border-radius: var(--radius-md, 8px); padding: 14px;",
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                                span { style: "font-size: 12.5px; font-weight: 700; color: var(--text-muted, #94a3b8); text-transform: uppercase; letter-spacing: 0.04em;", "Perfil Pré-definido" }
                                span { style: "font-size: 11px; color: var(--text-light, #64748b);", "Aplica permissões recomendadas automaticamente" }
                            }
                            div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                button {
                                    r#type: "button",
                                    class: if current_role == "dentist" { "btn-filter-pill active" } else { "btn-filter-pill" },
                                    onclick: move |_| {
                                        role.set("dentist".to_string());
                                        permissions.set(vec![
                                            "agenda".to_string(),
                                            "patients".to_string(),
                                            "treatments".to_string(),
                                            "documents".to_string(),
                                        ]);
                                    },
                                    "🦷 Dentista Clínico"
                                }
                                button {
                                    r#type: "button",
                                    class: if current_role == "receptionist" { "btn-filter-pill active" } else { "btn-filter-pill" },
                                    onclick: move |_| {
                                        role.set("receptionist".to_string());
                                        permissions.set(vec![
                                            "agenda".to_string(),
                                            "patients".to_string(),
                                            "finance".to_string(),
                                        ]);
                                    },
                                    "📋 Recepcionista / Secretária"
                                }
                                button {
                                    r#type: "button",
                                    class: if current_role == "admin" { "btn-filter-pill active" } else { "btn-filter-pill" },
                                    onclick: move |_| {
                                        role.set("admin".to_string());
                                        permissions.set(vec![
                                            "agenda".to_string(),
                                            "patients".to_string(),
                                            "finance".to_string(),
                                            "stock".to_string(),
                                            "treatments".to_string(),
                                            "documents".to_string(),
                                            "settings".to_string(),
                                        ]);
                                    },
                                    "👑 Administrador / Gestor"
                                }
                                button {
                                    r#type: "button",
                                    class: if current_role == "assistant" { "btn-filter-pill active" } else { "btn-filter-pill" },
                                    onclick: move |_| {
                                        role.set("assistant".to_string());
                                        permissions.set(vec![
                                            "agenda".to_string(),
                                            "patients".to_string(),
                                            "stock".to_string(),
                                        ]);
                                    },
                                    "🩺 Auxiliar (ASB)"
                                }
                            }
                        }

                        // DADOS PESSOAIS E DE ACESSO
                        div { style: "display: grid; grid-template-columns: 2fr 1fr; gap: 14px;",
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
                                label { class: "form-label", "Nome de Usuário (login) *" }
                                input {
                                    class: "form-input",
                                    placeholder: "ex: mariana.vasc",
                                    value: "{username}",
                                    oninput: move |e| username.set(e.value()),
                                }
                            }
                        }

                        div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 14px;",
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "E-mail Institucional" }
                                input {
                                    class: "form-input",
                                    r#type: "email",
                                    placeholder: "mariana@clinica.com",
                                    value: "{email}",
                                    oninput: move |e| email.set(e.value()),
                                }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "WhatsApp / Celular" }
                                input {
                                    class: "form-input",
                                    placeholder: "(11) 98888-7777",
                                    value: "{phone}",
                                    oninput: move |e| phone.set(e.value()),
                                }
                            }
                        }

                        div { style: "display: grid; grid-template-columns: 1.2fr 1.2fr 1.5fr; gap: 14px;",
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
                                label { class: "form-label", "Cargo no Sistema" }
                                select {
                                    class: "form-select",
                                    value: "{role}",
                                    onchange: move |e| role.set(e.value()),
                                    option { value: "dentist", "Dentista Clínico" }
                                    option { value: "admin", "Administrador Geral" }
                                    option { value: "receptionist", "Recepcionista" }
                                    option { value: "assistant", "Auxiliar Odontológico (ASB)" }
                                }
                            }
                            div { class: "form-field", style: "margin: 0;",
                                label { class: "form-label", "Registro Profissional (CRO / UF)" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: CRO-SP 123456",
                                    value: "{professional_registry}",
                                    oninput: move |e| professional_registry.set(e.value()),
                                }
                            }
                        }

                        // MATRIZ DETALHADA DE PERMISSÕES (PBAC)
                        div { class: "settings-card", style: "margin: 0; padding: 18px;",
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary, #00a0e4); margin: 0;", "Permissões de Acesso por Módulo" }
                                span { style: "font-size: 11.5px; color: var(--text-muted, #94a3b8);", "Controle granular por usuário" }
                            }

                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",
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
                                    div {
                                        strong { style: "display: block; font-size: 13px;", "📅 Agenda Clínica" }
                                        span { style: "font-size: 11px; color: var(--text-muted, #94a3b8);", "Ver, agendar e gerenciar consultas" }
                                    }
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
                                    div {
                                        strong { style: "display: block; font-size: 13px;", "👥 Prontuário & Pacientes" }
                                        span { style: "font-size: 11px; color: var(--text-muted, #94a3b8);", "Fichas, anamnese e evoluções clínicas" }
                                    }
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
                                    div {
                                        strong { style: "display: block; font-size: 13px;", "💵 Módulo Financeiro" }
                                        span { style: "font-size: 11px; color: var(--text-muted, #94a3b8);", "Lançamentos, recebimentos e relatórios" }
                                    }
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
                                    div {
                                        strong { style: "display: block; font-size: 13px;", "📦 Gestão de Estoque" }
                                        span { style: "font-size: 11px; color: var(--text-muted, #94a3b8);", "Controle de insumos e movimentações" }
                                    }
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
                                    div {
                                        strong { style: "display: block; font-size: 13px;", "🦷 Procedimentos & Catálogo" }
                                        span { style: "font-size: 11px; color: var(--text-muted, #94a3b8);", "Tabelas de procedimentos e orçamentos" }
                                    }
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
                                    div {
                                        strong { style: "display: block; font-size: 13px;", "📄 Assinatura Digital & Docs" }
                                        span { style: "font-size: 11px; color: var(--text-muted, #94a3b8);", "Contratos, atestados e termos legais" }
                                    }
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
                                    div {
                                        strong { style: "display: block; font-size: 13px;", "⚙️ Configurações da Clínica" }
                                        span { style: "font-size: 11px; color: var(--text-muted, #94a3b8);", "Gerenciamento institucional e equipe" }
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
