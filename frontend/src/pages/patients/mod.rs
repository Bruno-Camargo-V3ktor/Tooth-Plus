//! # Módulo de Gestão de Pacientes (Tooth Plus V2)
//!
//! Tabela completa de pacientes, busca rápida, prontuário detalhado com anamnese,
//! e integração com o modal reutilizável `PatientFormModal`.

use crate::api::patients::PatientsApi;
use crate::components::patient_form_modal::PatientFormModal;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconPlus, IconSearch, IconUser};
use dioxus::prelude::*;
use shared::patients::{Patient, PatientDetailsResponse, PatientKpis};

const STYLE: Asset = asset!("/src/pages/patients/style.css");

#[derive(Clone, PartialEq)]
enum DetailsTab {
    General,
    Anamnesis,
    Treatments,
}

#[component]
pub fn PatientsView() -> Element {
    let mut toast = consume_context::<ToastState>();

    let mut search_query = use_signal(String::new);
    let mut patients_list = use_signal(Vec::<Patient>::new);
    let mut kpis = use_signal(|| PatientKpis {
        total_patients: 0,
        new_this_month: 0,
        pending_documents_count: 0,
        active_treatments_count: 0,
    });
    let mut is_loading = use_signal(|| true);
    let mut show_new_modal = use_signal(|| false);
    let mut selected_patient_id = use_signal(|| Option::<String>::None);
    let mut selected_patient_details = use_signal(|| Option::<PatientDetailsResponse>::None);
    let mut details_tab = use_signal(|| DetailsTab::General);
    let mut is_loading_details = use_signal(|| false);

    // Carrega a lista de pacientes
    let load_patients = {
        let mut patients_sig = patients_list.clone();
        let mut kpis_sig = kpis.clone();
        let mut loading_sig = is_loading.clone();
        let search_sig = search_query.clone();
        let mut toast_sig = toast.clone();

        move || {
            let q = search_sig.read().clone();
            let q_param = if q.trim().is_empty() { None } else { Some(q) };
            let mut p_sig = patients_sig.clone();
            let mut k_sig = kpis_sig.clone();
            let mut l_sig = loading_sig.clone();
            let mut t_sig = toast_sig.clone();

            spawn(async move {
                l_sig.set(true);
                match PatientsApi::list_patients(q_param.as_deref()).await {
                    Ok(resp) => {
                        p_sig.set(resp.items);
                        k_sig.set(resp.kpis);
                    }
                    Err(e) => {
                        t_sig.show(format!("Erro ao buscar pacientes: {}", e), ToastVariant::Error);
                    }
                }
                l_sig.set(false);
            });
        }
    };

    // Efeito inicial para carregar pacientes
    use_effect({
        let mut lp = load_patients.clone();
        move || {
            lp();
        }
    });

    // Função para abrir detalhes do paciente
    let open_details = {
        let mut pid_sig = selected_patient_id.clone();
        let mut pdet_sig = selected_patient_details.clone();
        let mut loading_det_sig = is_loading_details.clone();
        let mut toast_det_sig = toast.clone();

        move |patient_id: String| {
            pid_sig.set(Some(patient_id.clone()));
            loading_det_sig.set(true);
            let mut pdet = pdet_sig.clone();
            let mut l_det = loading_det_sig.clone();
            let mut t_det = toast_det_sig.clone();

            spawn(async move {
                match PatientsApi::get_patient_details(&patient_id).await {
                    Ok(details) => {
                        pdet.set(Some(details));
                    }
                    Err(e) => {
                        t_det.show(format!("Erro ao carregar prontuário: {}", e), ToastVariant::Error);
                    }
                }
                l_det.set(false);
            });
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "patients-page",

            // 1. KPI Cards
            div { class: "patients-kpi-grid",
                div { class: "kpi-card",
                    span { class: "kpi-card-label", "Total de Pacientes" }
                    span { class: "kpi-card-value", "{kpis.read().total_patients}" }
                    span { class: "kpi-card-sub", "Cadastros ativos" }
                }
                div { class: "kpi-card",
                    span { class: "kpi-card-label", "Novos no Mês" }
                    span { class: "kpi-card-value", "{kpis.read().new_this_month}" }
                    span { class: "kpi-card-sub", "+12% vs mês anterior" }
                }
                div { class: "kpi-card",
                    span { class: "kpi-card-label", "Em Tratamento" }
                    span { class: "kpi-card-value", "{kpis.read().active_treatments_count}" }
                    span { class: "kpi-card-sub", "Consultas agendadas" }
                }
                div { class: "kpi-card",
                    span { class: "kpi-card-label", "Documentos Pendentes" }
                    span { class: "kpi-card-value", "{kpis.read().pending_documents_count}" }
                    span { class: "kpi-card-sub", "Aguardando assinatura" }
                }
            }

            // 2. Toolbar & Busca
            div { class: "patients-toolbar",
                div { class: "patients-search-box",
                    IconSearch { size: 18, color: "#94a3b8".to_string() }
                    input {
                        class: "patients-search-input",
                        r#type: "text",
                        placeholder: "Buscar por nome, CPF ou celular...",
                        value: "{search_query}",
                        oninput: {
                            let mut lp = load_patients.clone();
                            move |e: Event<FormData>| {
                                search_query.set(e.value());
                                lp();
                            }
                        }
                    }
                }

                div { class: "patients-actions",
                    button {
                        class: "btn-new-patient",
                        onclick: move |_| show_new_modal.set(true),
                        IconPlus { size: 16, color: "#ffffff".to_string() }
                        span { "Novo Paciente" }
                    }
                }
            }

            // 3. Tabela de Pacientes
            div { class: "patients-table-container",
                if is_loading() {
                    div { class: "empty-state",
                        div { class: "empty-state-icon", "👥" }
                        p { class: "empty-state-title", "Carregando pacientes..." }
                    }
                } else if patients_list.read().is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-state-icon", "🔍" }
                        p { class: "empty-state-title", "Nenhum paciente encontrado" }
                        p { class: "empty-state-desc", "Tente buscar com outros termos ou cadastre um novo paciente acima." }
                    }
                } else {
                    table { class: "patients-table",
                        thead {
                            tr {
                                th { "Paciente" }
                                th { "CPF" }
                                th { "Telefone" }
                                th { "Convênio / Plano" }
                                th { "Data de Cadastro" }
                            }
                        }
                        tbody {
                            for patient in patients_list.read().iter() {
                                {
                                    let p = patient.clone();
                                    let pid = p.id.clone();
                                    let mut op = open_details.clone();
                                    let initial = p.full_name.chars().next().unwrap_or('P').to_string();
                                    let cpf_display = p.document_cpf.clone().unwrap_or_else(|| "Não informado".to_string());
                                    let plan_display = p.insurance_plan.clone().unwrap_or_else(|| "Particular".to_string());
                                    let is_particular = p.insurance_plan.is_none();
                                    let created_fmt = p.created_at.split('T').next().unwrap_or(&p.created_at).to_string();
                                    let email_display = p.email.clone().unwrap_or_default();

                                    rsx! {
                                        tr {
                                            key: "{pid}",
                                            class: "patient-row",
                                            onclick: move |_| op(pid.clone()),

                                            td {
                                                div { class: "patient-cell-name",
                                                    div { class: "patient-avatar", "{initial}" }
                                                    div {
                                                        div { class: "patient-meta-name", "{p.full_name}" }
                                                        div { class: "patient-meta-sub", "{email_display}" }
                                                    }
                                                }
                                            }
                                            td { "{cpf_display}" }
                                            td { "{p.phone}" }
                                            td {
                                                span {
                                                    class: if is_particular { "badge-particular" } else { "badge-plan" },
                                                    "{plan_display}"
                                                }
                                            }
                                            td { "{created_fmt}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 4. Modal de Novo Paciente (reutilizável)
            if *show_new_modal.read() {
                PatientFormModal {
                    on_save: {
                        let mut lp = load_patients.clone();
                        let mut toast_created = toast.clone();
                        move |saved_id: String| {
                            show_new_modal.set(false);
                            toast_created.show("Paciente cadastrado com sucesso!", ToastVariant::Success);
                            lp();
                        }
                    },
                    on_close: move |_| show_new_modal.set(false),
                }
            }

            // 5. Modal / Prontuário Detalhado do Paciente
            if let Some(details) = selected_patient_details.read().clone() {
                div { class: "modal-overlay",
                    onclick: move |_| {
                        selected_patient_details.set(None);
                        selected_patient_id.set(None);
                    },

                    div {
                        class: "modal-box patient-details-box",
                        onclick: move |e| e.stop_propagation(),

                        // Header do Prontuário
                        div { class: "patient-profile-header",
                            div { class: "patient-profile-avatar",
                                "{details.patient.full_name.chars().next().unwrap_or('P')}"
                            }
                            div { class: "patient-profile-info",
                                h2 { "{details.patient.full_name}" }
                                div { class: "patient-profile-pills",
                                    span { "📱 {details.patient.phone}" }
                                    if let Some(ref cpf) = details.patient.document_cpf {
                                        span { "• CPF: {cpf}" }
                                    }
                                    if let Some(ref plan) = details.patient.insurance_plan {
                                        span { "• Convênio: {plan}" }
                                    }
                                }
                            }
                            button {
                                class: "modal-close-btn",
                                style: "margin-left: auto;",
                                onclick: move |_| {
                                    selected_patient_details.set(None);
                                    selected_patient_id.set(None);
                                },
                                "✕"
                            }
                        }

                        // Barra de Abas do Prontuário
                        div { class: "tab-underline-bar", style: "padding: 0 24px; background: #1e2433;",
                            button {
                                class: if *details_tab.read() == DetailsTab::General { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                                onclick: move |_| details_tab.set(DetailsTab::General),
                                "Dados Cadastrais"
                            }
                            button {
                                class: if *details_tab.read() == DetailsTab::Anamnesis { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                                onclick: move |_| details_tab.set(DetailsTab::Anamnesis),
                                "Anamnese & Alertas"
                            }
                            button {
                                class: if *details_tab.read() == DetailsTab::Treatments { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                                onclick: move |_| details_tab.set(DetailsTab::Treatments),
                                "Procedimentos ({details.treatments.len()})"
                            }
                        }

                        // Conteúdo da Aba
                        div { class: "patient-details-content",
                            match *details_tab.read() {
                                DetailsTab::General => rsx! {
                                    div { class: "patient-info-grid",
                                        div { class: "info-item-box",
                                            span { class: "info-item-label", "Data de Nascimento" }
                                            span { class: "info-item-val", "{details.patient.birth_date.as_deref().unwrap_or(\"Não informada\")}" }
                                        }
                                        div { class: "info-item-box",
                                            span { class: "info-item-label", "Sexo / Gênero" }
                                            span { class: "info-item-val", "{details.patient.gender.as_deref().unwrap_or(\"Não informado\")}" }
                                        }
                                        div { class: "info-item-box",
                                            span { class: "info-item-label", "E-mail" }
                                            span { class: "info-item-val", "{details.patient.email.as_deref().unwrap_or(\"Não informado\")}" }
                                        }
                                        div { class: "info-item-box",
                                            span { class: "info-item-label", "Profissão" }
                                            span { class: "info-item-val", "{details.patient.profession.as_deref().unwrap_or(\"Não informada\")}" }
                                        }
                                        div { class: "info-item-box",
                                            span { class: "info-item-label", "Endereço" }
                                            span { class: "info-item-val",
                                                "{details.patient.address_street.as_deref().unwrap_or(\"Logradouro não inf.\")}, {details.patient.address_number.as_deref().unwrap_or(\"S/N\")} - {details.patient.address_city.as_deref().unwrap_or(\"São Paulo\")}"
                                            }
                                        }
                                        div { class: "info-item-box",
                                            span { class: "info-item-label", "Contato de Emergência" }
                                            span { class: "info-item-val",
                                                "{details.patient.emergency_contact_name.as_deref().unwrap_or(\"Nenhum\")}"
                                            }
                                        }
                                    }
                                },
                                DetailsTab::Anamnesis => rsx! {
                                    div { style: "display: flex; flex-direction: column; gap: 14px;",
                                        div { class: "anamnesis-alert-row",
                                            div { class: "anamnesis-chip chip-success", "✓ Sem alergias a medicamentos relatadas" }
                                            div { class: "anamnesis-chip chip-warning", "⚠ Pressão arterial monitorada (13/8)" }
                                            div { class: "anamnesis-chip chip-danger", "⛔ Paciente alérgico a látex" }
                                        }

                                        if let Some(ref anam) = details.anamnesis {
                                            div { class: "patient-info-grid", style: "margin-top: 10px;",
                                                div { class: "info-item-box",
                                                    span { class: "info-item-label", "Queixa Principal" }
                                                    span { class: "info-item-val", "{anam.chief_complaint.as_deref().unwrap_or(\"Revisão semestral\")}" }
                                                }
                                                div { class: "info-item-box",
                                                    span { class: "info-item-label", "Status da Ficha" }
                                                    span { class: "info-item-val", { if anam.signed_at.is_some() { "Completa e assinada" } else { "Pendente de assinatura" } } }
                                                }
                                            }
                                        } else {
                                            div { class: "empty-state", style: "padding: 24px;",
                                                p { class: "empty-state-title", "Anamnese não preenchida" }
                                                p { class: "empty-state-desc", "O paciente ainda não completou o questionário médico de saúde." }
                                            }
                                        }
                                    }
                                },
                                DetailsTab::Treatments => rsx! {
                                    if details.treatments.is_empty() {
                                        div { class: "empty-state", style: "padding: 24px;",
                                            p { class: "empty-state-title", "Nenhum procedimento registrado" }
                                            p { class: "empty-state-desc", "Inicie um novo tratamento ou vincule uma consulta na agenda." }
                                        }
                                    } else {
                                        div { style: "display: flex; flex-direction: column; gap: 8px;",
                                            for treat in details.treatments.iter() {
                                                div {
                                                    key: "{treat.id}",
                                                    class: "info-item-box",
                                                    style: "flex-direction: row; justify-content: space-between; align-items: center;",
                                                    div {
                                                        div { style: "font-weight: 700; color: #f8fafc;", "{treat.procedure_name}" }
                                                        div { style: "font-size: 12px; color: #94a3b8;", "Dente: {treat.tooth_number.as_deref().unwrap_or(\"Geral\")}" }
                                                    }
                                                    span { class: "badge badge-blue", "{treat.status}" }
                                                }
                                            }
                                        }
                                    }
                                },
                            }
                        }

                        // Footer do Prontuário
                        div { class: "modal-footer",
                            button {
                                class: "btn-modal-ghost",
                                onclick: move |_| {
                                    selected_patient_details.set(None);
                                    selected_patient_id.set(None);
                                },
                                "Fechar Prontuário"
                            }
                        }
                    }
                }
            }
        }
    }
}
