//! # Listagem de Documentos Emitidos e Assinaturas (Frontend)
//!
//! Controla a visualização dos termos e contratos emitidos para pacientes,
//! status de assinatura em tempo real, auditoria de integridade e botões de ação e pré-visualização de PDF.

use crate::api::delete_patient_document;
use crate::components::icons::{
    IconCheckCircle, IconClock, IconEye, IconFile, IconFolder, IconLock, IconQrCode, IconRefresh,
    IconSearch, IconShieldCheck, IconSignature, IconTrash,
};
use crate::utils::resolve_file_url;
use dioxus::prelude::*;
use shared::documents::{DocumentsKpis, PatientDocument};

/// Formata o tipo de documento técnico para exibição em português.
fn format_doc_type(doc_type: &str) -> &'static str {
    match doc_type.to_lowercase().as_str() {
        "consent" => "Consentimento (TCLE)",
        "contract" => "Contrato de Serviços",
        "orthodontics" => "Ortodontia / Alinhadores",
        "implant" => "Implantodontia / Cirurgia",
        "prescription" => "Receituário / Atestado",
        "budget" => "Orçamento",
        _ => "Documento Clínico",
    }
}

/// Formata a data ISO para o padrão brasileiro DD/MM/AAAA.
fn format_br_date(date_str: &str) -> String {
    let clean = date_str.chars().take(10).collect::<String>();
    let parts: Vec<&str> = clean.split('-').collect();
    if parts.len() == 3 {
        format!("{}/{}/{}", parts[2], parts[1], parts[0])
    } else {
        clean
    }
}

/// Componente de exibição de documentos emitidos e auditoria de assinaturas.
#[component]
pub fn DocumentsListSection(
    documents: Vec<PatientDocument>,
    kpis: DocumentsKpis,
    is_loading: bool,
    search_query: Signal<String>,
    status_filter: Signal<String>,
    can_write: bool,
    can_delete: bool,
    token: String,
    clinic_id: String,
    on_open_issue_modal: EventHandler<()>,
    reload_trigger: Signal<usize>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
    pdf_preview_target: Signal<Option<(String, String)>>,
    qr_modal_doc: Signal<Option<PatientDocument>>,
) -> Element {
    let mut audit_modal_doc = use_signal(|| None::<PatientDocument>);
    let mut delete_target_id = use_signal(|| None::<(String, String)>);
    let mut is_deleting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_confirm_delete = move |_| {
        let Some((d_id, _)) = delete_target_id() else { return; };
        let t = tok.clone();
        let c = cid.clone();
        let mut del_sig = delete_target_id;
        let mut rel_sig = reload_trigger;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut is_del = is_deleting;

        is_del.set(true);
        spawn(async move {
            match delete_patient_document(&t, &d_id, &c).await {
                Ok(_) => {
                    del_sig.set(None);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Documento removido com sucesso!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir documento: {}", e)));
                }
            }
            is_del.set(false);
        });
    };

    let s_query = search_query().to_lowercase();
    let s_filter = status_filter();

    let filtered_docs: Vec<&PatientDocument> = documents
        .iter()
        .filter(|d| {
            let patient_completed = !d.requires_patient_signature || d.patient_signed_at.is_some();
            let doctor_completed = !d.requires_doctor_signature || d.doctor_signed_at.is_some();
            let has_any_sign = d.patient_signed_at.is_some() || d.doctor_signed_at.is_some();
            let is_done = d.status == "signed"
                || d.status == "completed"
                || (patient_completed && doctor_completed && (d.requires_patient_signature || d.requires_doctor_signature) && has_any_sign);

            if s_filter == "pending" {
                !is_done
            } else if s_filter == "signed" {
                is_done
            } else {
                true
            }
        })
        .filter(|d| {
            if s_query.is_empty() {
                return true;
            }
            d.title.to_lowercase().contains(&s_query)
                || d.patient_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&s_query)
        })
        .collect();

    rsx! {
        div { class: "emitted-docs-view",
            // KPI Summary Row
            div { class: "agenda-kpi-row mb-4",
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                        IconFile { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Total Emitidos" }
                    }
                    div { class: "agenda-kpi-val", "{kpis.total_documents}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                        IconLock { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Pendentes Assinatura" }
                    }
                    div { class: "agenda-kpi-val kpi-pending", "{kpis.pending_signatures}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                        IconCheckCircle { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Assinados (Válidos)" }
                    }
                    div { class: "agenda-kpi-val kpi-completed", "{kpis.completed_signed}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-progress",
                        IconSignature { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Modelos Ativos" }
                    }
                    div { class: "agenda-kpi-val kpi-progress", "{kpis.templates_count}" }
                }
            }

            // Toolbar
            div { class: "view-toolbar",
                div { class: "search-input-wrap",
                    IconSearch { size: 18, color: "#94a3b8".to_string() }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Buscar documento ou paciente...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                div { class: "toolbar-actions",
                    button {
                        class: "btn-refresh",
                        onclick: move |_| reload_trigger.set(reload_trigger() + 1),
                        title: "Recarregar documentos",
                        IconRefresh { size: 16, color: "#475569".to_string() }
                    }

                    select {
                        class: "select-field doc-filter-select",
                        value: "{status_filter}",
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "all", "Todos os Status" }
                        option { value: "pending", "Apenas Pendentes" }
                        option { value: "signed", "Apenas Assinados" }
                    }

                    if can_write {
                        button {
                            class: "btn-primary",
                            onclick: move |_| on_open_issue_modal.call(()),
                            IconSignature { size: 16, color: "#ffffff".to_string() }
                            span { " Emitir Novo Documento" }
                        }
                    }
                }
            }

            if is_loading {
                div { class: "loading-card",
                    div { class: "loading-spinner" }
                    p { "Carregando documentos emitidos..." }
                }
            } else if filtered_docs.is_empty() {
                div { class: "empty-state-card",
                    div { class: "empty-state-icon-box",
                        IconFolder { size: 32, color: "currentColor".to_string() }
                    }
                    h3 { "Nenhum documento emitido" }
                    p { "Emita contratos e termos para assinatura digital imediata de pacientes e profissionais." }
                }
            } else {
                div { class: "table-container",
                    table { class: "modern-table",
                        thead {
                            tr {
                                th { "Documento / Contrato" }
                                th { "Tipo" }
                                th { "Data de Emissão" }
                                th { "Assinatura do Paciente" }
                                th { "Assinatura Médica" }
                                th { "Status Geral" }
                                th { class: "text-right", "Ações" }
                            }
                        }
                        tbody {
                            for doc in filtered_docs {
                                {
                                    let doc_clone = doc.clone();
                                    let doc_for_audit = doc.clone();
                                    let doc_id_for_del = doc.id.clone();
                                    let doc_title_for_del = doc.title.clone();
                                    let patient_completed = !doc.requires_patient_signature || doc.patient_signed_at.is_some();
                                    let doctor_completed = !doc.requires_doctor_signature || doc.doctor_signed_at.is_some();
                                    let has_any_sign = doc.patient_signed_at.is_some() || doc.doctor_signed_at.is_some();
                                    let is_signed = doc.status == "signed"
                                        || doc.status == "completed"
                                        || (patient_completed && doctor_completed && (doc.requires_patient_signature || doc.requires_doctor_signature) && has_any_sign);

                                    let raw_url = if let Some(ref s) = doc.signed_pdf_url {
                                        s.clone()
                                    } else {
                                        doc.original_pdf_url.clone()
                                    };
                                    let pdf_url_to_preview = resolve_file_url(&raw_url);
                                    let pdf_title = doc.title.clone();
                                    let is_anamnesis = doc.document_type == "anamnesis"
                                        || doc.document_type == "anamnese"
                                        || doc.title.to_lowercase().contains("anamnes");

                                    rsx! {
                                        tr { key: "{doc.id}",
                                            td {
                                                div { class: "doc-title-cell",
                                                    IconFile { size: 18, color: "#0052cc".to_string() }
                                                    span { class: "font-semibold", "{doc.title}" }
                                                }
                                            }
                                            td {
                                                span { class: "badge-doc-type", "{format_doc_type(&doc.document_type)}" }
                                            }
                                            td { "{format_br_date(&doc.created_at)}" }
                                            td {
                                                if !doc.requires_patient_signature {
                                                    span { class: "badge-status-neutral", "Não Exigida" }
                                                } else if doc.patient_signed_at.is_some() {
                                                    span { class: "badge-status-completed",
                                                        IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                        span { " Assinado" }
                                                    }
                                                } else {
                                                    span { class: "badge-status-pending", "Pendente" }
                                                }
                                            }
                                            td {
                                                if !doc.requires_doctor_signature {
                                                    span { class: "badge-status-neutral", "Não Exigida" }
                                                } else if doc.doctor_signed_at.is_some() {
                                                    span { class: "badge-status-completed",
                                                        IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                        span { " Assinado" }
                                                    }
                                                } else {
                                                    span { class: "badge-status-pending", "Pendente" }
                                                }
                                            }
                                            td {
                                                if is_signed {
                                                    span { class: "badge-status-completed", "Concluído (Válido)" }
                                                } else {
                                                    span { class: "badge-status-pending", "Aguardando E-Sign" }
                                                }
                                            }
                                            td { class: "text-right",
                                                div { class: "table-actions-row",
                                                    button {
                                                        class: "btn-action-icon",
                                                        title: "Abrir QR Code / Link de Assinatura",
                                                        onclick: {
                                                            let d = doc_clone.clone();
                                                            let mut qr_sig = qr_modal_doc;
                                                            move |_| qr_sig.set(Some(d.clone()))
                                                        },
                                                        IconQrCode { size: 16, color: "#0052cc".to_string() }
                                                    }
                                                    if !is_anamnesis && !pdf_url_to_preview.is_empty() {
                                                        button {
                                                            class: "btn-action-icon",
                                                            title: "Visualizar Documento / PDF",
                                                            onclick: {
                                                                let u = pdf_url_to_preview.clone();
                                                                let tit = pdf_title.clone();
                                                                let mut preview_sig = pdf_preview_target;
                                                                move |_| preview_sig.set(Some((u.clone(), tit.clone())))
                                                            },
                                                            IconEye { size: 16, color: "#475569".to_string() }
                                                        }
                                                    }
                                                    button {
                                                        class: "btn-action-icon text-success",
                                                        title: "Auditoria & Criptografia SHA-256",
                                                        onclick: move |_| audit_modal_doc.set(Some(doc_for_audit.clone())),
                                                        IconShieldCheck { size: 16, color: "#10b981".to_string() }
                                                    }
                                                    if can_delete {
                                                        button {
                                                            class: "btn-action-icon text-danger",
                                                            title: "Excluir Documento",
                                                            onclick: move |_| delete_target_id.set(Some((doc_id_for_del.clone(), doc_title_for_del.clone()))),
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

            // Modal de Auditoria e Certificação Digital
            if let Some(ref doc) = *audit_modal_doc.read() {
                {
                    let is_completed = doc.status == "signed" || doc.status == "completed";
                    let checksum_display = doc.checksum_sha256.clone().unwrap_or_else(|| "Pendente de consolidação final".into());

                    rsx! {
                        div { class: "modal-overlay",
                        div { class: "action-modal audit-modal-card", style: "width: 660px; max-width: 95vw;",
                            div { class: "modal-header",
                                div { class: "audit-header-content",
                                    div { class: "audit-header-icon-box",
                                        IconShieldCheck { size: 24, color: "#10b981".to_string() }
                                    }
                                    div {
                                        h2 { class: "modal-title", "Auditoria de Integridade Criptográfica (SHA-256)" }
                                        p { class: "modal-subtitle", "Comprovação de autenticidade, carimbo de tempo e não-repúdio (Lei 14.063/2020)." }
                                    }
                                }
                                button { class: "modal-close", onclick: move |_| audit_modal_doc.set(None), "×" }
                            }

                            div { class: "modal-body",
                                // Banner de Status Jurídico
                                div { class: if is_completed { "audit-status-banner valid" } else { "audit-status-banner pending" },
                                    div { class: "banner-left-row",
                                        if is_completed {
                                            IconCheckCircle { size: 20, color: "#10b981".to_string() }
                                            div {
                                                strong { "Documento Concluído com Validade Jurídica Plena" }
                                                p { "Assinaturas coletadas e integridade assegurada por hash criptográfico SHA-256." }
                                            }
                                        } else {
                                            IconClock { size: 20, color: "#f59e0b".to_string() }
                                            div {
                                                strong { "Documento Pendente de Assinaturas" }
                                                p { "Aguardando conclusão das etapas de assinatura para registro definitivo do carimbo de tempo." }
                                            }
                                        }
                                    }
                                }

                                // Grid Estruturado de Detalhes
                                div { class: "audit-details-grid",
                                    div { class: "audit-card-item full-width",
                                        span { class: "audit-item-label", "Título do Documento" }
                                        div { class: "audit-item-value font-semibold", "{doc.title}" }
                                    }

                                    div { class: "audit-card-item",
                                        span { class: "audit-item-label", "Token Único (UUID)" }
                                        div { class: "audit-item-value font-mono", "{doc.signing_token}" }
                                    }

                                    div { class: "audit-card-item",
                                        span { class: "audit-item-label", "Status do Documento" }
                                        div { class: "audit-item-value",
                                            if is_completed {
                                                span { class: "badge-status-completed", "Assinado & Válido" }
                                            } else {
                                                span { class: "badge-status-pending", "Aguardando Assinatura" }
                                            }
                                        }
                                    }

                                    div { class: "audit-card-item full-width",
                                        div { class: "audit-item-top",
                                            span { class: "audit-item-label", "Checksum Criptográfico SHA-256" }
                                            span { class: "audit-pill-badge", "Imutável" }
                                        }
                                        div { class: "audit-hash-box",
                                            IconLock { size: 14, color: "#0052cc".to_string() }
                                            code { class: "audit-hash-text", "{checksum_display}" }
                                        }
                                    }

                                    div { class: "audit-card-item",
                                        span { class: "audit-item-label", "Assinatura do Paciente" }
                                        div { class: "audit-signer-status",
                                            if let Some(ref pat_time) = doc.patient_signed_at {
                                                IconCheckCircle { size: 16, color: "#10b981".to_string() }
                                                div {
                                                    span { class: "signer-status-text text-success font-semibold", "Assinado Digitalmente" }
                                                    span { class: "signer-time-text", "{format_br_date(pat_time)}" }
                                                }
                                            } else {
                                                IconClock { size: 16, color: "#94a3b8".to_string() }
                                                span { class: "signer-status-text text-muted", "Não assinado" }
                                            }
                                        }
                                    }

                                    div { class: "audit-card-item",
                                        span { class: "audit-item-label", "Assinatura Cirurgião-Dentista" }
                                        div { class: "audit-signer-status",
                                            if !doc.requires_doctor_signature {
                                                IconCheckCircle { size: 16, color: "#64748b".to_string() }
                                                div {
                                                    span { class: "signer-status-text text-muted font-semibold", "Não Exigida" }
                                                    span { class: "signer-time-text", "Dispensada no modelo" }
                                                }
                                            } else if let Some(ref doc_time) = doc.doctor_signed_at {
                                                IconCheckCircle { size: 16, color: "#10b981".to_string() }
                                                div {
                                                    span { class: "signer-status-text text-success font-semibold", "Assinado Digitalmente" }
                                                    span { class: "signer-time-text", "{format_br_date(doc_time)}" }
                                                }
                                            } else {
                                                IconClock { size: 16, color: "#94a3b8".to_string() }
                                                span { class: "signer-status-text text-muted", "Pendente" }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "modal-footer",
                                button { class: "btn-primary", onclick: move |_| audit_modal_doc.set(None), "Fechar Certificado" }
                            }
                        }
                    }
                }
            }
        }

            // Modal de Exclusão de Documento
            if let Some((_, ref d_title)) = *delete_target_id.read() {
                div { class: "modal-overlay",
                    div { class: "action-modal delete-modal-card",
                        div { class: "modal-header",
                            h2 { class: "modal-title text-danger", "Excluir Documento Emitido" }
                            button { class: "modal-close", onclick: move |_| delete_target_id.set(None), "×" }
                        }
                        div { class: "modal-body",
                            p { "Tem certeza que deseja excluir o documento ", strong { "{d_title}" }, "?" }
                            p { class: "text-muted font-xs mt-2", "Esta ação cancelará o token de assinatura digital e invalidará o link público." }
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
