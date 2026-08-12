use crate::api;
use crate::components::icons::{IconChevronDown, IconEdit, IconLock, IconPower, IconTrash};
use crate::components::ui_blocks::{ActionModal, PageHeader};
use dioxus::prelude::*;
use shared::users::{CreateUserRequest, ToggleStatusRequest, UserResponse};

#[component]
pub fn UsersView() -> Element {
    let active_clinic = consume_context::<Signal<crate::ActiveClinicState>>();
    let clinic_id = active_clinic()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut users_resource = use_resource(move || {
        let cid = clinic_id.clone();
        async move { api::fetch_users(&cid).await.unwrap_or_default() }
    });

    let mut search_query = use_signal(|| String::new());
    let mut is_form_modal_open = use_signal(|| false);
    let mut is_delete_modal_open = use_signal(|| false);
    let mut selected_user = use_signal(|| None::<UserResponse>);

    let search_val = search_query();
    let current_users = users_resource().unwrap_or_default();

    let filtered_users: Vec<UserResponse> = current_users
        .into_iter()
        .filter(|u| {
            u.full_name
                .to_lowercase()
                .contains(&search_val.to_lowercase())
        })
        .collect();

    rsx! {
        div { class: "users-page-container",

            PageHeader {
                title: "Equipe e Acessos".to_string(),
                subtitle: "Gerencie os funcionários e defina as permissões granulares por área.".to_string(),
                search_query: search_query,
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
                        on_edit: move |u| {
                            selected_user.set(Some(u));
                            is_form_modal_open.set(true);
                        },
                        on_delete: move |u| {
                            selected_user.set(Some(u));
                            is_delete_modal_open.set(true);
                        },
                        on_toggle: move |u: UserResponse| {
                            spawn(async move {
                                let req = ToggleStatusRequest { is_active: !u.is_active };
                                let _ = api::toggle_user_status(&u.id, req).await;
                                users_resource.restart();
                            });
                        }
                    }
                }
            }

            UserFormModal {
                is_open: is_form_modal_open,
                user: selected_user,
                on_success: move |_| {
                    is_form_modal_open.set(false);
                    users_resource.restart();
                }
            }

            UserDeleteModal {
                is_open: is_delete_modal_open,
                user: selected_user,
                on_success: move |_| {
                    is_delete_modal_open.set(false);
                    users_resource.restart();
                }
            }
        }
    }
}

#[component]
fn UserRow(
    user: UserResponse,
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
            div { class: "user-role-section", div { class: "role-badge role-{user.role}", "{user.role}" } }
            div { class: "user-status-section",
                div { class: if user.is_active { "status-badge active" } else { "status-badge inactive" },
                    span { class: if user.is_active { "status-dot green" } else { "status-dot red" } }
                    {if user.is_active { "Ativo" } else { "Inativo" }}
                }
            }
            div { class: "user-perms-section",
                IconLock { size: 16, color: "#94a3b8".to_string() }
                span { style: "margin-left: 6px;", "{perms_count} permissões" }
            }
            div { class: "user-actions-section",
                button { class: "icon-action-btn edit-btn-row", onclick: move |_| on_edit.call(u_edit.clone()), IconEdit { size: 18, color: "currentColor".to_string() } }
                button { class: "icon-action-btn delete-btn-row", onclick: move |_| on_delete.call(u_delete.clone()), IconTrash { size: 18, color: "currentColor".to_string() } }
                button { class: "icon-action-btn toggle-btn-row", onclick: move |_| on_toggle.call(u_toggle.clone()), IconPower { size: 18, color: "currentColor".to_string() } }
            }
        }
    }
}

#[component]
fn UserFormModal(
    is_open: Signal<bool>,
    user: Signal<Option<UserResponse>>,
    on_success: EventHandler<()>,
) -> Element {
    let title = if user().is_some() {
        "Editar Membro"
    } else {
        "Adicionar Novo Membro"
    };

    rsx! {
        ActionModal {
            is_open: is_open(),
            title: title.to_string(),
            on_close: move |_| is_open.set(false),

            div { class: "form-grid",
                div { class: "input-group-wrapper", style: "grid-column: 1 / -1;", input { class: "modern-input-field", placeholder: "Nome Completo" } }
                div { class: "input-group-wrapper", input { class: "modern-input-field", placeholder: "Login" } }
                div { class: "input-group-wrapper", input { class: "modern-input-field", r#type: "password", placeholder: "Senha Temporária" } }
                div { class: "input-group-wrapper", style: "grid-column: 1 / -1;",
                    select { class: "modern-input-field modern-select",
                        option { value: "dentist", "Dentista" }
                        option { value: "receptionist", "Recepcionista" }
                        option { value: "admin", "Administrador" }
                    }
                }
                div { style: "grid-column: 1 / -1; margin-top: 16px;",
                    h4 { style: "margin: 0 0 12px 0; font-size: 14px; color: #0f172a;", "Permissões de Acesso (PBAC)" }
                    div { class: "permissions-container",
                        PermissionCategory { title: "Módulo: Agenda".to_string(), permissions: vec!["Ler Agendamentos".into(), "Criar Agendamento".into(), "Deletar Agendamento".into()] }
                        PermissionCategory { title: "Módulo: Pacientes".to_string(), permissions: vec!["Ler Prontuário".into(), "Editar Ficha".into(), "Deletar Paciente".into()] }
                        PermissionCategory { title: "Módulo: Financeiro".to_string(), permissions: vec!["Ver Fluxo de Caixa".into(), "Lançar Receita".into(), "Estornar Pagamento".into()] }
                    }
                }
            }
            div { class: "modal-footer-actions",
                button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                button {
                    class: "btn-primary",
                    onclick: move |_| {
                        spawn(async move {
                            let _ = api::create_user(CreateUserRequest {
                                username: "".into(),
                                password_plain: "".into(),
                                full_name: "".into(),
                                role: "".into(),
                                clinic_id: "".into(),
                                permissions: vec![]
                            }).await;
                            on_success.call(());
                        });
                    },
                    "Salvar Membro"
                }
            }
        }
    }
}

#[component]
fn UserDeleteModal(
    is_open: Signal<bool>,
    user: Signal<Option<UserResponse>>,
    on_success: EventHandler<()>,
) -> Element {
    rsx! {
        ActionModal {
            is_open: is_open(),
            title: "Confirmar Exclusão".to_string(),
            on_close: move |_| is_open.set(false),

            div {
                p { class: "delete-modal-text", "Atenção: Tem certeza que deseja excluir permanentemente este usuário?" }
                div { class: "modal-footer-actions",
                    button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                    button {
                        class: "btn-danger",
                        onclick: move |_| {
                            if let Some(u) = user() {
                                spawn(async move {
                                    let _ = api::delete_user(&u.id).await;
                                    on_success.call(());
                                });
                            }
                        },
                        "Sim, excluir usuário"
                    }
                }
            }
        }
    }
}

#[component]
fn PermissionCategory(title: String, permissions: Vec<String>) -> Element {
    let mut expanded = use_signal(|| false);
    let chevron_class = if expanded() {
        "chevron-icon rotated"
    } else {
        "chevron-icon"
    };

    rsx! {
        div { class: "perm-category-box",
            div { class: "perm-category-header", onclick: move |_| expanded.set(!expanded()),
                span { class: "perm-category-title", "{title}" }
                IconChevronDown { size: 16, color: "currentColor".to_string(), class: chevron_class.to_string() }
            }
            if expanded() {
                div { class: "perm-category-body",
                    for perm in permissions {
                        label { class: "perm-checkbox-item",
                            input { r#type: "checkbox" }
                            span { "{perm}" }
                        }
                    }
                }
            }
        }
    }
}
