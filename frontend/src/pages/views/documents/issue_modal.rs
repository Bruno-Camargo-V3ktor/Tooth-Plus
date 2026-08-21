//! # Modal de Emissão de Contratos e Upload de Documentos Assinados (Frontend)
//!
//! Suporta dois fluxos clínicos integrados:
//! 1. **Nova Solicitação de Assinatura (E-Sign)**: Seleciona modelo, paciente e dentista responsável para geração de termo digital e QR Code.
//! 2. **Upload de Documento Já Assinado**: Permite arquivar termos e contratos físicos ou digitais previamente assinados pelo paciente e profissional.

use crate::api::{create_patient_document, upload_document_pdf};
use crate::components::icons::{
    IconCheckCircle, IconFile, IconSignature, IconUpload, IconUsers,
};
use dioxus::prelude::*;
use shared::documents::{ContractTemplate, CreatePatientDocumentRequest, PatientDocument};
use shared::patients::Patient;
use shared::users::UserResponse;

/// Modal para emissão de novos contratos clínicos vinculados a pacientes.
#[component]
pub fn IssueDocumentModal(
    token: String,
    clinic_id: String,
    is_open: Signal<bool>,
    templates: Vec<ContractTemplate>,
    patients: Vec<Patient>,
    users: Vec<UserResponse>,
    reload_trigger: Signal<usize>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
    qr_modal_doc: Signal<Option<PatientDocument>>,
) -> Element {
    let mut emit_mode = use_signal(|| "request_sign".to_string()); // "request_sign" | "upload_signed"
    let mut emit_patient_id = use_signal(String::new);
    let mut emit_doctor_id = use_signal(String::new);
    let mut emit_template_id = use_signal(String::new);
    let mut emit_doc_title = use_signal(String::new);
    let mut emit_doc_type = use_signal(|| "contract".to_string());
    let mut emit_pdf_url = use_signal(String::new);
    let mut is_uploading_doc_pdf = use_signal(|| false);
    let mut uploaded_doc_pdf_name = use_signal(String::new);
    let mut selected_patient_obj = use_signal(|| None::<Patient>);
    let mut is_submitting = use_signal(|| false);

    // Signature requirements configuration
    let mut req_patient_sign = use_signal(|| true);
    let mut req_doctor_sign = use_signal(|| false);
    let mut dentist_sign_mode = use_signal(|| "any".to_string()); // "any" | "specific"

    if !is_open() {
        return rsx! {};
    }

    let tok = token.clone();
    let handle_doc_pdf_upload = move |evt: FormEvent| {
        let t = tok.clone();
        let mut uploading_sig = is_uploading_doc_pdf;
        let mut pdf_url_sig = emit_pdf_url;
        let mut fname_sig = uploaded_doc_pdf_name;
        let mut err_sig = error_toast;

        for file in evt.files() {
            let filename = file.name();
            fname_sig.set(filename.clone());
            uploading_sig.set(true);
            let t_clone = t.clone();

            spawn(async move {
                if let Ok(bytes) = file.read_bytes().await {
                    let b64 = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &bytes,
                    );
                    match upload_document_pdf(&t_clone, &filename, &b64).await {
                        Ok(url) => {
                            pdf_url_sig.set(url);
                        }
                        Err(e) => {
                            err_sig.set(Some(format!("Erro no upload do PDF: {}", e)));
                        }
                    }
                }
                uploading_sig.set(false);
            });
        }
    };

    let tok_sub = token.clone();
    let cid_sub = clinic_id.clone();
    let mut handle_submit = move |_| {
        if emit_patient_id().trim().is_empty() {
            let mut err = error_toast;
            err.set(Some("Selecione o paciente do documento.".into()));
            return;
        }

        let is_upload_mode = emit_mode() == "upload_signed";

        if is_upload_mode && emit_pdf_url().trim().is_empty() {
            let mut err = error_toast;
            err.set(Some("Por favor, faça o upload do arquivo PDF assinado.".into()));
            return;
        }

        let tpl_id = if emit_template_id().is_empty() || is_upload_mode {
            None
        } else {
            Some(emit_template_id())
        };

        let allow_any = dentist_sign_mode() == "any";
        let doc_user_id = if req_doctor_sign() && !allow_any && !emit_doctor_id().is_empty() {
            Some(emit_doctor_id())
        } else {
            None
        };

        let title = if emit_doc_title().trim().is_empty() {
            if is_upload_mode {
                "Documento Clínico Assinado".to_string()
            } else {
                "Contrato de Prestação de Serviços Odontológicos".to_string()
            }
        } else {
            emit_doc_title().trim().to_string()
        };

        let pdf = if emit_pdf_url().is_empty() {
            None
        } else {
            Some(emit_pdf_url())
        };

        let req = CreatePatientDocumentRequest {
            clinic_id: cid_sub.clone(),
            patient_id: emit_patient_id(),
            template_id: tpl_id,
            doctor_user_id: doc_user_id,
            appointment_id: None,
            title,
            document_type: emit_doc_type(),
            pdf_url: pdf.clone(),
            signed_pdf_url: if is_upload_mode { pdf } else { None },
            is_already_signed: Some(is_upload_mode),
            requires_patient_signature: Some(req_patient_sign()),
            requires_doctor_signature: Some(req_doctor_sign()),
            allow_any_dentist_signature: Some(allow_any),
        };

        let t = tok_sub.clone();
        let mut open_sig = is_open;
        let mut rel_sig = reload_trigger;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut qr_sig = qr_modal_doc;
        let mut sub_sig = is_submitting;

        sub_sig.set(true);
        spawn(async move {
            match create_patient_document(&t, req).await {
                Ok(doc) => {
                    open_sig.set(false);
                    rel_sig.set(rel_sig() + 1);
                    if is_upload_mode {
                        toast.set(Some("Documento assinado arquivado com sucesso!".into()));
                    } else {
                        qr_sig.set(Some(doc));
                        toast.set(Some("Solicitação criada! Apresente o QR Code ou link para assinatura.".into()));
                    }
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao processar documento: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal doc-emit-modal",
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", "Emissão & Arquivamento de Documentos" }
                        p { class: "modal-subtitle", "Gere um link para assinatura digital instantânea ou arquive um documento previamente assinado." }
                    }
                    button { class: "modal-close", onclick: move |_| { let mut o = is_open; o.set(false); }, "×" }
                }

                // Mode Tabs Switcher
                div { class: "modal-nav-tabs",
                    button {
                        class: if emit_mode() == "request_sign" { "modal-tab-btn active" } else { "modal-tab-btn" },
                        onclick: move |_| emit_mode.set("request_sign".to_string()),
                        IconSignature { size: 16, color: "currentColor".to_string() }
                        span { " Nova Solicitação de Assinatura (E-Sign)" }
                    }
                    button {
                        class: if emit_mode() == "upload_signed" { "modal-tab-btn active" } else { "modal-tab-btn" },
                        onclick: move |_| emit_mode.set("upload_signed".to_string()),
                        IconUpload { size: 16, color: "currentColor".to_string() }
                        span { " Upload de Documento Já Assinado" }
                    }
                }

                div { class: "modal-body",
                    div { class: "form-group",
                        label { class: "form-label", "Paciente do Documento *" }
                        select {
                            class: "select-field",
                            value: "{emit_patient_id}",
                            onchange: {
                                let p_list = patients.clone();
                                let t_list = templates.clone();
                                move |e| {
                                    let val = e.value();
                                    emit_patient_id.set(val.clone());
                                    if let Some(p) = p_list.iter().find(|p| p.id == val) {
                                        selected_patient_obj.set(Some(p.clone()));
                                        if !emit_template_id().is_empty() {
                                            if let Some(t) = t_list.iter().find(|t| t.id == emit_template_id()) {
                                                emit_doc_title.set(format!("{} - {}", t.title, p.full_name));
                                            }
                                        }
                                    } else {
                                        selected_patient_obj.set(None);
                                    }
                                }
                            },
                            option { value: "", "Selecione o paciente cadastrado..." }
                            for p in &patients {
                                {
                                    let doc_lbl = p.document_cpf.as_deref().unwrap_or(p.document_rg.as_deref().unwrap_or("-"));
                                    rsx! {
                                        option { value: "{p.id}", "{p.full_name} ({doc_lbl})" }
                                    }
                                }
                            }
                        }
                    }

                    // Se for modo E-Sign, exibe seleção de modelo base
                    if emit_mode() == "request_sign" {
                        div { class: "form-group",
                            label { class: "form-label", "Modelo de Contrato Base *" }
                            select {
                                class: "select-field",
                                value: "{emit_template_id}",
                                onchange: {
                                    let t_list = templates.clone();
                                    move |e| {
                                        let val = e.value();
                                        emit_template_id.set(val.clone());
                                        if let Some(t) = t_list.iter().find(|t| t.id == val) {
                                            if let Some(ref p) = *selected_patient_obj.read() {
                                                emit_doc_title.set(format!("{} - {}", t.title, p.full_name));
                                            } else {
                                                emit_doc_title.set(t.title.clone());
                                            }
                                            emit_doc_type.set(t.category.clone());
                                            req_patient_sign.set(t.requires_patient_signature);
                                            req_doctor_sign.set(t.requires_doctor_signature);
                                            if !t.allow_any_dentist_signature {
                                                dentist_sign_mode.set("specific".into());
                                            }
                                        }
                                    }
                                },
                                option { value: "", "Selecione o modelo cadastrado..." }
                                for tpl in &templates {
                                    option { value: "{tpl.id}", "{tpl.title}" }
                                }
                            }
                        }
                    }

                    // Dynamic Patient Auto-Fill Details Box
                    if let Some(ref p) = *selected_patient_obj.read() {
                        div { class: "patient-autofill-card",
                            div { class: "patient-autofill-item",
                                span { class: "patient-autofill-label", "Paciente Selecionado" }
                                span { class: "patient-autofill-val", "{p.full_name}" }
                            }
                            div { class: "patient-autofill-item",
                                span { class: "patient-autofill-label", "Documento" }
                                span { class: "patient-autofill-val", "{p.document_cpf.as_deref().unwrap_or(p.document_rg.as_deref().unwrap_or(\"-\"))}" }
                            }

                            div { class: "patient-autofill-item",
                                span { class: "patient-autofill-label", "Telefone" }
                                span { class: "patient-autofill-val", "{p.phone}" }
                            }
                            div { class: "patient-autofill-item",
                                span { class: "patient-autofill-label", "Convênio" }
                                span { class: "patient-autofill-val", "{p.insurance_plan.as_deref().unwrap_or(\"Particular\")}" }
                            }
                        }
                    }

                    div { class: "form-row-2",
                        div { class: "form-group",
                            label { class: "form-label", "Título do Documento *" }
                            input {
                                r#type: "text",
                                class: "input-field",
                                placeholder: "Ex: Termo de Consentimento - Tratamento Endodôntico",
                                value: "{emit_doc_title}",
                                oninput: move |e| emit_doc_title.set(e.value()),
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Categoria do Documento" }
                            select {
                                class: "select-field",
                                value: "{emit_doc_type}",
                                onchange: move |e| emit_doc_type.set(e.value()),
                                option { value: "contract", "Contrato de Prestação de Serviços" }
                                option { value: "consent", "Termo de Consentimento (TCLE)" }
                                option { value: "orthodontics", "Contrato de Ortodontia / Alinhadores" }
                                option { value: "implant", "Contrato de Implantodontia / Cirurgia" }
                                option { value: "prescription", "Receituário / Atestado" }
                                option { value: "other", "Outro Termo / Declaração" }
                            }
                        }
                    }

                    // Configuração de Requisitos de Assinatura Digital
                    if emit_mode() == "request_sign" {
                        div { style: "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px 16px; margin-top: 4px; display: flex; flex-direction: column; gap: 12px;",
                            div { style: "display: flex; align-items: center; justify-content: space-between;",
                                span { style: "font-size: 13px; font-weight: 700; color: #0f172a;", "Requisitos de Assinatura Digital" }
                            }
                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",
                                label { style: "display: flex; align-items: center; gap: 8px; font-size: 12.5px; font-weight: 600; color: #334155; cursor: pointer;",
                                    input {
                                        r#type: "checkbox",
                                        checked: req_patient_sign(),
                                        onchange: move |e| req_patient_sign.set(e.value() == "true"),
                                    }
                                    span { "Assinatura do Paciente" }
                                }
                                label { style: "display: flex; align-items: center; gap: 8px; font-size: 12.5px; font-weight: 600; color: #334155; cursor: pointer;",
                                    input {
                                        r#type: "checkbox",
                                        checked: req_doctor_sign(),
                                        onchange: move |e| req_doctor_sign.set(e.value() == "true"),
                                    }
                                    span { "Assinatura do Dentista" }
                                }
                            }

                            if req_doctor_sign() {
                                div { style: "padding-top: 10px; border-top: 1px dashed #cbd5e1; display: flex; flex-direction: column; gap: 8px;",
                                    span { style: "font-size: 12px; font-weight: 600; color: #475569;", "Quem pode assinar como Dentista?" }
                                    div { style: "display: flex; gap: 16px;",
                                        label { style: "display: flex; align-items: center; gap: 6px; font-size: 12px; cursor: pointer;",
                                            input {
                                                r#type: "radio",
                                                name: "dentist_mode",
                                                checked: dentist_sign_mode() == "any",
                                                onchange: move |_| dentist_sign_mode.set("any".into()),
                                            }
                                            span { "Qualquer Dentista da Clínica" }
                                        }
                                        label { style: "display: flex; align-items: center; gap: 6px; font-size: 12px; cursor: pointer;",
                                            input {
                                                r#type: "radio",
                                                name: "dentist_mode",
                                                checked: dentist_sign_mode() == "specific",
                                                onchange: move |_| dentist_sign_mode.set("specific".into()),
                                            }
                                            span { "Dentista Específico" }
                                        }
                                    }
                                    if dentist_sign_mode() == "specific" {
                                        div { class: "form-group mt-2",
                                            label { class: "form-label", "Selecione o Dentista Obrigatório *" }
                                            select {
                                                class: "select-field",
                                                value: "{emit_doctor_id}",
                                                onchange: move |e| emit_doctor_id.set(e.value()),
                                                option { value: "", "Selecione o dentista responsável..." }
                                                for u in &users {
                                                    option { value: "{u.id}", "{u.full_name} ({u.role})" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Se for Upload de Documento Já Assinado, exibe dropzone obrigatório
                    if emit_mode() == "upload_signed" {
                        div { class: "form-group",
                            label { class: "form-label", "Arquivo PDF Já Assinado *" }
                            div { class: "doc-upload-dropzone",
                                input {
                                    r#type: "file",
                                    accept: ".pdf",
                                    class: "file-input-hidden",
                                    style: "display: none !important;",
                                    id: "emit-pdf-upload",
                                    onchange: handle_doc_pdf_upload,
                                }
                                label {
                                    r#for: "emit-pdf-upload",
                                    class: "upload-dropzone-label",
                                    if is_uploading_doc_pdf() {
                                        div { class: "upload-loading-spin" }
                                        span { "Processando upload do PDF assinado..." }
                                    } else if !uploaded_doc_pdf_name().is_empty() {
                                        div { class: "upload-title-row text-success",
                                            IconCheckCircle { size: 18, color: "#10b981".to_string() }
                                            span { "{uploaded_doc_pdf_name()}" }
                                        }
                                        span { class: "upload-subtitle", "Documento pronto para ser arquivado no prontuário." }
                                    } else {
                                        div { class: "upload-title-row",
                                            IconUpload { size: 18, color: "#0052cc".to_string() }
                                            span { "Clique para selecionar o PDF assinado pelo paciente" }
                                        }
                                        span { class: "upload-subtitle", "Formatos aceitos: PDF digitalizado ou assinado eletronicamente" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| { let mut o = is_open; o.set(false); }, "Cancelar" }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting() || is_uploading_doc_pdf(),
                        onclick: move |e| handle_submit(e),
                        if emit_mode() == "upload_signed" {
                            IconCheckCircle { size: 16, color: "#ffffff".to_string() }
                            span { if is_submitting() { "Arquivando..." } else { "Arquivar Documento Assinado" } }
                        } else {
                            IconSignature { size: 16, color: "#ffffff".to_string() }
                            span { if is_submitting() { "Gerando..." } else { "Emitir e Gerar QR Code" } }
                        }
                    }
                }
            }
        }
    }
}
