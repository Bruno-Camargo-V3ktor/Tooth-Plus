use crate::api::{
    create_patient_document, create_template, delete_patient_document, delete_template,
    fetch_documents, fetch_patients, update_template,
};
use crate::components::icons::{
    IconCheckCircle, IconExternalLink, IconEye, IconFile, IconQrCode, IconRefresh, IconSearch,
    IconShieldCheck, IconSignature, IconTrash,
};
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use qrcode::QrCode;
use qrcode::render::svg;
use shared::documents::{
    ContractTemplate, CreateContractTemplateRequest, CreatePatientDocumentRequest, DocumentsKpis,
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
    let clinic_id = clinic.as_ref().map(|c| c.clinic_id.clone()).unwrap_or_default();


    let mut active_main_tab = use_signal(|| "emitted".to_string()); // "emitted" | "templates"
    let mut documents_list = use_signal(Vec::<PatientDocument>::new);
    let mut templates_list = use_signal(Vec::<ContractTemplate>::new);
    let mut patients_list = use_signal(Vec::<Patient>::new);
    let mut kpis = use_signal(DocumentsKpis::default);

    let mut is_loading = use_signal(|| true);
    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "all".to_string());

    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    // Modals
    let mut is_emit_modal_open = use_signal(|| false);
    let mut is_template_modal_open = use_signal(|| false);
    let mut editing_template_id = use_signal(|| None::<String>);
    let mut qr_modal_doc = use_signal(|| None::<PatientDocument>);
    let mut pdf_preview_target = use_signal(|| None::<(String, String)>);

    // Form inputs: Emit Document
    let mut emit_patient_id = use_signal(String::new);
    let mut emit_template_id = use_signal(String::new);
    let mut emit_doc_title = use_signal(String::new);
    let mut emit_doc_type = use_signal(|| "contract".to_string());
    let mut emit_pdf_url = use_signal(String::new);

    // Form inputs: Template Editor
    let mut tpl_title = use_signal(String::new);
    let mut tpl_category = use_signal(|| "contract".to_string());
    let mut tpl_desc = use_signal(String::new);
    let mut tpl_pdf_url = use_signal(|| "https://placehold.co/800x1100/ffffff/0f172a?text=Modelo+de+Contrato".to_string());
    let mut tpl_signature_fields = use_signal(Vec::<SignatureField>::new);

    // New Signature Tag Form
    let mut new_tag_signer = use_signal(|| "patient".to_string());
    let mut new_tag_page = use_signal(|| 1u32);
    let mut new_tag_x = use_signal(|| 15.0f32);
    let mut new_tag_y = use_signal(|| 80.0f32);
    let mut new_tag_label = use_signal(|| "Assinatura do Paciente".to_string());

    let load_documents_data = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        move || {
            let t = token.clone();
            let cid = clinic_id.clone();
            let st = status_filter();
            let st_opt = if st == "all" { None } else { Some(st) };

            spawn(async move {
                is_loading.set(true);
                let st_ref = st_opt.as_deref();
                match fetch_documents(&t, &cid, None, st_ref).await {
                    Ok(resp) => {
                        documents_list.set(resp.documents);
                        templates_list.set(resp.templates);
                        kpis.set(resp.kpis);
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
                is_loading.set(false);
            });
        }
    };

    let load_patients_dropdown = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        move || {
            let t = token.clone();
            let cid = clinic_id.clone();
            spawn(async move {
                if let Ok(resp) = fetch_patients(&t, &cid, None).await {
                    patients_list.set(resp.items);
                }
            });
        }
    };

    use_effect({
        let ld = load_documents_data.clone();
        let lp = load_patients_dropdown.clone();
        move || {
            ld();
            lp();
        }
    });

    let open_create_template_modal = move |_| {
        editing_template_id.set(None);
        tpl_title.set(String::new());
        tpl_category.set("contract".to_string());
        tpl_desc.set(String::new());
        tpl_pdf_url.set("https://placehold.co/800x1100/ffffff/0f172a?text=Modelo+de+Contrato".to_string());
        
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
        let ld = load_documents_data.clone();
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let ld_call = ld.clone();
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
                        description: if tpl_desc().is_empty() { None } else { Some(tpl_desc()) },
                        pdf_url: tpl_pdf_url(),
                        signature_fields: tpl_signature_fields(),
                    };
                    if let Ok(_) = update_template(&t, &id, req).await {
                        toast_msg.set(Some("Modelo de contrato atualizado!".into()));
                        is_template_modal_open.set(false);
                        ld_call();
                    }
                } else {
                    let req = CreateContractTemplateRequest {
                        clinic_id: cid,
                        title: tpl_title(),
                        category: tpl_category(),
                        description: if tpl_desc().is_empty() { None } else { Some(tpl_desc()) },
                        pdf_url: tpl_pdf_url(),
                        signature_fields: tpl_signature_fields(),
                    };
                    if let Ok(_) = create_template(&t, req).await {
                        toast_msg.set(Some("Modelo de contrato criado com sucesso!".into()));
                        is_template_modal_open.set(false);
                        ld_call();
                    }
                }
            });
        }
    };

    let on_submit_emit_doc = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        let ld = load_documents_data.clone();
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let ld_call = ld.clone();

            if emit_patient_id().trim().is_empty() {
                error_toast.set(Some("Selecione o paciente para emissão.".into()));
                return;
            }

            let tpl_id = if emit_template_id().is_empty() { None } else { Some(emit_template_id()) };
            let pdf = if emit_pdf_url().is_empty() { None } else { Some(emit_pdf_url()) };

            let req = CreatePatientDocumentRequest {
                clinic_id: cid,
                patient_id: emit_patient_id(),
                template_id: tpl_id,
                doctor_user_id: None,
                appointment_id: None,
                title: if emit_doc_title().is_empty() { "Termo de Consentimento Odontológico".to_string() } else { emit_doc_title() },
                document_type: emit_doc_type(),
                pdf_url: pdf,
            };

            spawn(async move {
                match create_patient_document(&t, req).await {
                    Ok(doc) => {
                        toast_msg.set(Some("Documento emitido com sucesso!".into()));
                        is_emit_modal_open.set(false);
                        qr_modal_doc.set(Some(doc));
                        ld_call();
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
            });
        }
    };

    let ld_refresh = load_documents_data.clone();

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
                        h3 { class: "kpi-value", "{kpis().total_documents}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-amber-light",
                        IconSignature { size: 24, color: "#f59e0b".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Pendentes de Assinatura" }
                        h3 { class: "kpi-value", "{kpis().pending_signatures}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-emerald-light",
                        IconCheckCircle { size: 24, color: "#10b981".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "100% Assinados e Validados" }
                        h3 { class: "kpi-value", "{kpis().completed_signed}" }
                    }
                }
                div { class: "kpi-card",
                    div { class: "kpi-icon-wrap bg-purple-light",
                        IconShieldCheck { size: 24, color: "#8b5cf6".to_string() }
                    }
                    div { class: "kpi-content",
                        span { class: "kpi-label", "Modelos de Contrato" }
                        h3 { class: "kpi-value", "{kpis().templates_count}" }
                    }
                }
            }

            // Main Tabs Switcher
            div { class: "documents-tab-bar",
                button {
                    class: if active_main_tab() == "emitted" { "doc-main-tab active" } else { "doc-main-tab" },
                    onclick: move |_| active_main_tab.set("emitted".to_string()),
                    IconFile { size: 16, color: "currentColor".to_string() }
                    " Documentos & Contratos Emitidos ({documents_list().len()})"
                }
                button {
                    class: if active_main_tab() == "templates" { "doc-main-tab active" } else { "doc-main-tab" },
                    onclick: move |_| active_main_tab.set("templates".to_string()),
                    IconSignature { size: 16, color: "currentColor".to_string() }
                    " Modelos de Contratos & E-Sign ({templates_list().len()})"
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
                                onclick: move |_| ld_refresh(),
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

                    if is_loading() {
                        div { class: "loading-card",
                            div { class: "loading-spinner" }
                            p { "Carregando documentos emitidos..." }
                        }
                    } else if documents_list().is_empty() {
                        div { class: "empty-state-card",
                            IconFile { size: 48, color: "#94a3b8".to_string() }
                            h3 { "Nenhum documento emitido" }
                            p { "Emita contratos e termos para assinatura digital imediata de pacientes e doutores." }
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
                                    for doc in documents_list().iter() {
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
                                                                let ld = load_documents_data.clone();
                                                                move |_| {
                                                                    let t_call = t.clone();
                                                                    let cid_call = cid.clone();
                                                                    let did_call = did.clone();
                                                                    let ld_call = ld.clone();
                                                                    spawn(async move {
                                                                        if delete_patient_document(&t_call, &did_call, &cid_call).await.is_ok() {
                                                                            toast_msg.set(Some("Documento excluído.".into()));
                                                                            ld_call();
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

                    if templates_list().is_empty() {
                        div { class: "empty-state-card",
                            IconSignature { size: 48, color: "#94a3b8".to_string() }
                            h3 { "Nenhum modelo de contrato cadastrado" }
                            p { "Crie modelos como Termo TCLE, Contrato de Ortodontia ou Implante com tags de assinatura." }
                        }
                    } else {
                        div { class: "templates-grid",
                            for tpl in templates_list().iter() {
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
                                                    let ld = load_documents_data.clone();
                                                    move |_| {
                                                        let t_call = t.clone();
                                                        let cid_call = cid.clone();
                                                        let tid_call = tid.clone();
                                                        let ld_call = ld.clone();
                                                        spawn(async move {
                                                            if delete_template(&t_call, &tid_call, &cid_call).await.is_ok() {
                                                                toast_msg.set(Some("Modelo excluído.".into()));
                                                                ld_call();
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
                                        onchange: move |e| emit_patient_id.set(e.value()),
                                        option { value: "", "Selecione o paciente..." }
                                        for p in patients_list().iter() {
                                            option { value: "{p.id}", "{p.full_name} ({p.document_cpf})" }
                                        }
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Modelo de Contrato Base" }
                                    select {
                                        class: "select-field",
                                        value: "{emit_template_id}",
                                        onchange: move |e| emit_template_id.set(e.value()),
                                        option { value: "", "Documento em Branco / Padrão" }
                                        for tpl in templates_list().iter() {
                                            option { value: "{tpl.id}", "{tpl.title}" }
                                        }
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
                                    label { class: "form-label", "URL do PDF (Opcional)" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "https://... (ou deixe vazio)",
                                        value: "{emit_pdf_url}",
                                        oninput: move |e| emit_pdf_url.set(e.value()),
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
            // MODAL: CRIAR / EDITAR MODELO DE CONTRATO
            // =========================================================================
            if is_template_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal template-modal-wide",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Editor de Modelo de Contrato & Tags de Assinatura" }
                                p { class: "modal-subtitle", "Defina o PDF base e posicione as assinaturas do Paciente e do Doutor." }
                            }
                            button { class: "modal-close", onclick: move |_| is_template_modal_open.set(false), "×" }
                        }

                        div { class: "modal-body",
                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Título do Modelo *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "Ex: Contrato de Prestação de Serviços Odontológicos",
                                        value: "{tpl_title}",
                                        oninput: move |e| tpl_title.set(e.value()),
                                    }
                                }
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
                            }

                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Descrição do Modelo" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "Breve explicação sobre a finalidade deste contrato...",
                                        value: "{tpl_desc}",
                                        oninput: move |e| tpl_desc.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "URL do PDF Base *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "https://... (URL do PDF template)",
                                        value: "{tpl_pdf_url}",
                                        oninput: move |e| tpl_pdf_url.set(e.value()),
                                    }
                                }
                            }

                            // Interactive Signature Markers Manager
                            div { class: "signature-tags-section",
                                h3 { class: "section-subtitle", "Marcações de Assinatura no Documento" }
                                p { class: "section-desc", "Adicione múltiplos pontos de assinatura para o Paciente e para o Doutor." }

                                div { class: "add-tag-box",
                                    div { class: "form-row-2",
                                        div { class: "form-group",
                                            label { class: "form-label", "Quem deve assinar?" }
                                            select {
                                                class: "select-field",
                                                value: "{new_tag_signer}",
                                                onchange: move |e| {
                                                    let val = e.value();
                                                    new_tag_signer.set(val.clone());
                                                    if val == "patient" {
                                                        new_tag_label.set("Assinatura do Paciente".into());
                                                    } else {
                                                        new_tag_label.set("Assinatura do Cirurgião-Dentista".into());
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
                                                placeholder: "Ex: Assinatura do Contratante",
                                                value: "{new_tag_label}",
                                                oninput: move |e| new_tag_label.set(e.value()),
                                            }
                                        }
                                    }

                                    div { class: "form-row-2", style: "margin-top: 10px;",
                                        div { class: "form-group",
                                            label { class: "form-label", "Página do PDF" }
                                            input {
                                                r#type: "number",
                                                class: "input-field",
                                                min: "1",
                                                value: "{new_tag_page}",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<u32>() {
                                                        new_tag_page.set(v);
                                                    }
                                                },
                                            }
                                        }
                                        div { class: "form-group", style: "display: flex; justify-content: flex-end; align-items: flex-end;",
                                            button {
                                                class: "btn-secondary",
                                                style: "height: 42px; width: 100%; justify-content: center;",
                                                onclick: move |_| {
                                                    let mut list = tpl_signature_fields();
                                                    let s_type = new_tag_signer();
                                                    let default_x = if s_type == "patient" { 15.0 } else { 55.0 };
                                                    let new_field = SignatureField {
                                                        id: format!("tag_{}_{}", s_type, list.len() + 1),
                                                        signer_type: s_type,
                                                        page_number: new_tag_page(),
                                                        x_pct: default_x,
                                                        y_pct: 82.0,
                                                        width_pct: 30.0,
                                                        height_pct: 10.0,
                                                        label: new_tag_label(),
                                                        is_required: true,
                                                    };
                                                    list.push(new_field);
                                                    tpl_signature_fields.set(list);
                                                },
                                                IconSignature { size: 14, color: "#0052cc".to_string() }
                                                " + Adicionar Ponto de Assinatura"
                                            }
                                        }
                                    }
                                }

                                // List of configured signature tags
                                div { class: "tags-configured-list",
                                    for (idx, tag) in tpl_signature_fields().iter().enumerate() {
                                        div { class: if tag.signer_type == "patient" { "tag-row patient" } else { "tag-row doctor" },
                                            div { class: "tag-info",
                                                span { class: "tag-role-pill",
                                                    if tag.signer_type == "patient" { "Paciente" } else { "Doutor" }
                                                }
                                                span { class: "tag-label-text", "{tag.label}" }
                                                span { class: "tag-meta-sub", "Pág. {tag.page_number} (Posição: X={tag.x_pct}%, Y={tag.y_pct}%)" }
                                            }
                                            button {
                                                class: "btn-remove-tag",
                                                onclick: move |_| {
                                                    let mut list = tpl_signature_fields();
                                                    if idx < list.len() {
                                                        list.remove(idx);
                                                        tpl_signature_fields.set(list);
                                                    }
                                                },
                                                "Remover"
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
            // MODAL: VISUALIZADOR DE PDF / DOCUMENTO
            // =========================================================================
            if let Some((ref url, ref title)) = pdf_preview_target() {
                div { class: "modal-overlay",
                    div { class: "action-modal pdf-viewer-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "{title}" }
                                p { class: "modal-subtitle", "Visualização do documento em PDF" }
                            }
                            div { style: "display: flex; align-items: center; gap: 10px;",
                                a {
                                    href: "{url}",
                                    target: "_blank",
                                    class: "btn-secondary",
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
                        div { class: "modal-body pdf-iframe-container",
                            iframe {
                                src: "{url}",
                                title: "{title}",
                                style: "width: 100%; height: 100%; min-height: 520px; border: none;",
                            }
                        }
                    }
                }
            }
        }
    }
}
