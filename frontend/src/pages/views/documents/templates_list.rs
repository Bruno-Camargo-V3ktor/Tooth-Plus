//! # Listagem de Modelos de Contratos / Templates (Frontend)
//!
//! Exibe os cartões de modelos cadastrados, visualização de finalidade,
//! campos de tags configurados e ações de edição, exclusão e visualização de PDF base.

use crate::api::delete_template;
use crate::components::icons::{IconEdit, IconEye, IconFile, IconSignature, IconTrash};
use dioxus::prelude::*;
use shared::documents::ContractTemplate;

/// Formata a categoria técnica do modelo para exibição amigável em português.
fn format_template_category(cat: &str) -> &'static str {
    match cat.to_lowercase().as_str() {
        "consent" => "Termo TCLE",
        "contract" => "Contrato Geral",
        "orthodontics" => "Ortodontia / Alinhadores",
        "implant" => "Implantodontia / Cirurgia",
        "prescription" => "Receituário / Atestado",
        _ => "Modelo de Termo",
    }
}

/// Componente de exibição da galeria de modelos de contratos e termos da clínica.
#[component]
pub fn TemplatesListSection(
    templates: Vec<ContractTemplate>,
    is_loading: bool,
    can_write: bool,
    can_delete: bool,
    token: String,
    clinic_id: String,
    on_open_create_template: EventHandler<()>,
    on_edit_template: EventHandler<ContractTemplate>,
    reload_trigger: Signal<usize>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
    pdf_preview_target: Signal<Option<(String, String)>>,
) -> Element {
    let mut delete_target_id = use_signal(|| None::<(String, String)>);
    let mut is_deleting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_confirm_delete = move |_| {
        let Some((t_id, _)) = delete_target_id() else { return; };
        let t = tok.clone();
        let c = cid.clone();
        let mut del_sig = delete_target_id;
        let mut rel_sig = reload_trigger;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut is_del = is_deleting;

        is_del.set(true);
        spawn(async move {
            match delete_template(&t, &t_id, &c).await {
                Ok(_) => {
                    del_sig.set(None);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Modelo de contrato excluído com sucesso!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir modelo: {}", e)));
                }
            }
            is_del.set(false);
        });
    };

    rsx! {
        div { class: "templates-view",
            div { class: "tab-header-row",
                div {
                    h3 { class: "tab-pane-title", "Modelos de Contratos e Documentos com Tags" }
                    p { class: "tab-pane-subtitle", "Cadastre modelos em PDF com substituição automática de dados e assinaturas via tags dinâmicas." }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| on_open_create_template.call(()),
                        IconSignature { size: 16, color: "#ffffff".to_string() }
                        span { " Novo Modelo de Contrato" }
                    }
                }
            }

            if is_loading {
                div { class: "loading-card",
                    div { class: "loading-spinner" }
                    p { "Carregando modelos de contratos..." }
                }
            } else if templates.is_empty() {
                div { class: "empty-state-card",
                    div { class: "empty-state-icon-box",
                        IconFile { size: 32, color: "currentColor".to_string() }
                    }
                    h3 { "Nenhum modelo cadastrado" }
                    p { "Crie modelos como Termo TCLE, Contrato de Ortodontia ou Implante com tags dinâmicas." }
                }
            } else {
                div { class: "templates-grid",
                    for tpl in templates {
                        {
                            let tpl_clone = tpl.clone();
                            let tpl_id_for_del = tpl.id.clone();
                            let tpl_title_for_del = tpl.title.clone();
                            let pdf_url = tpl.pdf_url.clone();
                            let title = tpl.title.clone();

                            rsx! {
                                div { key: "{tpl.id}", class: "template-card",
                                    div { class: "template-card-header",
                                        div {
                                            span { class: "template-cat-badge", "{format_template_category(&tpl.category)}" }
                                            h4 { class: "template-card-title", "{tpl.title}" }
                                        }
                                        div { class: "template-tags-count",
                                            IconSignature { size: 14, color: "#0052cc".to_string() }
                                            span { "E-Sign Tags" }
                                        }
                                    }

                                    if let Some(ref d) = tpl.description {
                                        p { class: "template-card-desc", "{d}" }
                                    }

                                    div { class: "template-card-footer",
                                        button {
                                            class: "btn-secondary btn-sm",
                                            style: "gap: 6px; padding: 6px 12px; font-size: 13px;",
                                            onclick: {
                                                let u = pdf_url.clone();
                                                let tit = title.clone();
                                                let mut preview_sig = pdf_preview_target;
                                                move |_| preview_sig.set(Some((u.clone(), tit.clone())))
                                            },
                                            IconEye { size: 15, color: "#0052cc".to_string() }
                                            span { "Visualizar PDF" }
                                        }
                                        div { class: "card-actions-group", style: "display: flex; gap: 8px; align-items: center;",
                                            if can_write {
                                                button {
                                                    class: "btn-action-icon",
                                                    title: "Editar Modelo",
                                                    onclick: move |_| on_edit_template.call(tpl_clone.clone()),
                                                    IconEdit { size: 16, color: "#475569".to_string() }
                                                }
                                            }
                                            if can_delete {
                                                button {
                                                    class: "btn-action-icon text-danger",
                                                    title: "Excluir Modelo",
                                                    onclick: move |_| delete_target_id.set(Some((tpl_id_for_del.clone(), tpl_title_for_del.clone()))),
                                                    IconTrash { size: 16, color: "#ef4444".to_string() }
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

            // Modal de Exclusão de Modelo
            if let Some((_, ref t_title)) = *delete_target_id.read() {
                div { class: "modal-overlay",
                    div { class: "action-modal delete-modal-card",
                        div { class: "modal-header",
                            h2 { class: "modal-title text-danger", "Excluir Modelo de Contrato" }
                            button { class: "modal-close", onclick: move |_| delete_target_id.set(None), "×" }
                        }
                        div { class: "modal-body",
                            p { "Tem certeza que deseja excluir o modelo ", strong { "{t_title}" }, "?" }
                            p { class: "text-muted font-xs mt-2", "Esta ação não afetará os documentos emitidos anteriormente com este modelo." }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| delete_target_id.set(None), "Cancelar" }
                            button {
                                class: "btn-danger",
                                disabled: is_deleting(),
                                onclick: move |e| handle_confirm_delete(e),
                                if is_deleting() { "Excluindo..." } else { "Confirmar Exclusão" }
                            }
                        }
                    }
                }
            }
        }
    }
}
