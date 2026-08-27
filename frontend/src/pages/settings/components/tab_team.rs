use crate::api::users::UsersApi;
use crate::components::modal::Modal;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{
    IconActivity, IconCheck, IconClose, IconEdit, IconFileText, IconPlus, IconSearch,
    IconShieldCheck, IconTooth, IconTrash, IconUser, IconUsers,
};
use crate::permissions::ALL_PERMISSION_GROUPS;
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
    let mut permissions = use_signal(|| vec![
        "agenda:read".to_string(),
        "agenda:write".to_string(),
        "patients:read".to_string(),
        "patients:write".to_string(),
        "anamnese:read".to_string(),
        "anamnese:write".to_string(),
        "treatment_plans:read".to_string(),
        "treatment_plans:write".to_string(),
        "documents:read".to_string(),
        "documents:write".to_string(),
        "documents:sign".to_string(),
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
        permissions.set(vec![
            "agenda:read".to_string(),
            "agenda:write".to_string(),
            "patients:read".to_string(),
            "patients:write".to_string(),
            "anamnese:read".to_string(),
            "anamnese:write".to_string(),
            "treatment_plans:read".to_string(),
            "treatment_plans:write".to_string(),
            "documents:read".to_string(),
            "documents:write".to_string(),
            "documents:sign".to_string(),
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
                            th { "Permissões Ativas" }
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
                                            span {
                                                class: "badge badge-blue",
                                                style: "font-size: 12px; font-weight: 700; padding: 4px 10px;",
                                                "{u_perms.len()} permissões concedidas"
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

            // MODAL DE CADASTRO E CONTROLE DE PERMISSÕES GRANULARES
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
                        // SELETOR RÁPIDO DE PERFIL (PRESETS) COM ÍCONES SVG
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
                                            "agenda:read".to_string(), "agenda:write".to_string(),
                                            "patients:read".to_string(), "patients:write".to_string(), "patients:evolutions".to_string(),
                                            "anamnese:read".to_string(), "anamnese:write".to_string(),
                                            "treatment_plans:read".to_string(), "treatment_plans:write".to_string(),
                                            "documents:read".to_string(), "documents:write".to_string(), "documents:sign".to_string(),
                                        ]);
                                    },
                                    IconTooth { size: 14, color: "currentColor".to_string() }
                                    span { "Dentista Clínico" }
                                }
                                button {
                                    r#type: "button",
                                    class: if current_role == "receptionist" { "btn-filter-pill active" } else { "btn-filter-pill" },
                                    onclick: move |_| {
                                        role.set("receptionist".to_string());
                                        permissions.set(vec![
                                            "agenda:read".to_string(), "agenda:write".to_string(), "agenda:delete".to_string(),
                                            "patients:read".to_string(), "patients:write".to_string(),
                                            "finance:read_income".to_string(), "finance:write_income".to_string(),
                                        ]);
                                    },
                                    IconFileText { size: 14, color: "currentColor".to_string() }
                                    span { "Recepcionista / Secretária" }
                                }
                                button {
                                    r#type: "button",
                                    class: if current_role == "admin" { "btn-filter-pill active" } else { "btn-filter-pill" },
                                    onclick: move |_| {
                                        role.set("admin".to_string());
                                        let all_p: Vec<String> = ALL_PERMISSION_GROUPS
                                            .iter()
                                            .flat_map(|g| g.items.iter().map(|(k, _)| k.to_string()))
                                            .collect();
                                        permissions.set(all_p);
                                    },
                                    IconShieldCheck { size: 14, color: "currentColor".to_string() }
                                    span { "Administrador Geral" }
                                }
                                button {
                                    r#type: "button",
                                    class: if current_role == "assistant" { "btn-filter-pill active" } else { "btn-filter-pill" },
                                    onclick: move |_| {
                                        role.set("assistant".to_string());
                                        permissions.set(vec![
                                            "agenda:read".to_string(),
                                            "patients:read".to_string(),
                                            "stock:read".to_string(), "stock:movement".to_string(),
                                        ]);
                                    },
                                    IconActivity { size: 14, color: "currentColor".to_string() }
                                    span { "Auxiliar (ASB)" }
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

                        // MATRIZ COMPLETA DE PERMISSÕES GRANULARES (PBAC)
                        div { style: "display: flex; flex-direction: column; gap: 12px; margin-top: 6px;",
                            div { style: "display: flex; justify-content: space-between; align-items: center; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, rgba(255,255,255,0.08));",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary, #00a0e4); margin: 0;", "Matriz de Permissões Granulares" }
                                span { style: "font-size: 12px; color: var(--text-muted, #94a3b8); font-weight: 600;", "{permissions.read().len()} itens ativos" }
                            }

                            for grp in ALL_PERMISSION_GROUPS {
                                {
                                    let grp_perms: Vec<String> = grp.items.iter().map(|(k, _)| k.to_string()).collect();
                                    let active_count = grp_perms.iter().filter(|p| permissions.read().contains(p)).count();
                                    let all_active = active_count == grp_perms.len() && !grp_perms.is_empty();
                                    let grp_perms_toggle = grp_perms.clone();

                                    rsx! {
                                        div { key: "{grp.key}", class: "permission-group-card",
                                            div { class: "permission-group-header",
                                                div {
                                                    h5 { class: "permission-group-title", "{grp.label}" }
                                                    p { style: "font-size: 11.5px; color: var(--text-muted, #94a3b8); margin: 2px 0 0 0;", "{grp.description}" }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "btn-text-sm",
                                                    style: "color: var(--primary, #00a0e4); font-size: 11.5px; font-weight: 700; background: none; border: none; cursor: pointer;",
                                                    onclick: move |_| {
                                                        let mut cur = permissions.read().clone();
                                                        if all_active {
                                                            cur.retain(|p| !grp_perms_toggle.contains(p));
                                                        } else {
                                                            for p in &grp_perms_toggle {
                                                                if !cur.contains(p) {
                                                                    cur.push(p.clone());
                                                                }
                                                            }
                                                        }
                                                        permissions.set(cur);
                                                    },
                                                    if all_active { "Desmarcar todas" } else { "Marcar todas" }
                                                }
                                            }

                                            div { class: "permission-group-options",
                                                for &(perm_key, perm_label) in grp.items {
                                                    {
                                                        let p_key = perm_key.to_string();
                                                        let is_chk = permissions.read().contains(&p_key);

                                                        rsx! {
                                                            label { key: "{perm_key}", class: "settings-checkbox-item", style: "padding: 8px 10px; font-size: 12.5px;",
                                                                input {
                                                                    r#type: "checkbox",
                                                                    checked: is_chk,
                                                                    onchange: move |e: FormEvent| {
                                                                        let mut cur = permissions.read().clone();
                                                                        if e.checked() {
                                                                            if !cur.contains(&p_key) {
                                                                                cur.push(p_key.clone());
                                                                            }
                                                                        } else {
                                                                            cur.retain(|x| x != &p_key);
                                                                        }
                                                                        permissions.set(cur);
                                                                    },
                                                                }
                                                                span { "{perm_label}" }
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
                    }
                }
            }
        }
    }
}
