//! # Gestão de Equipe, Usuários e Permissões Granulares (Frontend)
//!
//! Exibe os indicadores da equipe clínica, listagem com busca e filtros,
//! e modal estruturado em seções com matriz de permissões por módulo.

use crate::api;
use crate::components::icons::{
    IconBuilding, IconCheckCircle, IconChevronDown, IconEdit, IconLock, IconPlus, IconPower,
    IconRefresh, IconSearch, IconShieldCheck, IconTooth, IconTrash, IconUsers,
};
use crate::permissions::{self, ALL_PERMISSION_GROUPS};
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::users::{CreateUserRequest, ToggleStatusRequest, UpdateUserRequest, UserResponse};

/// Componente principal da visualização de usuários e equipe.
#[component]
pub fn UsersView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = permissions::has_permission(&sess, &clinic, "users:read");
    let can_write = permissions::has_permission(&sess, &clinic, "users:write");
    let can_manage_status = permissions::has_permission(&sess, &clinic, "users:manage_status");

    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();
    let clinic_id = clinic.as_ref().map(|c| c.clinic_id.clone()).unwrap_or_default();
    let available_clinics = sess.as_ref().map(|s| s.clinics.clone()).unwrap_or_default();

    let tok_res = token.clone();
    let cid_res = clinic_id.clone();
    let mut users_resource = use_resource(move || {
        let t = tok_res.clone();
        let cid = cid_res.clone();
        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                vec![]
            } else {
                api::fetch_users(&t, &cid).await.unwrap_or_default()
            }
        }
    });

    let mut search_query = use_signal(String::new);
    let mut role_filter = use_signal(|| "all".to_string());
    let mut is_form_modal_open = use_signal(|| false);
    let mut is_delete_modal_open = use_signal(|| false);
    let mut selected_user = use_signal(|| None::<UserResponse>);
    let mut toast_msg = use_signal(|| None::<String>);

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não tem permissão para visualizar a equipe desta unidade." }
            }
        };
    }

    let search_val = search_query();
    let current_users = users_resource().unwrap_or_default();

    // KPIs da Equipe
    let total_users = current_users.len();
    let dentists_count = current_users.iter().filter(|u| u.role == "dentist").count();
    let admins_count = current_users.iter().filter(|u| u.role == "admin" || u.role == "manager").count();
    let active_count = current_users.iter().filter(|u| u.is_active).count();

    // Filtragem
    let filtered_users: Vec<UserResponse> = current_users
        .into_iter()
        .filter(|u| {
            let matches_search = search_val.is_empty()
                || u.full_name.to_lowercase().contains(&search_val.to_lowercase())
                || u.username.to_lowercase().contains(&search_val.to_lowercase())
                || u.role.to_lowercase().contains(&search_val.to_lowercase());

            let matches_role = match role_filter().as_str() {
                "dentist" => u.role == "dentist",
                "admin" => u.role == "admin" || u.role == "manager",
                "active" => u.is_active,
                _ => true,
            };

            matches_search && matches_role
        })
        .collect();

    let t_toggle = token.clone();
    let cid_toggle = clinic_id.clone();

    rsx! {
        div { class: "users-page-container",
            // Toast de Feedback
            if let Some(ref msg) = *toast_msg.read() {
                div { class: "toast-error",
                    span { "{msg}" }
                    button { class: "toast-close-btn", onclick: move |_| toast_msg.set(None), "×" }
                }
            }

            // 1. TOP: 4 KPI Cards Minimalistas Padronizados
            div { class: "agenda-kpi-row",
                // 1. TOTAL DE MEMBROS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                        IconUsers { size: 18, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "TOTAL DE MEMBROS" }
                    }
                    div { class: "agenda-kpi-val", "{total_users}" }
                }

                // 2. DENTISTAS / CLÍNICOS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                        IconTooth { size: 18, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "DENTISTAS / CLÍNICOS" }
                    }
                    div { class: "agenda-kpi-val", "{dentists_count}" }
                }

                // 3. ADMINISTRADORES
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                        IconShieldCheck { size: 18, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "ADMINISTRADORES" }
                    }
                    div { class: "agenda-kpi-val", "{admins_count}" }
                }

                // 4. USUÁRIOS ATIVOS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-progress",
                        IconCheckCircle { size: 18, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "USUÁRIOS ATIVOS" }
                    }
                    div { class: "agenda-kpi-val", "{active_count}" }
                }
            }

            // 2. View Toolbar com Barra de Busca e Botões Separados
            div { class: "view-toolbar",
                div { class: "search-input-wrap",
                    IconSearch { size: 18, color: "#94a3b8".to_string() }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Buscar por nome, login @username ou cargo...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value())
                    }
                }

                div { class: "toolbar-actions",
                    button {
                        class: "btn-refresh",
                        onclick: move |_| users_resource.restart(),
                        title: "Recarregar lista",
                        IconRefresh { size: 16, color: "#475569".to_string() }
                    }

                    if can_write {
                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                selected_user.set(None);
                                is_form_modal_open.set(true);
                            },
                            IconPlus { size: 16, color: "#ffffff".to_string() }
                            span { " Novo Usuário" }
                        }
                    }
                }
            }

            // Filtros Rápidos por Cargo
            div { class: "users-filter-pills-row",
                button {
                    class: if role_filter() == "all" { "filter-pill active" } else { "filter-pill" },
                    onclick: move |_| role_filter.set("all".to_string()),
                    "Todos ({total_users})"
                }
                button {
                    class: if role_filter() == "dentist" { "filter-pill active" } else { "filter-pill" },
                    onclick: move |_| role_filter.set("dentist".to_string()),
                    "Dentistas ({dentists_count})"
                }
                button {
                    class: if role_filter() == "admin" { "filter-pill active" } else { "filter-pill" },
                    onclick: move |_| role_filter.set("admin".to_string()),
                    "Administradores ({admins_count})"
                }
                button {
                    class: if role_filter() == "active" { "filter-pill active" } else { "filter-pill" },
                    onclick: move |_| role_filter.set("active".to_string()),
                    "Ativos ({active_count})"
                }
            }

            // 3. Listagem de Usuários
            if filtered_users.is_empty() {
                div { class: "empty-state-card",
                    div { class: "empty-state-icon-box",
                        IconUsers { size: 32, color: "currentColor".to_string() }
                    }
                    h3 { "Nenhum usuário encontrado" }
                    p { "Nenhum membro da equipe corresponde aos critérios de busca ou filtros selecionados." }
                }
            } else {
                div { class: "users-list-wrapper",
                    for user in filtered_users {
                        UserRow {
                            key: "{user.id}",
                            user: user.clone(),
                            can_write,
                            can_manage_status,
                            on_edit: move |u| {
                                selected_user.set(Some(u));
                                is_form_modal_open.set(true);
                            },
                            on_delete: move |u| {
                                selected_user.set(Some(u));
                                is_delete_modal_open.set(true);
                            },
                            on_toggle: {
                                let t = t_toggle.clone();
                                let cid = cid_toggle.clone();
                                move |u: UserResponse| {
                                    let t = t.clone();
                                    let cid = cid.clone();
                                    spawn(async move {
                                        let req = ToggleStatusRequest { is_active: !u.is_active };
                                        if let Err(err) = api::toggle_user_status(&t, &u.id, &cid, req).await {
                                            toast_msg.set(Some(err));
                                        }
                                        users_resource.restart();
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Modais de Ação
            if is_form_modal_open() {
                UserFormModal {
                    is_open: is_form_modal_open,
                    user: selected_user,
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    available_clinics: available_clinics.clone(),
                    on_success: move |_| {
                        is_form_modal_open.set(false);
                        users_resource.restart();
                    },
                    on_error: move |err| toast_msg.set(Some(err))
                }
            }

            if is_delete_modal_open() {
                UserDeleteModal {
                    is_open: is_delete_modal_open,
                    user: selected_user,
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    on_success: move |_| {
                        is_delete_modal_open.set(false);
                        users_resource.restart();
                    },
                    on_error: move |err| toast_msg.set(Some(err))
                }
            }
        }
    }
}

/// Linha de Usuário com Avatar colorido e botões de ação estilizados.
#[component]
fn UserRow(
    user: UserResponse,
    can_write: bool,
    can_manage_status: bool,
    on_edit: EventHandler<UserResponse>,
    on_delete: EventHandler<UserResponse>,
    on_toggle: EventHandler<UserResponse>,
) -> Element {
    let u_edit = user.clone();
    let u_delete = user.clone();
    let u_toggle = user.clone();
    let perms_count = user.permissions.len();

    let initial = user.full_name.chars().next().unwrap_or('U');
    let role_display = match user.role.as_str() {
        "admin" => "Administrador",
        "dentist" => "Dentista",
        "receptionist" => "Recepcionista",
        "assistant" => "Auxiliar",
        "manager" => "Gerente",
        _ => "Outro",
    };

    rsx! {
        div { class: "user-card-row",
            div { class: "user-info-section",
                div { class: "user-avatar-small", "{initial}" }
                div { class: "user-text-group",
                    h3 { class: "user-fullname", "{user.full_name}" }
                    span { class: "user-username", "@{user.username}" }
                }
            }
            div { class: "user-role-section",
                div { class: "role-badge role-{user.role}", "{role_display}" }
            }
            div { class: "user-status-section",
                div { class: if user.is_active { "status-badge active" } else { "status-badge inactive" },
                    span { class: if user.is_active { "status-dot green" } else { "status-dot red" } }
                    span { if user.is_active { "Ativo" } else { "Inativo" } }
                }
            }
            div { class: "user-perms-section",
                IconLock { size: 14, color: "#64748b".to_string() }
                span { class: "perms-count-label ml-1", "{perms_count} permissões" }
            }
            div { class: "user-actions-section",
                if can_write {
                    button {
                        class: "btn-action-icon",
                        title: "Editar Usuário e Permissões",
                        onclick: move |_| on_edit.call(u_edit.clone()),
                        IconEdit { size: 16, color: "#475569".to_string() }
                    }
                    button {
                        class: "btn-action-icon btn-action-danger",
                        title: "Remover Acesso",
                        onclick: move |_| on_delete.call(u_delete.clone()),
                        IconTrash { size: 16, color: "#ef4444".to_string() }
                    }
                }
                if can_manage_status {
                    button {
                        class: if user.is_active { "btn-action-icon btn-action-power-active" } else { "btn-action-icon btn-action-power-inactive" },
                        title: if user.is_active { "Desativar Usuário" } else { "Ativar Usuário" },
                        onclick: move |_| on_toggle.call(u_toggle.clone()),
                        IconPower { size: 16, color: "currentColor".to_string() }
                    }
                }
            }
        }
    }
}

/// Modal Estruturado de Cadastro e Edição de Membro com Separação Perfeita de Configurações.
#[component]
fn UserFormModal(
    is_open: Signal<bool>,
    user: Signal<Option<UserResponse>>,
    token: String,
    clinic_id: String,
    available_clinics: Vec<shared::models::ClinicAccess>,
    on_success: EventHandler<()>,
    on_error: EventHandler<String>,
) -> Element {
    let editing_user = user();
    let is_edit_mode = editing_user.is_some();
    let title = if is_edit_mode {
        "Editar Membro da Equipe"
    } else {
        "Adicionar Novo Membro"
    };

    let mut full_name = use_signal(String::new);
    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut document_cpf = use_signal(String::new);
    let mut professional_registry = use_signal(String::new);
    let mut role = use_signal(|| "dentist".to_string());
    let mut selected_permissions = use_signal(Vec::<String>::new);
    let mut selected_clinics = use_signal(Vec::<String>::new);
    let mut is_submitting = use_signal(|| false);

    let cid_eff = clinic_id.clone();
    use_effect(use_reactive(&editing_user, move |opt_u| {
        if let Some(u) = opt_u {
            full_name.set(u.full_name);
            username.set(u.username);
            email.set(u.email.unwrap_or_default());
            phone.set(u.phone.unwrap_or_default());
            password.set(String::new());
            document_cpf.set(u.document_cpf);
            professional_registry.set(u.professional_registry.unwrap_or_default());
            role.set(u.role);
            selected_permissions.set(u.permissions);
            if u.clinic_ids.is_empty() {
                selected_clinics.set(vec![cid_eff.clone()]);
            } else {
                selected_clinics.set(u.clinic_ids);
            }
        } else {
            full_name.set(String::new());
            username.set(String::new());
            email.set(String::new());
            phone.set(String::new());
            password.set(String::new());
            document_cpf.set(String::new());
            professional_registry.set(String::new());
            role.set("dentist".to_string());
            selected_permissions.set(vec![
                "agenda:read".to_string(),
                "agenda:write".to_string(),
                "patients:read".to_string(),
                "patients:write".to_string(),
            ]);
            selected_clinics.set(vec![cid_eff.clone()]);
        }
    }));

    let tok_submit = token.clone();
    let cid_submit = clinic_id.clone();

    let mut handle_submit = move |_| {
        if full_name().trim().is_empty() || username().trim().is_empty() {
            on_error.call("Preencha o nome e o login do usuário.".to_string());
            return;
        }
        if !is_edit_mode && password().trim().is_empty() {
            on_error.call("Informe a senha do novo usuário.".to_string());
            return;
        }
        if selected_clinics().is_empty() {
            on_error.call("Selecione ao menos uma unidade para o membro.".to_string());
            return;
        }
        is_submitting.set(true);
        let t = tok_submit.clone();
        let cid = cid_submit.clone();
        let u_opt = user();

        spawn(async move {
            if let Some(u) = u_opt {
                let req = UpdateUserRequest {
                    full_name: Some(full_name()),
                    email: if email().trim().is_empty() { None } else { Some(email()) },
                    phone: if phone().trim().is_empty() { None } else { Some(phone()) },
                    new_password: if password().trim().is_empty() { None } else { Some(password()) },
                    document_cpf: Some(document_cpf()),
                    professional_registry: if professional_registry().trim().is_empty() {
                        None
                    } else {
                        Some(professional_registry())
                    },
                    role: Some(role()),
                    permissions: Some(selected_permissions()),
                    clinic_ids: Some(selected_clinics()),
                };
                match api::update_user(&t, &u.id, &cid, req).await {
                    Ok(_) => on_success.call(()),
                    Err(e) => on_error.call(e),
                }
            } else {
                let req = CreateUserRequest {
                    username: username(),
                    password_plain: password(),
                    full_name: full_name(),
                    email: if email().trim().is_empty() { None } else { Some(email()) },
                    phone: if phone().trim().is_empty() { None } else { Some(phone()) },
                    document_cpf: document_cpf(),
                    professional_registry: if professional_registry().trim().is_empty() {
                        None
                    } else {
                        Some(professional_registry())
                    },
                    role: role(),
                    permissions: selected_permissions(),
                    clinic_ids: selected_clinics(),
                };
                match api::create_user(&t, req).await {
                    Ok(_) => on_success.call(()),
                    Err(e) => on_error.call(e),
                }
            }
            is_submitting.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal stock-custom-modal", style: "max-width: 820px; max-height: 90vh; display: flex; flex-direction: column;",
                // Cabeçalho do Modal
                div { class: "settings-header",
                    div {
                        h2 { class: "settings-title", "{title}" }
                        p { class: "text-muted font-xs mt-1",
                            "Defina credenciais de acesso, cargo, unidades e matriz de permissões granulares."
                        }
                    }
                    button { class: "close-btn", onclick: move |_| is_open.set(false), "×" }
                }

                // Conteúdo Estruturado em 3 Seções Claras
                div { class: "settings-content", style: "overflow-y: auto; gap: 20px; padding: 20px 24px;",
                    // SEÇÃO 1: Dados Cadastrais & Credenciais
                    div { class: "user-form-section-card",
                        div { class: "user-form-section-header",
                            div { class: "user-form-section-icon",
                                IconUsers { size: 16, color: "currentColor".to_string() }
                            }
                            div { class: "user-form-section-title-wrap",
                                h3 { class: "user-form-section-title", "Identificação & Credenciais de Acesso" }
                                p { class: "user-form-section-desc", "Dados pessoais, cargo profissional e login no sistema" }
                            }
                        }

                        div { class: "form-grid-2",
                            div { class: "form-group",
                                label { "Nome Completo *" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: Dr. André Martins",
                                    value: "{full_name}",
                                    oninput: move |e| full_name.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Cargo / Função *" }
                                select {
                                    class: "form-input",
                                    value: "{role}",
                                    onchange: move |e| role.set(e.value()),
                                    option { value: "dentist", "Dentista / Clínico" }
                                    option { value: "receptionist", "Recepcionista" }
                                    option { value: "assistant", "Auxiliar Odontológico (ASB)" }
                                    option { value: "admin", "Administrador do Sistema" }
                                    option { value: "manager", "Gerente de Clínica" }
                                    option { value: "other", "Outro" }
                                }
                            }
                            div { class: "form-group",
                                label { "Nome de Usuário (Login) *" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: dr.andre",
                                    disabled: is_edit_mode,
                                    value: "{username}",
                                    oninput: move |e| username.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { if is_edit_mode { "Nova Senha" } else { "Senha Inicial *" } }
                                input {
                                    class: "form-input",
                                    r#type: "password",
                                    placeholder: if is_edit_mode { "Deixe em branco para manter" } else { "Defina a senha de acesso" },
                                    value: "{password}",
                                    oninput: move |e| password.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "CPF" }
                                input {
                                    class: "form-input",
                                    placeholder: "000.000.000-00",
                                    value: "{document_cpf}",
                                    oninput: move |e| document_cpf.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Registro Profissional (CRO)" }
                                input {
                                    class: "form-input",
                                    placeholder: "CRO-SP 12345",
                                    value: "{professional_registry}",
                                    oninput: move |e| professional_registry.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "E-mail Profissional" }
                                input {
                                    class: "form-input",
                                    r#type: "email",
                                    placeholder: "andre.martins@clinica.com.br",
                                    value: "{email}",
                                    oninput: move |e| email.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Telefone / WhatsApp" }
                                input {
                                    class: "form-input",
                                    placeholder: "(11) 98888-7777",
                                    value: "{phone}",
                                    oninput: move |e| phone.set(e.value())
                                }
                            }
                        }
                    }

                    // SEÇÃO 2: Unidades Autorizadas
                    div { class: "user-form-section-card",
                        div { class: "user-form-section-header",
                            div { class: "user-form-section-icon",
                                IconBuilding { size: 16, color: "currentColor".to_string() }
                            }
                            div { class: "user-form-section-title-wrap",
                                h3 { class: "user-form-section-title", "Unidades com Acesso Permitido" }
                                p { class: "user-form-section-desc", "Selecione as clínicas nas quais este colaborador poderá atuar" }
                            }
                        }

                        div { class: "perm-clinics-list",
                            for c in available_clinics {
                                {
                                    let cid = c.clinic_id.clone();
                                    let is_checked = selected_clinics().contains(&cid);
                                    rsx! {
                                        label { key: "{c.clinic_id}", class: "perm-checkbox-item",
                                            input {
                                                r#type: "checkbox",
                                                checked: is_checked,
                                                onchange: move |e: FormEvent| {
                                                    let mut current = selected_clinics();
                                                    if e.checked() {
                                                        if !current.contains(&cid) {
                                                            current.push(cid.clone());
                                                        }
                                                    } else {
                                                        current.retain(|x| x != &cid);
                                                    }
                                                    selected_clinics.set(current);
                                                }
                                            }
                                            span { class: "font-semibold", "{c.trading_name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // SEÇÃO 3: Matriz Granular de Permissões
                    div { class: "user-form-section-card",
                        div { class: "user-form-section-header",
                            div { class: "user-form-section-icon",
                                IconShieldCheck { size: 16, color: "currentColor".to_string() }
                            }
                            div { class: "user-form-section-title-wrap",
                                h3 { class: "user-form-section-title", "Matriz Granular de Permissões" }
                                p { class: "user-form-section-desc", "Controle fino de privilégios de leitura, escrita e exclusão por módulo" }
                            }
                        }

                        div { class: "perm-groups-grid-layout",
                            for group in ALL_PERMISSION_GROUPS.iter() {
                                {
                                    let grp = group;
                                    let grp_perms: Vec<String> = grp.items.iter().map(|(p, _)| p.to_string()).collect();
                                    let active_in_group = grp_perms.iter().filter(|p| selected_permissions().contains(p)).count();
                                    let all_active = active_in_group == grp_perms.len() && !grp_perms.is_empty();

                                    let mut is_open = use_signal(|| true);

                                    rsx! {
                                        div { key: "{grp.label}", class: "perm-category-box",
                                            div {
                                                class: "perm-category-header",
                                                onclick: move |_| is_open.set(!is_open()),
                                                div { class: "perm-header-left",
                                                    span { class: "perm-category-title", "{grp.label}" }
                                                    span {
                                                        class: if active_in_group > 0 { "perm-badge-count active" } else { "perm-badge-count" },
                                                        "{active_in_group}/{grp_perms.len()}"
                                                    }
                                                }
                                                div { class: if is_open() { "chevron-icon rotated" } else { "chevron-icon" },
                                                    IconChevronDown { size: 14, color: "#64748b".to_string(), class: None }
                                                }
                                            }

                                            if is_open() {
                                                div { class: "perm-category-body",
                                                    div { class: "perm-category-actions-bar",
                                                        button {
                                                            class: "btn-text-sm",
                                                            r#type: "button",
                                                            onclick: move |_| {
                                                                let mut current = selected_permissions();
                                                                if all_active {
                                                                    current.retain(|p| !grp_perms.contains(p));
                                                                } else {
                                                                    for p in &grp_perms {
                                                                        if !current.contains(p) {
                                                                            current.push(p.clone());
                                                                        }
                                                                    }
                                                                }
                                                                selected_permissions.set(current);
                                                            },
                                                            if all_active { "Desmarcar todas" } else { "Marcar todas" }
                                                        }
                                                    }

                                                    div { class: "perm-checkbox-grid",
                                                        for &(perm_key, perm_label) in grp.items {
                                                            {
                                                                let p_key = perm_key.to_string();
                                                                let is_checked = selected_permissions().contains(&p_key);
                                                                rsx! {
                                                                    label { key: "{perm_key}", class: "perm-checkbox-item",
                                                                        input {
                                                                            r#type: "checkbox",
                                                                            checked: is_checked,
                                                                            onchange: move |e: FormEvent| {
                                                                                let mut current = selected_permissions();
                                                                                if e.checked() {
                                                                                    if !current.contains(&p_key) {
                                                                                        current.push(p_key.clone());
                                                                                    }
                                                                                } else {
                                                                                    current.retain(|x| x != &p_key);
                                                                                }
                                                                                selected_permissions.set(current);
                                                                            }
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

                // Rodapé com Botões de Ação
                div { class: "modal-footer-actions",
                    button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        if is_submitting() { "Salvando..." } else { "Salvar Membro" }
                    }
                }
            }
        }
    }
}

/// Modal de Confirmação de Exclusão / Remoção de Acesso.
#[component]
fn UserDeleteModal(
    is_open: Signal<bool>,
    user: Signal<Option<UserResponse>>,
    token: String,
    clinic_id: String,
    on_success: EventHandler<()>,
    on_error: EventHandler<String>,
) -> Element {
    let u_opt = user();
    let Some(u) = u_opt else { return rsx! { div {} } };

    let mut is_deleting = use_signal(|| false);

    let t_del = token.clone();
    let cid_del = clinic_id.clone();
    let u_id = u.id.clone();

    let mut handle_confirm = move |_| {
        is_deleting.set(true);
        let t = t_del.clone();
        let cid = cid_del.clone();
        let uid = u_id.clone();

        spawn(async move {
            match api::delete_user(&t, &uid, &cid).await {
                Ok(_) => on_success.call(()),
                Err(e) => on_error.call(e),
            }
            is_deleting.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay", onclick: move |_| is_open.set(false),
            div { class: "action-modal stock-custom-modal", style: "max-width: 440px;", onclick: move |e| e.stop_propagation(),
                div { class: "settings-header",
                    div {
                        h2 { class: "settings-title text-danger", "Excluir Membro da Equipe" }
                    }
                    button { class: "close-btn", onclick: move |_| is_open.set(false), "×" }
                }
                div { class: "settings-content text-center py-4",
                    div { class: "delete-modal-icon-box",
                        IconTrash { size: 28, color: "#ef4444".to_string() }
                    }
                    h3 { class: "modal-confirm-title", "Excluir {u.full_name}?" }
                    p { class: "modal-confirm-desc",
                        "O usuário @{u.username} perderá o acesso definitivo ao sistema. Esta ação não pode ser desfeita."
                    }
                }
                div { class: "modal-footer-actions",
                    button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                    button {
                        class: "btn-danger",
                        disabled: is_deleting(),
                        onclick: move |e| handle_confirm(e),
                        if is_deleting() { "Excluindo..." } else { "Sim, Excluir Usuário" }
                    }
                }
            }
        }
    }
}
