use crate::api::{
    create_patient_document, create_template, delete_patient_document, delete_template,
    fetch_documents, fetch_patients, update_template,
};
use crate::components::icons::{
    IconCheckCircle, IconExternalLink, IconEye, IconFile, IconQrCode, IconRefresh, IconSearch,
    IconShieldCheck, IconSignature, IconTooth, IconTrash,
};
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use base64::{Engine as _, engine::general_purpose};
use crate::components::icons::IconUpload;
use qrcode::QrCode;
use qrcode::render::svg;
use shared::documents::{
    CreateContractTemplateRequest, CreatePatientDocumentRequest, DocumentsKpis,
    PatientDocument, SignatureField, UpdateContractTemplateRequest,
};
use shared::patients::Patient;

fn generate_qr_svg(url: &str) -> String {
    if let Ok(code) = QrCode::new(url.as_bytes()) {
        code.render::<svg::Color>()
            .min_dimensions(180, 180)
            .dark_color(svg::Color("#0052cc"))
            .light_color(svg::Color("#ffffff"))
            .build()
    } else {
        String::new()
    }
}

fn format_br_date(date_str: &str) -> String {
    let clean = date_str.chars().take(10).collect::<String>();
    let parts: Vec<&str> = clean.split('-').collect();
    if parts.len() == 3 {
        format!("{}/{}/{}", parts[2], parts[1], parts[0])
    } else {
        clean
    }
}

#[component]
pub fn DocumentsView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = permissions::has_permission(&sess, &clinic, "documents:read");
    let can_write = permissions::has_permission(&sess, &clinic, "documents:write");
    let can_delete = permissions::has_permission(&sess, &clinic, "documents:delete");

    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();
    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar os documentos desta unidade." }
            }
        };
    }

    let mut active_main_tab = use_signal(|| "emitted".to_string()); // "emitted" | "templates"
    let mut search_query = use_signal(String::new);
    let status_filter = use_signal(|| "all".to_string());
    let mut reload_trigger = use_signal(|| 0usize);

    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    let tok_res = token.clone();
    let cid_res = clinic_id.clone();
    let documents_resource = use_resource(move || {
        let t = tok_res.clone();
        let cid = cid_res.clone();
        let st = status_filter();
        let _ = reload_trigger();
        let st_opt = if st == "all" { None } else { Some(st) };

        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                return Ok(shared::documents::DocumentsListResponse {
                    documents: vec![],
                    templates: vec![],
                    kpis: DocumentsKpis::default(),
                });
            }
            let st_ref = st_opt.as_deref();
            fetch_documents(&t, &cid, None, st_ref).await
        }
    });

    let tok_pat = token.clone();
    let cid_pat = clinic_id.clone();
    let patients_resource = use_resource(move || {
        let t = tok_pat.clone();
        let cid = cid_pat.clone();
        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                return Ok(shared::patients::PatientListResponse { items: vec![], kpis: shared::patients::PatientKpis::default(), total: 0 });
            }
            fetch_patients(&t, &cid, None).await
        }
    });

    let (documents_list, templates_list, kpis, is_loading) = match &*documents_resource.read() {
        Some(Ok(resp)) => (resp.documents.clone(), resp.templates.clone(), resp.kpis.clone(), false),
        Some(Err(_)) => (vec![], vec![], DocumentsKpis::default(), false),
        None => (vec![], vec![], DocumentsKpis::default(), true),
    };

    let patients_list = match &*patients_resource.read() {
        Some(Ok(resp)) => resp.items.clone(),
        _ => vec![],
    };

    // Modals
    let mut is_emit_modal_open = use_signal(|| false);
    let mut is_template_modal_open = use_signal(|| false);
    let mut editing_template_id = use_signal(|| None::<String>);
    let mut qr_modal_doc = use_signal(|| None::<PatientDocument>);
    let mut pdf_preview_target = use_signal(|| None::<(String, String)>);
    let mut selected_patient_obj = use_signal(|| None::<Patient>);

    // Form inputs: Emit Document
    let mut emit_patient_id = use_signal(String::new);
    let mut emit_template_id = use_signal(String::new);
    let mut emit_doc_title = use_signal(String::new);
    let mut emit_doc_type = use_signal(|| "contract".to_string());
    let mut emit_pdf_url = use_signal(String::new);
    let mut is_uploading_doc_pdf = use_signal(|| false);
    let mut uploaded_doc_pdf_name = use_signal(String::new);
    let mut is_uploading_tpl_pdf = use_signal(|| false);
    let mut uploaded_tpl_pdf_name = use_signal(String::new);

    // Form inputs: Template Editor
    let mut tpl_title = use_signal(String::new);
    let mut tpl_category = use_signal(|| "contract".to_string());
    let mut tpl_desc = use_signal(String::new);
    let mut tpl_pdf_url = use_signal(|| {
        "https://placehold.co/800x1100/ffffff/0f172a?text=Modelo+de+Contrato".to_string()
    });
    let mut tpl_signature_fields = use_signal(Vec::<SignatureField>::new);

    // New Signature Tag Form
    let mut new_tag_signer = use_signal(|| "patient".to_string());
    let new_tag_page = use_signal(|| 1u32);
    let mut new_tag_x = use_signal(|| 15.0f32);
    let mut new_tag_y = use_signal(|| 80.0f32);
    let mut new_tag_label = use_signal(|| "Assinatura do Paciente".to_string());

    let _refresh_data = move || {
        reload_trigger.set(reload_trigger() + 1);
    };

    let open_create_template_modal = move |_| {
        editing_template_id.set(None);
        tpl_title.set(String::new());
        tpl_category.set("contract".to_string());
        tpl_desc.set(String::new());
        tpl_pdf_url
            .set("https://placehold.co/800x1100/ffffff/0f172a?text=Modelo+de+Contrato".to_string());

        let default_fields = vec![
            SignatureField {
                id: "sig_pat_1".to_string(),
                signer_type: "patient".to_string(),
                page_number: 1,
                x_pct: 15.0,
                y_pct: 82.0,
                width_pct: 30.0,
                height_pct: 10.0,
                label: "Assinatura do Paciente".to_string(),
                is_required: true,
            },
            SignatureField {
                id: "sig_doc_1".to_string(),
                signer_type: "doctor".to_string(),
                page_number: 1,
                x_pct: 55.0,
                y_pct: 82.0,
                width_pct: 30.0,
                height_pct: 10.0,
                label: "Assinatura do Cirurgião-Dentista".to_string(),
                is_required: true,
            },
        ];
        tpl_signature_fields.set(default_fields);
        is_template_modal_open.set(true);
    };

    let on_submit_template = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        let mut reload_doc = reload_trigger;
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let mut reload_doc = reload_trigger;
            let edit_id = editing_template_id();

            if tpl_title().trim().is_empty() {
                error_toast.set(Some("Por favor, informe o título do modelo.".into()));
                return;
            }

            spawn(async move {
                if let Some(id) = edit_id {
                    let req = UpdateContractTemplateRequest {
                        clinic_id: cid,
                        title: tpl_title(),
                        category: tpl_category(),
                        description: if tpl_desc().is_empty() {
                            None
                        } else {
                            Some(tpl_desc())
                        },
                        pdf_url: tpl_pdf_url(),
                        signature_fields: tpl_signature_fields(),
                    };
                    if let Ok(_) = update_template(&t, &id, req).await {
                        toast_msg.set(Some("Modelo de contrato atualizado!".into()));
                        is_template_modal_open.set(false);
                        reload_doc.set(reload_doc() + 1);
                    }
                } else {
                    let req = CreateContractTemplateRequest {
                        clinic_id: cid,
                        title: tpl_title(),
                        category: tpl_category(),
                        description: if tpl_desc().is_empty() {
                            None
                        } else {
                            Some(tpl_desc())
                        },
                        pdf_url: tpl_pdf_url(),
                        signature_fields: tpl_signature_fields(),
                    };
                    if let Ok(_) = create_template(&t, req).await {
                        toast_msg.set(Some("Modelo de contrato criado com sucesso!".into()));
                        is_template_modal_open.set(false);
                        reload_doc.set(reload_doc() + 1);
                    }
                }
            });
        }
    };

    let on_submit_emit_doc = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        let mut reload_doc = reload_trigger;
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let mut reload_doc = reload_trigger;

            if emit_patient_id().trim().is_empty() {
                error_toast.set(Some("Selecione o paciente para emissão.".into()));
                return;
            }

            let tpl_id = if emit_template_id().is_empty() {
                None
            } else {
                Some(emit_template_id())
            };
            let pdf = if emit_pdf_url().is_empty() {
                None
            } else {
                Some(emit_pdf_url())
            };

            let req = CreatePatientDocumentRequest {
                clinic_id: cid,
                patient_id: emit_patient_id(),
                template_id: tpl_id,
                doctor_user_id: None,
                appointment_id: None,
                title: if emit_doc_title().is_empty() {
                    "Termo de Consentimento Odontológico".to_string()
                } else {
                    emit_doc_title()
                },
                document_type: emit_doc_type(),
                pdf_url: pdf,
            };

            spawn(async move {
                match create_patient_document(&t, req).await {
                    Ok(doc) => {
                        toast_msg.set(Some("Documento emitido com sucesso!".into()));
                        is_emit_modal_open.set(false);
                        qr_modal_doc.set(Some(doc));
                        reload_doc.set(reload_doc() + 1);
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
            });
        }
    };

    

    rsx! {
        div { class: "documents-view-container",
            // Toasts
            if let Some(ref msg) = toast_msg() {
                div { class: "toast toast-success",
                    IconCheckCircle { size: 18, color: "#10b981".to_string() }
                    span { "{msg}" }
                }
            }
            if let Some(ref err) = error_toast() {
                div { class: "toast toast-error",
                    span { "{err}" }
                }
            }

            // Top KPIs
            div { class: "kpi-grid",
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-blue-light",
                        IconFile { size: 24, color: "#0052cc".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Total de Documentos" }
                        h3 { class: "kpi-value", "{kpis.total_documents}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-amber-light",
                        IconSignature { size: 24, color: "#f59e0b".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Pendentes de Assinatura" }
                        h3 { class: "kpi-value", "{kpis.pending_signatures}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-emerald-light",
                        IconCheckCircle { size: 24, color: "#10b981".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "100% Assinados e Validados" }
                        h3 { class: "kpi-value", "{kpis.completed_signed}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-purple-light",
                        IconShieldCheck { size: 24, color: "#8b5cf6".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Modelos de Contrato" }
                        h3 { class: "kpi-value", "{kpis.templates_count}" }
                    }
                }
            }

            // Main Tabs Switcher
            div { class: "documents-tab-bar",
                button {
                    class: if active_main_tab() == "emitted" { "doc-main-tab active" } else { "doc-main-tab" },
                    onclick: move |_| active_main_tab.set("emitted".to_string()),
                    IconFile { size: 16, color: "currentColor".to_string() }
                    " Documentos & Contratos Emitidos ({documents_list.len()})"
                }
                button {
                    class: if active_main_tab() == "templates" { "doc-main-tab active" } else { "doc-main-tab" },
                    onclick: move |_| active_main_tab.set("templates".to_string()),
                    IconSignature { size: 16, color: "currentColor".to_string() }
                    " Modelos de Contratos & E-Sign ({templates_list.len()})"
                }
            }

            // TAB 1: DOCUMENTOS EMITIDOS
            if active_main_tab() == "emitted" {
                div { class: "emitted-docs-view",
                    div { class: "view-toolbar",
                        div { class: "search-input-wrap",
                            IconSearch { size: 18, color: "#94a3b8".to_string() }
                            input {
                                r#type: "text",
                                class: "search-input",
                                placeholder: "Buscar por título ou paciente...",
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value()),
                            }
                        }

                        div { class: "toolbar-actions",
                            button {
                                class: "btn-refresh",
                                onclick: move |_| reload_trigger.set(reload_trigger() + 1),
                                IconRefresh { size: 16, color: "#475569".to_string() }
                            }
                            if can_write {
                                button {
                                    class: "btn-primary",
                                    onclick: move |_| {
                                        emit_patient_id.set(String::new());
                                        emit_template_id.set(String::new());
                                        emit_doc_title.set(String::new());
                                        emit_doc_type.set("contract".to_string());
                                        emit_pdf_url.set(String::new());
                                        is_emit_modal_open.set(true);
                                    },
                                    IconSignature { size: 16, color: "#ffffff".to_string() }
                                    " Emitir Novo Documento"
                                }
                            }
                        }
                    }

                    if is_loading {
                        div { class: "loading-card",
                            div { class: "loading-spinner" }
                            p { "Carregando documentos emitidos..." }
                        }
                    } else if documents_list.is_empty() {
                        div { class: "empty-state-card",
                            div { class: "empty-state-icon-box",
                                IconFile { size: 32, color: "currentColor".to_string() }
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
                                    for doc in documents_list.iter() {
                                        tr {
                                            td {
                                                div { class: "doc-title-cell",
                                                    IconFile { size: 18, color: "#0052cc".to_string() }
                                                    span { class: "font-semibold", "{doc.title}" }
                                                }
                                            }
                                            td {
                                                span { class: "badge-doc-type", "{doc.document_type}" }
                                            }
                                            td { "{format_br_date(&doc.created_at)}" }
                                            td {
                                                if doc.patient_signed_at.is_some() {
                                                    span { class: "badge-status-completed",
                                                        IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                        " Assinado"
                                                    }
                                                } else {
                                                    span { class: "badge-status-pending", "Pendente" }
                                                }
                                            }
                                            td {
                                                if doc.doctor_signed_at.is_some() {
                                                    span { class: "badge-status-completed",
                                                        IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                        " Assinado"
                                                    }
                                                } else {
                                                    span { class: "badge-status-pending", "Pendente" }
                                                }
                                            }
                                            td {
                                                if doc.status == "signed" || doc.status == "completed" {
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
                                                            let d = doc.clone();
                                                            move |_| qr_modal_doc.set(Some(d.clone()))
                                                        },
                                                        IconQrCode { size: 16, color: "#0052cc".to_string() }
                                                    }
                                                    button {
                                                        class: "btn-action-icon",
                                                        title: "Visualizar Documento / PDF",
                                                        onclick: {
                                                            let url = if let Some(ref s) = doc.signed_pdf_url { s.clone() } else { doc.original_pdf_url.clone() };
                                                            let tit = doc.title.clone();
                                                            move |_| pdf_preview_target.set(Some((url.clone(), tit.clone())))
                                                        },
                                                        IconEye { size: 16, color: "#475569".to_string() }
                                                    }
                                                    if can_delete {
                                                        button {
                                                            class: "btn-action-icon text-danger",
                                                            title: "Excluir",
                                                            onclick: {
                                                                let did = doc.id.clone();
                                                                let t = token.clone();
                                                                let cid = clinic_id.clone();
                                                                let mut reload_doc = reload_trigger;
                                                                move |_| {
                                                                    let t_call = t.clone();
                                                                    let cid_call = cid.clone();
                                                                    let did_call = did.clone();
                                                                    let mut reload_doc = reload_trigger;
                                                                    spawn(async move {
                                                                        if delete_patient_document(&t_call, &did_call, &cid_call).await.is_ok() {
                                                                            toast_msg.set(Some("Documento excluído.".into()));
                                                                            reload_doc.set(reload_doc() + 1);
                                                                        }
                                                                    });
                                                                }
                                                            },
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

            // TAB 2: MODELOS DE CONTRATOS
            if active_main_tab() == "templates" {
                div { class: "templates-view",
                    div { class: "tab-header-row",
                        div {
                            h3 { class: "tab-pane-title", "Modelos de Contratos e Posicionamento de Assinaturas" }
                            p { class: "tab-pane-subtitle", "Cadastre modelos de PDF e defina onde o paciente e o doutor devem assinar digitalmente." }
                        }
                        if can_write {
                            button {
                                class: "btn-primary",
                                onclick: open_create_template_modal,
                                IconSignature { size: 16, color: "#ffffff".to_string() }
                                " Novo Modelo de Contrato"
                            }
                        }
                    }

                    if templates_list.is_empty() {
                        div { class: "empty-state-card",
                            div { class: "empty-state-icon-box",
                                IconSignature { size: 32, color: "currentColor".to_string() }
                            }
                            h3 { "Nenhum modelo de contrato cadastrado" }
                            p { "Crie modelos como Termo TCLE, Contrato de Ortodontia ou Implante com tags de assinatura." }
                        }
                    } else {
                        div { class: "templates-grid",
                            for tpl in templates_list.iter() {
                                div { class: "template-card",
                                    div { class: "template-card-header",
                                        div {
                                            span { class: "template-cat-badge", "{tpl.category.to_uppercase()}" }
                                            h4 { class: "template-card-title", "{tpl.title}" }
                                        }
                                        div { class: "template-tags-count",
                                            IconSignature { size: 14, color: "#0052cc".to_string() }
                                            span { "{tpl.signature_fields.len()} campos de assinatura" }
                                        }
                                    }

                                    if let Some(ref d) = tpl.description {
                                        p { class: "template-card-desc", "{d}" }
                                    }

                                    div { class: "template-fields-chips",
                                        for f in tpl.signature_fields.iter() {
                                            span { class: if f.signer_type == "patient" { "field-chip patient" } else { "field-chip doctor" },
                                                "{f.label} (Pág. {f.page_number})"
                                            }
                                        }
                                    }

                                    div { class: "template-card-footer",
                                        button {
                                            class: "btn-preview-tpl",
                                            onclick: {
                                                let url = tpl.pdf_url.clone();
                                                let tit = tpl.title.clone();
                                                move |_| pdf_preview_target.set(Some((url.clone(), tit.clone())))
                                            },
                                            IconEye { size: 14, color: "#0052cc".to_string() }
                                            " Visualizar PDF Base"
                                        }
                                        if can_delete {
                                            button {
                                                class: "btn-action-icon text-danger",
                                                title: "Excluir Modelo",
                                                onclick: {
                                                    let tid = tpl.id.clone();
                                                    let t = token.clone();
                                                    let cid = clinic_id.clone();
                                                    let mut reload_doc = reload_trigger;
                                                    move |_| {
                                                        let t_call = t.clone();
                                                        let cid_call = cid.clone();
                                                        let tid_call = tid.clone();
                                                        let mut reload_doc = reload_trigger;
                                                        spawn(async move {
                                                            if delete_template(&t_call, &tid_call, &cid_call).await.is_ok() {
                                                                toast_msg.set(Some("Modelo excluído.".into()));
                                                                reload_doc.set(reload_doc() + 1);
                                                            }
                                                        });
                                                    }
                                                },
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

            // =========================================================================
            // MODAL: EMITIR NOVO DOCUMENTO
            // =========================================================================
            if is_emit_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal doc-emit-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Emitir Novo Documento / Contrato" }
                                p { class: "modal-subtitle", "Selecione o paciente e o modelo para gerar o link e QR Code de assinatura." }
                            }
                            button { class: "modal-close", onclick: move |_| is_emit_modal_open.set(false), "×" }
                        }

                        div { class: "modal-body",
                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Paciente *" }
                                    select {
                                        class: "select-field",
                                        value: "{emit_patient_id}",
                                        onchange: move |e| {
                                            let val = e.value();
                                            emit_patient_id.set(val.clone());
                                            if let Some(p) = patients_list.iter().find(|p| p.id == val).cloned() {
                                                emit_doc_title.set(format!("Contrato de Prestação de Serviços - {}", p.full_name));
                                                selected_patient_obj.set(Some(p));
                                            } else {
                                                selected_patient_obj.set(None);
                                            }
                                        },
                                        option { value: "", "Selecione o paciente..." }
                                        for p in patients_list.iter() {
                                            option { value: "{p.id}", "{p.full_name} ({p.document_cpf})" }
                                        }
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Modelo de Contrato Base" }
                                    select {
                                        class: "select-field",
                                        value: "{emit_template_id}",
                                        onchange: move |e| {
                                            let val = e.value();
                                            emit_template_id.set(val.clone());
                                            if let Some(t) = templates_list.iter().find(|t| t.id == val) {
                                                if emit_doc_title().is_empty() || emit_doc_title().starts_with("Contrato") {
                                                    if let Some(ref p) = selected_patient_obj() {
                                                        emit_doc_title.set(format!("{} - {}", t.title, p.full_name));
                                                    } else {
                                                        emit_doc_title.set(t.title.clone());
                                                    }
                                                }
                                            }
                                        },
                                        option { value: "", "Documento em Branco / Padrão" }
                                        for tpl in templates_list.iter() {
                                            option { value: "{tpl.id}", "{tpl.title}" }
                                        }
                                    }
                                }
                            }

                            // Dynamic Patient Auto-Fill Details Box
                            if let Some(ref p) = selected_patient_obj() {
                                div { class: "patient-autofill-card",
                                    div { class: "patient-autofill-item",
                                        span { class: "patient-autofill-label", "Paciente Selecionado" }
                                        span { class: "patient-autofill-val", "{p.full_name}" }
                                    }
                                    div { class: "patient-autofill-item",
                                        span { class: "patient-autofill-label", "CPF Criptografado / Protegido" }
                                        span { class: "patient-autofill-val", "{p.document_cpf}" }
                                    }
                                    div { class: "patient-autofill-item",
                                        span { class: "patient-autofill-label", "Telefone / WhatsApp" }
                                        span { class: "patient-autofill-val", "{p.phone}" }
                                    }
                                    div { class: "patient-autofill-item",
                                        span { class: "patient-autofill-label", "Convênio" }
                                        span { class: "patient-autofill-val", "{p.insurance_plan.as_deref().unwrap_or(\"Particular\")}" }
                                    }
                                }
                            }

                            div { class: "form-group",
                                label { class: "form-label", "Título do Documento *" }
                                input {
                                    r#type: "text",
                                    class: "input-field",
                                    placeholder: "Ex: Termo de Consentimento - Implante Unitário",
                                    value: "{emit_doc_title}",
                                    oninput: move |e| emit_doc_title.set(e.value()),
                                }
                            }

                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Tipo de Documento" }
                                    select {
                                        class: "select-field",
                                        value: "{emit_doc_type}",
                                        onchange: move |e| emit_doc_type.set(e.value()),
                                        option { value: "contract", "Contrato de Prestação de Serviços (E-Sign)" }
                                        option { value: "consent", "Termo de Consentimento Livre e Esclarecido (TCLE)" }
                                        option { value: "budget", "Orçamento Aprovado" }
                                        option { value: "static_upload", "Upload de Documento Já Assinado" }
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Arquivo PDF (Opcional se usar modelo)" }
                                    label { class: "attachment-dropzone mini-dropzone",
                                        input {
                                            r#type: "file",
                                            accept: ".pdf,application/pdf",
                                            onchange: {
                                                let t = token.clone();
                                                move |evt: FormEvent| {
                                                    for file in evt.files() {
                                                        let fname = file.name();
                                                        uploaded_doc_pdf_name.set(fname.clone());
                                                        is_uploading_doc_pdf.set(true);
                                                        let t_c = t.clone();
                                                        spawn(async move {
                                                            if let Ok(bytes) = file.read_bytes().await {
                                                                let b64 = general_purpose::STANDARD.encode(&bytes);
                                                                if let Ok(url) = crate::api::upload_document_pdf(&t_c, &fname, &b64).await {
                                                                    emit_pdf_url.set(url);
                                                                }
                                                            }
                                                            is_uploading_doc_pdf.set(false);
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "dropzone-label",
                                            IconUpload { size: 16, color: "#0052cc".to_string() }
                                            span {
                                                if is_uploading_doc_pdf() {
                                                    "Enviando PDF..."
                                                } else if !emit_pdf_url().is_empty() {
                                                    "✓ Documento PDF Carregado"
                                                } else {
                                                    "Fazer Upload do PDF"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_emit_modal_open.set(false), "Cancelar" }
                            button { class: "btn-primary", onclick: on_submit_emit_doc, "Emitir e Gerar QR Code" }
                        }
                    }
                }
            }

            // =========================================================================
            // MODAL: CRIAR / EDITAR MODELO DE CONTRATO (DESIGNER VISUAL COM CANVAS A4)
            // =========================================================================
            if is_template_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal template-modal-wide",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Editor Visual de Modelo & Posição de Assinaturas" }
                                p { class: "modal-subtitle", "Visualize a página do documento, use as âncoras rápidas ou ajuste as coordenadas exatas." }
                            }
                            button { class: "modal-close", onclick: move |_| is_template_modal_open.set(false), "×" }
                        }

                        div { class: "modal-body",
                            div { class: "template-editor-grid",
                                // -------------------------------------------------------------
                                // COLUNA ESQUERDA: CANVAS VISUAL DO DOCUMENTO (PREVIEW A4)
                                // -------------------------------------------------------------
                                div { class: "pdf-canvas-column",
                                    div { class: "pdf-canvas-title",
                                        span { "Pré-visualização do Documento (A4)" }
                                        span { class: "pdf-canvas-hint", "Página {new_tag_page()}" }
                                    }

                                    div { class: "pdf-page-preview-wrapper",
                                        div {
                                            class: "pdf-page-canvas",
                                            // Header
                                            div { class: "pdf-canvas-letterhead",
                                                div { class: "pdf-canvas-logo",
                                                    IconTooth { size: 14, color: "#0052cc".to_string() }
                                                    span { "Tooth Plus Dental Clinic" }
                                                }
                                                span { style: "font-size: 9px; color: #64748b;", "CNPJ: 00.000.000/0001-00" }
                                            }

                                            // Title
                                            div { class: "pdf-canvas-doc-title",
                                                if tpl_title().is_empty() {
                                                    "CONTRATO DE PRESTAÇÃO DE SERVIÇOS ODONTOLÓGICOS"
                                                } else {
                                                    "{tpl_title().to_uppercase()}"
                                                }
                                            }

                                            // Body Text Simulation
                                            div { class: "pdf-simulated-text",
                                                p {
                                                    "Pelo presente instrumento particular, a clínica qualificada no cabeçalho e o paciente "
                                                    span { class: "pdf-tag-highlight", "{{paciente_nome}}" }
                                                    ", portador do CPF nº "
                                                    span { class: "pdf-tag-highlight", "{{paciente_cpf}}" }
                                                    ", residente no endereço "
                                                    span { class: "pdf-tag-highlight", "{{paciente_endereco}}" }
                                                    ", celebram o presente acordo de tratamento odontológico especializado."
                                                }
                                                p {
                                                    "Cláusula 1ª - O profissional responsável "
                                                    span { class: "pdf-tag-highlight", "{{doutor_nome}}" }
                                                    " compromete-se a executar os procedimentos acordados com estrita observância das normas do CFO e biossegurança."
                                                }
                                                p {
                                                    "Cláusula 2ª - As partes concordam com os valores pactuados e assinam eletronicamente o presente instrumento em "
                                                    span { class: "pdf-tag-highlight", "{{data_hoje}}" }
                                                    "."
                                                }
                                            }

                                            // Configured Signature Markers Overlay
                                            for (idx, tag) in tpl_signature_fields().iter().enumerate() {
                                                if tag.page_number == new_tag_page() {
                                                    div {
                                                        class: if tag.signer_type == "patient" { "pdf-signature-marker patient" } else { "pdf-signature-marker doctor" },
                                                        style: "left: {tag.x_pct}%; top: {tag.y_pct}%; width: {tag.width_pct}%;",
                                                        if tag.signer_type == "patient" {
                                                            IconSignature { size: 12, color: "#0052cc".to_string() }
                                                        } else {
                                                            IconShieldCheck { size: 12, color: "#059669".to_string() }
                                                        }
                                                        span { style: "white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", "{tag.label}" }
                                                        button {
                                                            class: "marker-close-btn",
                                                            onclick: {
                                                                let mut list = tpl_signature_fields();
                                                                move |e: MouseEvent| {
                                                                    e.stop_propagation();
                                                                    if idx < list.len() {
                                                                        list.remove(idx);
                                                                        tpl_signature_fields.set(list.clone());
                                                                    }
                                                                }
                                                            },
                                                            "×"
                                                        }
                                                    }
                                                }
                                            }

                                            // Active Indicator for Tag being positioned
                                            div {
                                                class: if new_tag_signer() == "patient" { "pdf-signature-marker patient" } else { "pdf-signature-marker doctor" },
                                                style: "left: {new_tag_x()}%; top: {new_tag_y()}%; width: 32%; opacity: 0.85; border-style: solid; box-shadow: 0 0 0 3px rgba(0, 82, 204, 0.3);",
                                                if new_tag_signer() == "patient" {
                                                    IconSignature { size: 12, color: "#0052cc".to_string() }
                                                } else {
                                                    IconShieldCheck { size: 12, color: "#059669".to_string() }
                                                }
                                                span { style: "white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", "{new_tag_label()}" }
                                                span { style: "font-size: 8.5px; opacity: 0.8;", "(Novo)" }
                                            }

                                            div { class: "pdf-canvas-page-num", "Página {new_tag_page()} de 1" }
                                        }
                                    }
                                }

                                // -------------------------------------------------------------
                                // COLUNA DIREITA: CONTROLES DE POSICIONAMENTO E VARIÁVEIS
                                // -------------------------------------------------------------
                                div { style: "display: flex; flex-direction: column; gap: 14px;",
                                    div { class: "form-group",
                                        label { class: "form-label", "Título do Modelo *" }
                                        input {
                                            r#type: "text",
                                            class: "input-field",
                                            placeholder: "Ex: Contrato de Prestação de Serviços",
                                            value: "{tpl_title}",
                                            oninput: move |e| tpl_title.set(e.value()),
                                        }
                                    }

                                    div { class: "form-row-2",
                                        div { class: "form-group",
                                            label { class: "form-label", "Categoria" }
                                            select {
                                                class: "select-field",
                                                value: "{tpl_category}",
                                                onchange: move |e| tpl_category.set(e.value()),
                                                option { value: "contract", "Contrato Geral" }
                                                option { value: "consent", "Termo de Consentimento (TCLE)" }
                                                option { value: "orthodontics", "Ortodontia" }
                                                option { value: "implant", "Implantodontia / Cirurgia" }
                                                option { value: "aesthetic", "Harmonização & Estética" }
                                            }
                                        }
                                        div { class: "form-group",
                                            label { class: "form-label", "Arquivo PDF Base do Modelo" }
                                            label { class: "attachment-dropzone mini-dropzone",
                                                input {
                                                    r#type: "file",
                                                    accept: ".pdf,application/pdf",
                                                    onchange: {
                                                        let t = token.clone();
                                                        move |evt: FormEvent| {
                                                            for file in evt.files() {
                                                                let fname = file.name();
                                                                uploaded_tpl_pdf_name.set(fname.clone());
                                                                is_uploading_tpl_pdf.set(true);
                                                                let t_c = t.clone();
                                                                spawn(async move {
                                                                    if let Ok(bytes) = file.read_bytes().await {
                                                                        let b64 = general_purpose::STANDARD.encode(&bytes);
                                                                        if let Ok(url) = crate::api::upload_document_pdf(&t_c, &fname, &b64).await {
                                                                            tpl_pdf_url.set(url);
                                                                        }
                                                                    }
                                                                    is_uploading_tpl_pdf.set(false);
                                                                });
                                                            }
                                                        }
                                                    }
                                                }
                                                div { class: "dropzone-label",
                                                    IconUpload { size: 16, color: "#0052cc".to_string() }
                                                    span {
                                                        if is_uploading_tpl_pdf() {
                                                            "Enviando PDF do Modelo..."
                                                        } else if !tpl_pdf_url().is_empty() {
                                                            "✓ PDF do Modelo Carregado"
                                                        } else {
                                                            "Fazer Upload do PDF do Modelo"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Dynamic Variables helper pills
                                    div { class: "form-group",
                                        label { class: "form-label", "Variáveis Dinâmicas do Paciente / Clínica" }
                                        div { class: "variables-pills-row",
                                            span { class: "var-pill", "{{paciente_nome}}" }
                                            span { class: "var-pill", "{{paciente_cpf}}" }
                                            span { class: "var-pill", "{{paciente_telefone}}" }
                                            span { class: "var-pill", "{{paciente_endereco}}" }
                                            span { class: "var-pill", "{{clinica_nome}}" }
                                            span { class: "var-pill", "{{doutor_nome}}" }
                                            span { class: "var-pill", "{{data_hoje}}" }
                                        }
                                    }

                                    // Signature Tags Designer Section
                                    div { class: "signature-tags-section",
                                        h3 { class: "section-subtitle", "Âncoras & Posicionamento de Assinatura" }
                                        p { class: "section-desc", "Escolha uma posição rápida ou ajuste os controles abaixo:" }

                                        // Preset Anchor Buttons
                                        div { class: "position-presets-grid",
                                            button {
                                                r#type: "button",
                                                class: "btn-preset-anchor",
                                                onclick: move |_| {
                                                    new_tag_signer.set("patient".into());
                                                    new_tag_x.set(10.0);
                                                    new_tag_y.set(82.0);
                                                    new_tag_label.set("Assinatura do Paciente".into());
                                                },
                                                IconSignature { size: 14, color: "#0052cc".to_string() }
                                                " Rodapé Esq. (Paciente)"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-preset-anchor",
                                                onclick: move |_| {
                                                    new_tag_signer.set("doctor".into());
                                                    new_tag_x.set(55.0);
                                                    new_tag_y.set(82.0);
                                                    new_tag_label.set("Dr(a). Responsável Técnico".into());
                                                },
                                                IconShieldCheck { size: 14, color: "#059669".to_string() }
                                                " Rodapé Dir. (Doutor)"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-preset-anchor",
                                                onclick: move |_| {
                                                    let mut list = tpl_signature_fields();
                                                    list.push(SignatureField {
                                                        id: format!("tag_patient_{}", list.len() + 1),
                                                        signer_type: "patient".into(),
                                                        page_number: new_tag_page(),
                                                        x_pct: 10.0,
                                                        y_pct: 82.0,
                                                        width_pct: 32.0,
                                                        height_pct: 10.0,
                                                        label: "Assinatura do Paciente".into(),
                                                        is_required: true,
                                                    });
                                                    list.push(SignatureField {
                                                        id: format!("tag_doctor_{}", list.len() + 2),
                                                        signer_type: "doctor".into(),
                                                        page_number: new_tag_page(),
                                                        x_pct: 55.0,
                                                        y_pct: 82.0,
                                                        width_pct: 32.0,
                                                        height_pct: 10.0,
                                                        label: "Dr(a). Responsável Técnico".into(),
                                                        is_required: true,
                                                    });
                                                    tpl_signature_fields.set(list);
                                                },
                                                IconSignature { size: 14, color: "#f59e0b".to_string() }
                                                " Dupla Assinatura (Ambos)"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "btn-preset-anchor",
                                                onclick: move |_| {
                                                    new_tag_x.set(30.0);
                                                    new_tag_y.set(86.0);
                                                    new_tag_label.set("Assinatura Central".into());
                                                },
                                                IconSignature { size: 14, color: "#64748b".to_string() }
                                                " Centralizado no Final"
                                            }
                                        }

                                        // Coordinate Sliders Fine Tuning
                                        div { class: "coord-sliders-row",
                                            div { class: "coord-slider-group",
                                                div { class: "coord-slider-label",
                                                    span { "Posição Horizontal X" }
                                                    span { "{new_tag_x():.0}%" }
                                                }
                                                input {
                                                    r#type: "range",
                                                    min: "5",
                                                    max: "70",
                                                    step: "1",
                                                    class: "coord-slider-input",
                                                    value: "{new_tag_x()}",
                                                    oninput: move |e| {
                                                        if let Ok(v) = e.value().parse::<f32>() {
                                                            new_tag_x.set(v);
                                                        }
                                                    }
                                                }
                                            }
                                            div { class: "coord-slider-group",
                                                div { class: "coord-slider-label",
                                                    span { "Posição Vertical Y" }
                                                    span { "{new_tag_y():.0}%" }
                                                }
                                                input {
                                                    r#type: "range",
                                                    min: "5",
                                                    max: "90",
                                                    step: "1",
                                                    class: "coord-slider-input",
                                                    value: "{new_tag_y()}",
                                                    oninput: move |e| {
                                                        if let Ok(v) = e.value().parse::<f32>() {
                                                            new_tag_y.set(v);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Tag Settings Row
                                        div { class: "form-row-2",
                                            div { class: "form-group",
                                                label { class: "form-label", "Quem Assina?" }
                                                select {
                                                    class: "select-field",
                                                    value: "{new_tag_signer}",
                                                    onchange: move |e| {
                                                        let val = e.value();
                                                        new_tag_signer.set(val.clone());
                                                        if val == "patient" {
                                                            new_tag_label.set("Assinatura do Paciente".into());
                                                        } else {
                                                            new_tag_label.set("Dr(a). Responsável Técnico".into());
                                                        }
                                                    },
                                                    option { value: "patient", "Paciente" }
                                                    option { value: "doctor", "Doutor / Responsável Técnico" }
                                                }
                                            }
                                            div { class: "form-group",
                                                label { class: "form-label", "Rótulo da Assinatura" }
                                                input {
                                                    r#type: "text",
                                                    class: "input-field",
                                                    value: "{new_tag_label}",
                                                    oninput: move |e| new_tag_label.set(e.value()),
                                                }
                                            }
                                        }

                                        button {
                                            r#type: "button",
                                            class: "btn-secondary",
                                            style: "height: 42px; width: 100%; justify-content: center; font-weight: 700;",
                                            onclick: move |_| {
                                                let mut list = tpl_signature_fields();
                                                let s_type = new_tag_signer();
                                                let new_field = SignatureField {
                                                    id: format!("tag_{}_{}", s_type, list.len() + 1),
                                                    signer_type: s_type,
                                                    page_number: new_tag_page(),
                                                    x_pct: new_tag_x(),
                                                    y_pct: new_tag_y(),
                                                    width_pct: 32.0,
                                                    height_pct: 10.0,
                                                    label: new_tag_label(),
                                                    is_required: true,
                                                };
                                                list.push(new_field);
                                                tpl_signature_fields.set(list);
                                            },
                                            IconSignature { size: 16, color: "#0052cc".to_string() }
                                            " + Inserir Marcação na Página"
                                        }

                                        // Configured Tags List
                                        div { class: "tags-configured-list",
                                            for (idx, tag) in tpl_signature_fields().iter().enumerate() {
                                                div { class: if tag.signer_type == "patient" { "tag-row patient" } else { "tag-row doctor" },
                                                    div { class: "tag-info",
                                                        span { class: "tag-role-pill",
                                                            if tag.signer_type == "patient" { "Paciente" } else { "Doutor" }
                                                        }
                                                        span { class: "tag-label-text", "{tag.label}" }
                                                        span { class: "tag-meta-sub", "Pág. {tag.page_number} (X={tag.x_pct:.0}%, Y={tag.y_pct:.0}%)" }
                                                    }
                                                    button {
                                                        class: "btn-remove-tag",
                                                        onclick: {
                                                            let mut list = tpl_signature_fields();
                                                            move |_| {
                                                                if idx < list.len() {
                                                                    list.remove(idx);
                                                                    tpl_signature_fields.set(list.clone());
                                                                }
                                                            }
                                                        },
                                                        "Remover"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_template_modal_open.set(false), "Cancelar" }
                            button { class: "btn-primary", onclick: on_submit_template, "Salvar Modelo de Contrato" }
                        }
                    }
                }
            }

            // =========================================================================
            // MODAL: QR CODE DE ASSINATURA (SVG OFFLINE)
            // =========================================================================
            if let Some(ref doc) = qr_modal_doc() {
                {
                    let link_url = format!("http://localhost:8080/sign/{}", doc.signing_token);
                    let qr_svg = generate_qr_svg(&link_url);
                    rsx! {
                        div { class: "modal-overlay",
                            div { class: "action-modal qr-modal-card",
                                div { class: "modal-header",
                                    div {
                                        h2 { class: "modal-title", "Assinatura Digital via QR Code" }
                                        p { class: "modal-subtitle", "Aponte a câmera do celular ou acesse o link para assinar digitalmente." }
                                    }
                                    button { class: "modal-close", onclick: move |_| qr_modal_doc.set(None), "×" }
                                }

                                div { class: "modal-body text-center",
                                    div { class: "qr-box-center",
                                        div {
                                            class: "qr-svg-wrapper",
                                            dangerous_inner_html: "{qr_svg}"
                                        }
                                    }

                                    p { class: "qr-doc-title", "{doc.title}" }
                                    p { class: "qr-hint", "O paciente poderá visualizar o contrato na íntegra, autenticar-se e desenhar a assinatura na tela do celular ou tablet." }

                                    div { class: "qr-link-copy-box",
                                        input {
                                            r#type: "text",
                                            readonly: true,
                                            class: "input-field",
                                            value: "{link_url}",
                                        }
                                        a {
                                            href: "{link_url}",
                                            target: "_blank",
                                            class: "btn-secondary",
                                            IconExternalLink { size: 16, color: "#0052cc".to_string() }
                                            " Abrir Portal"
                                        }
                                    }
                                }

                                div { class: "modal-footer",
                                    button { class: "btn-primary full-width", onclick: move |_| qr_modal_doc.set(None), "Concluir" }
                                }
                            }
                        }
                    }
                }
            }

            // =========================================================================
            // MODAL: VISUALIZADOR DE PDF / DOCUMENTO (WEB NATIVO)
            // =========================================================================
            if let Some((ref url, ref title)) = pdf_preview_target() {
                div { class: "modal-overlay",
                    onclick: move |_| pdf_preview_target.set(None),
                    div { class: "action-modal pdf-viewer-modal",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "{title}" }
                                p { class: "modal-subtitle", "Documento PDF Clínico" }
                            }
                            div { class: "modal-header-actions",
                                a {
                                    href: "{url}",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "btn-secondary btn-sm",
                                    IconExternalLink { size: 14, color: "#0052cc".to_string() }
                                    " Abrir em Nova Aba"
                                }
                                button {
                                    class: "modal-close",
                                    onclick: move |_| pdf_preview_target.set(None),
                                    "×"
                                }
                            }
                        }
                        div { class: "modal-body pdf-viewer-modal-body",
                            object {
                                data: "{url}#toolbar=1&view=FitH",
                                r#type: "application/pdf",
                                class: "pdf-modal-embed",
                                iframe {
                                    src: "{url}",
                                    class: "pdf-modal-embed",
                                    title: "{title}",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
