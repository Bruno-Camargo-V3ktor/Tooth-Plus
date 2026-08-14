use crate::api;
use crate::components::icons::{IconChevronDown, IconEdit, IconLock, IconPower, IconTrash};
use crate::components::ui_blocks::{ActionModal, PageHeader};
use crate::permissions::{self, ALL_PERMISSION_GROUPS};
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::users::{CreateUserRequest, ToggleStatusRequest, UpdateUserRequest, UserResponse};

#[component]
pub fn UsersView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = permissions::has_permission(&sess, &clinic, "users:read");
    let can_write = permissions::has_permission(&sess, &clinic, "users:write");
    let can_manage_status = permissions::has_permission(&sess, &clinic, "users:manage_status");

    let token = sess
        .as_ref()
        .map(|s| s.token.clone())
        .unwrap_or_default();

    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let available_clinics = sess
        .as_ref()
        .map(|s| s.clinics.clone())
        .unwrap_or_default();

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

    let search_query = use_signal(|| String::new());
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

    let filtered_users: Vec<UserResponse> = current_users
        .into_iter()
        .filter(|u| {
            u.full_name.to_lowercase().contains(&search_val.to_lowercase())
                || u.username.to_lowercase().contains(&search_val.to_lowercase())
        })
        .collect();

    let t_toggle = token.clone();
    let cid_toggle = clinic_id.clone();

    rsx! {
        div { class: "users-page-container",

            if let Some(msg) = toast_msg() {
                div { class: "toast-error",
                    span { "{msg}" }
                    button { class: "toast-close-btn", onclick: move |_| toast_msg.set(None), "×" }
                }
            }

            PageHeader {
                search_query,
                show_new_btn: can_write,
                btn_text: "Novo Usuário".to_string(),
                on_new: move |_| {
                    selected_user.set(None);
                    is_form_modal_open.set(true);
                }
            }

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

            if can_write {
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

            if can_write {
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

    rsx! {
        div { class: "user-card-row",
            div { class: "user-info-section",
                div { class: "user-avatar-small", "{user.full_name.chars().next().unwrap_or('U')}" }
                div { class: "user-text-group",
                    h3 { class: "user-fullname", "{user.full_name}" }
                    span { class: "user-username", "@{user.username}" }
                }
            }
            div { class: "user-role-section",
                div { class: "role-badge role-{user.role}", "{user.role}" }
            }
            div { class: "user-status-section",
                div { class: if user.is_active { "status-badge active" } else { "status-badge inactive" },
                    span { class: if user.is_active { "status-dot green" } else { "status-dot red" } }
                    { if user.is_active { "Ativo" } else { "Inativo" } }
                }
            }
            div { class: "user-perms-section",
                IconLock { size: 16, color: "#94a3b8".to_string() }
                span { class: "perms-count-label", "{perms_count} permissões" }
            }
            div { class: "user-actions-section",
                if can_write {
                    button { class: "icon-action-btn edit-btn-row", onclick: move |_| on_edit.call(u_edit.clone()), IconEdit { size: 18, color: "currentColor".to_string() } }
                    button { class: "icon-action-btn delete-btn-row", onclick: move |_| on_delete.call(u_delete.clone()), IconTrash { size: 18, color: "currentColor".to_string() } }
                }
                if can_manage_status {
                    button { class: "icon-action-btn toggle-btn-row", onclick: move |_| on_toggle.call(u_toggle.clone()), IconPower { size: 18, color: "currentColor".to_string() } }
                }
            }
        }
    }
}

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
    let title = if is_edit_mode { "Editar Membro" } else { "Adicionar Novo Membro" };

    let mut full_name = use_signal(|| String::new());
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut document_cpf = use_signal(|| String::new());
    let mut professional_registry = use_signal(|| String::new());
    let mut role = use_signal(|| "dentist".to_string());
    let mut selected_permissions = use_signal(|| Vec::<String>::new());
    let mut selected_clinics = use_signal(|| Vec::<String>::new());
    let mut is_submitting = use_signal(|| false);

    let cid_eff = clinic_id.clone();
    use_effect(use_reactive(&editing_user, move |opt_u| {
        if let Some(u) = opt_u {
            full_name.set(u.full_name);
            username.set(u.username);
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

    let handle_submit = move |_| {
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
        ActionModal {
            is_open: is_open(),
            title: title.to_string(),
            on_close: move |_| is_open.set(false),

            div { class: "form-grid",
                div { class: "input-group-wrapper full-width",
                    label { "Nome Completo" }
                    input {
                        class: "modern-input-field",
                        placeholder: "Ex: Dra. Ana Lima",
                        value: "{full_name}",
                        oninput: move |e| full_name.set(e.value())
                    }
                }
                div { class: "input-group-wrapper",
                    label { "Nome de Usuário (Login)" }
                    input {
                        class: "modern-input-field",
                        placeholder: "Ex: ana.lima",
                        disabled: is_edit_mode,
                        value: "{username}",
                        oninput: move |e| username.set(e.value())
                    }
                }
                div { class: "input-group-wrapper",
                    label { if is_edit_mode { "Senha (em branco = manter atual)" } else { "Senha Inicial" } }
                    input {
                        class: "modern-input-field",
                        r#type: "password",
                        placeholder: if is_edit_mode { "••••••••" } else { "Senha de acesso" },
                        value: "{password}",
                        oninput: move |e| password.set(e.value())
                    }
                }
                div { class: "input-group-wrapper",
                    label { "CPF" }
                    input {
                        class: "modern-input-field",
                        placeholder: "000.000.000-00",
                        value: "{document_cpf}",
                        oninput: move |e| document_cpf.set(e.value())
                    }
                }
                div { class: "input-group-wrapper",
                    label { "Registro Profissional (CRO)" }
                    input {
                        class: "modern-input-field",
                        placeholder: "CRO-SP 12345",
                        value: "{professional_registry}",
                        oninput: move |e| professional_registry.set(e.value())
                    }
                }
                div { class: "input-group-wrapper full-width",
                    label { "Cargo / Função" }
                    select {
                        class: "modern-input-field modern-select",
                        value: "{role}",
                        onchange: move |e| role.set(e.value()),
                        option { value: "dentist", "Dentista" }
                        option { value: "receptionist", "Recepcionista" }
                        option { value: "assistant", "Auxiliar" }
                        option { value: "admin", "Administrador" }
                        option { value: "manager", "Gerente" }
                        option { value: "other", "Outro" }
                    }
                }

                div { class: "input-group-wrapper full-width",
                    h4 { class: "form-section-title", "Unidades de Acesso" }
                    div { class: "permissions-container",
                        div { class: "perm-category-body perm-clinics-list",
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
                                                        current.retain(|id| id != &cid);
                                                    }
                                                    selected_clinics.set(current);
                                                }
                                            }
                                            span { "{c.trading_name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "input-group-wrapper full-width",
                    h4 { class: "form-section-title", "Permissões Granulares (PBAC)" }
                    div { class: "permissions-container",
                        for group in ALL_PERMISSION_GROUPS {
                            PermissionCategoryAccordion {
                                key: "{group.label}",
                                title: group.label.to_string(),
                                permissions: group.items.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
                                selected: selected_permissions,
                            }
                        }
                    }
                }
            }

            div { class: "modal-footer-actions",
                button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                button {
                    class: "btn-primary",
                    disabled: is_submitting(),
                    onclick: handle_submit,
                    if is_submitting() { "Salvando..." } else { "Salvar Membro" }
                }
            }
        }
    }
}

#[component]
fn PermissionCategoryAccordion(
    title: String,
    permissions: Vec<(String, String)>,
    mut selected: Signal<Vec<String>>,
) -> Element {
    let mut expanded = use_signal(|| false);
    let chevron_class = if expanded() { "chevron-icon rotated" } else { "chevron-icon" };

    let total = permissions.len();
    let current_selected = selected();
    let count_selected = permissions.iter().filter(|(k, _)| current_selected.contains(k)).count();
    let all_keys: Vec<String> = permissions.iter().map(|(k, _)| k.clone()).collect();
    let all_selected = count_selected == total && total > 0;

    rsx! {
        div { class: "perm-category-box",
            div { class: "perm-category-header", onclick: move |_| expanded.set(!expanded()),
                div { class: "perm-header-left",
                    span { class: "perm-category-title", "{title}" }
                    span { class: if count_selected > 0 { "perm-badge-count active" } else { "perm-badge-count" },
                        "{count_selected}/{total}"
                    }
                }
                IconChevronDown { size: 16, color: "currentColor".to_string(), class: chevron_class.to_string() }
            }
            if expanded() {
                div { class: "perm-category-body",
                    div { class: "perm-category-actions-bar",
                        button {
                            class: "btn-text-sm",
                            r#type: "button",
                            onclick: {
                                let keys = all_keys.clone();
                                move |_| {
                                    let mut curr = selected();
                                    if all_selected {
                                        curr.retain(|k| !keys.contains(k));
                                    } else {
                                        for k in &keys {
                                            if !curr.contains(k) {
                                                curr.push(k.clone());
                                            }
                                        }
                                    }
                                    selected.set(curr);
                                }
                            },
                            if all_selected { "Desmarcar todos" } else { "Marcar todos" }
                        }
                    }
                    div { class: "perm-checkbox-grid",
                        for (perm_key, perm_label) in permissions {
                            PermissionItemCheckbox {
                                key: "{perm_key}",
                                perm_key: perm_key.clone(),
                                perm_label: perm_label.clone(),
                                selected,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PermissionItemCheckbox(
    perm_key: String,
    perm_label: String,
    mut selected: Signal<Vec<String>>,
) -> Element {
    let pk = perm_key.clone();
    let is_checked = selected().contains(&pk);

    rsx! {
        label { class: "perm-checkbox-item",
            input {
                r#type: "checkbox",
                checked: is_checked,
                onchange: move |e: FormEvent| {
                    let mut current = selected();
                    if e.checked() {
                        if !current.contains(&pk) {
                            current.push(pk.clone());
                        }
                    } else {
                        current.retain(|p| p != &pk);
                    }
                    selected.set(current);
                }
            }
            span { "{perm_label}" }
            span { class: "perm-key-label", "({perm_key})" }
        }
    }
}

#[component]
fn UserDeleteModal(
    is_open: Signal<bool>,
    user: Signal<Option<UserResponse>>,
    token: String,
    clinic_id: String,
    on_success: EventHandler<()>,
    on_error: EventHandler<String>,
) -> Element {
    let mut is_deleting = use_signal(|| false);

    let handle_delete = move |_| {
        if let Some(u) = user() {
            is_deleting.set(true);
            let t = token.clone();
            let cid = clinic_id.clone();
            let uid = u.id.clone();
            spawn(async move {
                match api::delete_user(&t, &uid, &cid).await {
                    Ok(_) => on_success.call(()),
                    Err(e) => on_error.call(e),
                }
                is_deleting.set(false);
            });
        }
    };

    rsx! {
        ActionModal {
            is_open: is_open(),
            title: "Confirmar Exclusão".to_string(),
            on_close: move |_| is_open.set(false),

            div {
                p { class: "delete-modal-text", "Atenção: tem certeza que deseja remover o acesso deste membro nesta unidade? Esta ação não apaga o usuário do sistema." }
                div { class: "modal-footer-actions",
                    button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                    button {
                        class: "btn-danger",
                        disabled: is_deleting(),
                        onclick: handle_delete,
                        if is_deleting() { "Removendo..." } else { "Sim, remover acesso" }
                    }
                }
            }
        }
    }
}
