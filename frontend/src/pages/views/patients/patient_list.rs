//! # Listagem e KPIs de Pacientes (Frontend)
//!
//! Controla a visualização em tabela dos pacientes cadastrados, barra de pesquisa,
//! cartões de métricas (KPIs) e ações de abertura do prontuário ou exclusão.

use crate::api::delete_patient;
use crate::components::icons::{
    IconCheckCircle, IconEye, IconFile, IconFolder, IconRefresh, IconSearch, IconSignature,
    IconTooth, IconTrash, IconUsers,
};
use dioxus::prelude::*;
use shared::patients::{Patient, PatientKpis};

/// Componente de listagem dos pacientes com indicadores superiores.
#[component]
pub fn PatientListSection(
    patients: Vec<Patient>,
    kpis: PatientKpis,
    is_loading: bool,
    search_query: Signal<String>,
    can_write: bool,
    can_delete: bool,
    token: String,
    clinic_id: String,
    on_open_create_modal: EventHandler<()>,
    on_select_patient: EventHandler<String>,
    reload_trigger: Signal<usize>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut delete_target_id = use_signal(|| None::<(String, String)>);
    let mut is_deleting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_confirm_delete = move |_| {
        let Some((p_id, _)) = delete_target_id() else { return; };
        let t = tok.clone();
        let c = cid.clone();
        let mut del_sig = delete_target_id;
        let mut rel_sig = reload_trigger;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut is_del = is_deleting;

        is_del.set(true);
        spawn(async move {
            match delete_patient(&t, &p_id, &c).await {
                Ok(_) => {
                    del_sig.set(None);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Paciente removido com sucesso!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir paciente: {}", e)));
                }
            }
            is_del.set(false);
        });
    };

    let s_query = search_query().to_lowercase();
    let filtered_patients: Vec<&Patient> = patients
        .iter()
        .filter(|p| {
            if s_query.is_empty() {
                return true;
            }
            p.full_name.to_lowercase().contains(&s_query)
                || p.document_cpf.contains(&s_query)
                || p.phone.contains(&s_query)
        })
        .collect();

    rsx! {
        div { class: "patients-list-view",
            // KPI Cards Row
            div { class: "kpi-grid",
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-blue-light",
                        IconUsers { size: 24, color: "#0052cc".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Total de Pacientes" }
                        h3 { class: "kpi-value", "{kpis.total_patients}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-emerald-light",
                        IconCheckCircle { size: 24, color: "#10b981".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Novos no Mês" }
                        h3 { class: "kpi-value", "{kpis.new_this_month}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-amber-light",
                        IconSignature { size: 24, color: "#f59e0b".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Docs. Pendentes Assinatura" }
                        h3 { class: "kpi-value", "{kpis.pending_documents_count}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-purple-light",
                        IconTooth { size: 24, color: "#8b5cf6".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Em Tratamento Ativo" }
                        h3 { class: "kpi-value", "{kpis.active_treatments_count}" }
                    }
                }
            }

            // View Toolbar
            div { class: "view-toolbar",
                div { class: "search-input-wrap",
                    IconSearch { size: 18, color: "#94a3b8".to_string() }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Buscar por nome, CPF ou telefone...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value())
                    }
                }

                div { class: "toolbar-actions",
                    button {
                        class: "btn-refresh",
                        onclick: move |_| reload_trigger.set(reload_trigger() + 1),
                        title: "Recarregar lista",
                        IconRefresh { size: 16, color: "#475569".to_string() }
                    }

                    if can_write {
                        button {
                            class: "btn-primary",
                            onclick: move |_| on_open_create_modal.call(()),
                            IconUsers { size: 16, color: "#ffffff".to_string() }
                            span { " Novo Paciente" }
                        }
                    }
                }
            }

            if is_loading {
                div { class: "loading-card",
                    div { class: "loading-spinner" }
                    p { "Carregando prontuários..." }
                }
            } else if filtered_patients.is_empty() {
                div { class: "empty-state-card",
                    div { class: "empty-state-icon-box",
                        IconFolder { size: 32, color: "currentColor".to_string() }
                    }
                    h3 { "Nenhum paciente localizado" }
                    p { "Utilize a busca acima ou clique em 'Novo Paciente' para cadastrar." }
                }
            } else {
                div { class: "table-container",
                    table { class: "modern-table",
                        thead {
                            tr {
                                th { "Paciente" }
                                th { "CPF / Documento" }
                                th { "Telefone / Contato" }
                                th { "Convênio" }
                                th { "Cadastro" }
                                th { class: "text-right", "Ações" }
                            }
                        }
                        tbody {
                            for pat in filtered_patients {
                                {
                                    let pid = pat.id.clone();
                                    let pid_for_del = pat.id.clone();
                                    let pname_for_del = pat.full_name.clone();
                                    let on_sel = on_select_patient.clone();

                                    rsx! {
                                        tr { key: "{pat.id}",
                                            td {
                                                div { class: "patient-avatar-cell",
                                                    div { class: "patient-avatar-circle",
                                                        "{pat.full_name.chars().next().unwrap_or('P')}"
                                                    }
                                                    div { class: "patient-name-col",
                                                        span { class: "patient-name-title font-semibold", "{pat.full_name}" }
                                                        if let Some(ref g) = pat.gender {
                                                            span { class: "badge-gender", "{g}" }
                                                        }
                                                    }
                                                }
                                            }
                                            td {
                                                span { class: "font-mono font-xs", "{pat.document_cpf}" }
                                            }
                                            td { "{pat.phone}" }
                                            td {
                                                span { class: "badge-outline",
                                                    "{pat.insurance_plan.as_deref().unwrap_or(\"Particular\")}"
                                                }
                                            }
                                            td { "{pat.created_at.chars().take(10).collect::<String>()}" }
                                            td { class: "text-right",
                                                div { class: "table-actions-row",
                                                    button {
                                                        class: "btn-action-icon",
                                                        title: "Abrir Prontuário Completo",
                                                        onclick: move |_| on_sel.call(pid.clone()),
                                                        IconEye { size: 16, color: "#0052cc".to_string() }
                                                    }
                                                    if can_delete {
                                                        button {
                                                            class: "btn-action-icon text-danger",
                                                            title: "Excluir Paciente",
                                                            onclick: move |_| delete_target_id.set(Some((pid_for_del.clone(), pname_for_del.clone()))),
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
                }
            }

            // Modal de Exclusão de Paciente
            if let Some((_, ref p_name)) = *delete_target_id.read() {
                div { class: "modal-overlay",
                    div { class: "action-modal delete-modal-card",
                        div { class: "modal-header",
                            h2 { class: "modal-title text-danger", "Excluir Paciente" }
                            button { class: "modal-close", onclick: move |_| delete_target_id.set(None), "×" }
                        }
                        div { class: "modal-body",
                            p { "Tem certeza que deseja excluir o prontuário de ", strong { "{p_name}" }, "?" }
                            p { class: "text-muted font-xs mt-2", "Esta ação apagará o histórico clínico, exames e termos associados." }
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
