//! # Tabela e Listagem de Pacientes (Frontend)
//!
//! Exibe os KPIs do topo, barra de busca, filtros por tipo de convênio
//! e a listagem tabular com ações de abrir prontuário e exclusão.

use crate::api::delete_patient;
use crate::components::icons::{
    IconCheckCircle, IconEdit, IconFolder, IconHeartPulse, IconLock, IconRefresh, IconSearch,
    IconTooth, IconTrash, IconUsers,
};
use dioxus::prelude::*;
use shared::patients::{Patient, PatientKpis};

/// Formata a data ISO para o padrão brasileiro DD/MM/YYYY.
fn format_br_date(date_str: &str) -> String {
    if date_str.len() >= 10 {
        let parts: Vec<&str> = date_str[0..10].split('-').collect();
        if parts.len() == 3 {
            return format!("{}/{}/{}", parts[2], parts[1], parts[0]);
        }
    }
    date_str.to_string()
}

/// Retorna uma classe de cor de avatar baseada no caractere inicial.
fn get_avatar_color_class(initial: char) -> &'static str {
    match initial.to_ascii_uppercase() {
        'A'..='E' => "patient-row-avatar avatar-emerald",
        'F'..='J' => "patient-row-avatar avatar-blue",
        'K'..='O' => "patient-row-avatar avatar-indigo",
        'P'..='T' => "patient-row-avatar avatar-amber",
        _ => "patient-row-avatar avatar-purple",
    }
}

/// Componente de listagem dos pacientes com KPIs e filtros.
#[component]
pub fn PatientListSection(
    kpis: PatientKpis,
    patients: Vec<Patient>,
    is_loading: bool,
    search_query: Signal<String>,
    reload_trigger: Signal<usize>,
    can_write: bool,
    can_delete: bool,
    can_manage_templates: bool,
    token: String,
    clinic_id: String,
    on_select_patient: EventHandler<String>,
    on_open_create_modal: EventHandler<()>,
    on_open_templates_modal: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut delete_target_id = use_signal(|| None::<(String, String)>);
    let mut is_deleting = use_signal(|| false);
    let mut filter_plan = use_signal(|| "all".to_string());

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_confirm_delete = move |_| {
        let Some((ref p_id, _)) = *delete_target_id.read() else {
            return;
        };
        let p_id_clone = p_id.clone();
        let t = tok.clone();
        let c = cid.clone();
        let mut target_sig = delete_target_id;
        let mut rel_sig = reload_trigger;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut is_del = is_deleting;

        is_del.set(true);
        spawn(async move {
            match delete_patient(&t, &p_id_clone, &c).await {
                Ok(_) => {
                    target_sig.set(None);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Paciente excluído com sucesso!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir paciente: {}", e)));
                }
            }
            is_del.set(false);
        });
    };

    let total_count = patients.len();
    let filtered_patients: Vec<Patient> = patients
        .into_iter()
        .filter(|p| {
            let plan = p.insurance_plan.as_deref().unwrap_or("Particular");
            match filter_plan().as_str() {
                "particular" => plan.eq_ignore_ascii_case("particular") || plan.is_empty(),
                "convenio" => !plan.eq_ignore_ascii_case("particular") && !plan.is_empty(),
                _ => true,
            }
        })
        .collect();

    rsx! {
        div { class: "patients-list-view",
            // 1. TOP: 4 Minimalist Horizontal KPI Cards
            div { class: "agenda-kpi-row",
                // 1. TOTAL DE PACIENTES
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                        IconUsers { size: 18, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "TOTAL DE PACIENTES" }
                    }
                    div { class: "agenda-kpi-val", "{kpis.total_patients}" }
                }

                // 2. NOVOS NO MÊS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                        IconCheckCircle { size: 18, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "NOVOS NO MÊS" }
                    }
                    div { class: "agenda-kpi-val", "{kpis.new_this_month}" }
                }

                // 3. DOCS. PENDENTES DE ASSINATURA
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                        IconEdit { size: 18, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "DOCS. PENDENTES DE ASSINATURA" }
                    }
                    div { class: "agenda-kpi-val", "{kpis.pending_documents_count}" }
                }

                // 4. EM TRATAMENTO ATIVO
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-progress",
                        IconTooth { size: 18, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "EM TRATAMENTO ATIVO" }
                    }
                    div { class: "agenda-kpi-val", "{kpis.active_treatments_count}" }
                }
            }

            // 2. View Toolbar
            div { class: "view-toolbar",
                div { class: "search-input-wrap",
                    IconSearch { size: 18, color: "#94a3b8".to_string() }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Buscar paciente por nome, CPF ou telefone...",
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

                    if can_manage_templates {
                        button {
                            class: "btn-secondary",
                            onclick: move |_| on_open_templates_modal.call(()),
                            IconHeartPulse { size: 16, color: "currentColor".to_string() }
                            span { " Modelos de Anamnese" }
                        }
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

            // 3. Filter Pills Row
            div { class: "patient-filter-pills-row",
                button {
                    class: if filter_plan() == "all" { "patient-filter-pill active" } else { "patient-filter-pill" },
                    onclick: move |_| filter_plan.set("all".to_string()),
                    "Todos os Pacientes ({total_count})"
                }
                button {
                    class: if filter_plan() == "particular" { "patient-filter-pill active" } else { "patient-filter-pill" },
                    onclick: move |_| filter_plan.set("particular".to_string()),
                    "Particular"
                }
                button {
                    class: if filter_plan() == "convenio" { "patient-filter-pill active" } else { "patient-filter-pill" },
                    onclick: move |_| filter_plan.set("convenio".to_string()),
                    "Com Convênio"
                }
            }

            // 4. Patients Table List
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
                                th { "PACIENTE" }
                                th { "DOCUMENTO (CPF / RG)" }
                                th { "TELEFONE / WHATSAPP" }
                                th { "PLANO / CONVÊNIO" }
                                th { "CADASTRO" }
                                th { class: "text-right", "AÇÕES" }
                            }
                        }
                        tbody {
                            for pat in filtered_patients {
                                {
                                    let pid = pat.id.clone();
                                    let pid_for_del = pat.id.clone();
                                    let pname_for_del = pat.full_name.clone();
                                    let on_sel = on_select_patient.clone();
                                    let initial = pat.full_name.chars().next().unwrap_or('P');
                                    let avatar_class = get_avatar_color_class(initial);
                                    let is_particular = pat.insurance_plan.as_deref().map(|p| p.eq_ignore_ascii_case("Particular")).unwrap_or(true);
                                    let plan_name = pat.insurance_plan.as_deref().unwrap_or("Particular");
                                    let clean_phone: String = pat.phone.chars().filter(|c| c.is_ascii_digit()).collect();
                                    let wa_url = format!("https://wa.me/55{}", clean_phone);

                                    let doc_display = if let Some(ref cpf) = pat.document_cpf {
                                        format!("CPF: {}", cpf)
                                    } else if let Some(ref rg) = pat.document_rg {
                                        format!("RG: {}", rg)
                                    } else {
                                        "Não informado".to_string()
                                    };

                                    rsx! {
                                        tr { key: "{pat.id}",
                                            td {
                                                div { class: "patient-cell-info",
                                                    div { class: "{avatar_class}",
                                                        "{initial}"
                                                    }
                                                    div {
                                                        p { class: "patient-table-name", "{pat.full_name}" }
                                                        p { class: "patient-table-email", "{pat.email.as_deref().unwrap_or(\"sem e-mail\")}" }
                                                    }
                                                }
                                            }
                                            td {
                                                span { class: "cpf-protected-pill",
                                                    IconLock { size: 12, color: "#64748b".to_string() }
                                                    span { "{doc_display}" }
                                                }
                                            }
                                            td {
                                                div { class: "patient-phone-cell",
                                                    span { "{pat.phone}" }
                                                    a {
                                                        class: "btn-wa-link",
                                                        href: "{wa_url}",
                                                        target: "_blank",
                                                        title: "Conversar no WhatsApp",
                                                        "💬"
                                                    }
                                                }
                                            }
                                            td {
                                                if is_particular {
                                                    span { class: "badge-insurance-particular", "Particular" }
                                                } else {
                                                    span { class: "badge-insurance-plan", "{plan_name}" }
                                                }
                                            }
                                            td { "{format_br_date(&pat.created_at)}" }
                                            td { class: "text-right",
                                                div { class: "table-actions-row",
                                                    button {
                                                        class: "btn-open-prontuario-table",
                                                        title: "Abrir Prontuário Completo",
                                                        onclick: move |_| on_sel.call(pid.clone()),
                                                        IconFolder { size: 14, color: "#ffffff".to_string() }
                                                        span { " Prontuário" }
                                                    }
                                                    if can_delete {
                                                        button {
                                                            class: "btn-delete-row-table",
                                                            title: "Excluir Paciente",
                                                            onclick: move |_| delete_target_id.set(Some((pid_for_del.clone(), pname_for_del.clone()))),
                                                            IconTrash { size: 16, color: "currentColor".to_string() }
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
                        div { class: "settings-header",
                            h2 { class: "settings-title text-danger", "Excluir Paciente" }
                            button { class: "close-btn", onclick: move |_| delete_target_id.set(None), "×" }
                        }
                        div { class: "settings-content",
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

