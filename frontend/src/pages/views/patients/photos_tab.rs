//! # Aba de Exames e Galeria de Fotos Intraorais / Radiografias (Frontend)
//!
//! Exibe a galeria de imagens radiográficas, laudos e fotos odontológicas do paciente,
//! com visualizador em tela cheia (zoom) e envio de novos arquivos.

use crate::api::{create_patient_exam, upload_document_pdf};
use crate::components::icons::{IconCheckCircle, IconEye, IconPlus, IconUpload};
use dioxus::prelude::*;
use shared::patients::{CreatePatientExamRequest, PatientExam};

/// Componente da aba de exames, laudos e fotos clínicas do paciente.
#[component]
pub fn PatientPhotosTab(
    patient_id: String,
    clinic_id: String,
    exams: Vec<PatientExam>,
    can_write: bool,
    token: String,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut is_create_modal_open = use_signal(|| false);
    let mut selected_preview_url = use_signal(|| None::<String>);

    let mut exam_title = use_signal(String::new);
    let mut exam_type = use_signal(|| "Radiografia Panorâmica".to_string());
    let mut exam_notes = use_signal(String::new);
    let mut exam_file_url = use_signal(String::new);
    let mut is_uploading_file = use_signal(|| false);
    let mut uploaded_filename = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();

    let handle_file_change = move |evt: FormEvent| {
        let t = tok.clone();
        let mut uploading_sig = is_uploading_file;
        let mut file_url_sig = exam_file_url;
        let mut fname_sig = uploaded_filename;
        let mut err_sig = error_toast;

        for file in evt.files() {
            let filename = file.name();
            fname_sig.set(filename.clone());
            uploading_sig.set(true);
            let t_clone = t.clone();

            spawn(async move {
                if let Ok(bytes) = file.read_bytes().await {
                    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
                    match upload_document_pdf(&t_clone, &filename, &b64).await {
                        Ok(url) => {
                            file_url_sig.set(url);
                        }
                        Err(e) => {
                            err_sig.set(Some(format!("Erro no upload: {}", e)));
                        }
                    }
                }
                uploading_sig.set(false);
            });
        }
    };

    let tok_sub = token.clone();
    let pat_id_sub = patient_id.clone();
    let cid_sub = clinic_id.clone();
    let on_reload = reload_patient_details.clone();

    let mut handle_submit = move |_| {
        let title = exam_title().trim().to_string();
        if title.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o título do exame/foto.".into()));
            return;
        }

        let file_url = exam_file_url().trim().to_string();
        let file_urls = if file_url.is_empty() {
            vec![]
        } else {
            vec![file_url]
        };

        let req = CreatePatientExamRequest {
            clinic_id: cid_sub.clone(),
            title,
            exam_type: exam_type(),
            requested_date: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
            result_date: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
            file_urls,
            clinical_interpretation: if exam_notes().trim().is_empty() { None } else { Some(exam_notes().trim().to_string()) },
        };

        let t = tok_sub.clone();
        let p = pat_id_sub.clone();
        let mut open_sig = is_create_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_submitting;
        let reload = on_reload.clone();

        sub_sig.set(true);
        spawn(async move {
            match create_patient_exam(&t, &p, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    toast.set(Some("Exame anexado com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao registrar exame: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "patient-tab-content",
            div { class: "tab-header-actions-row",
                div { class: "tab-header-title-group",
                    h3 { class: "tab-header-title", "Galeria de Exames e Radiografias" }
                    p { class: "tab-header-desc", "Radiografias panorâmicas, periapicais, tomografias e fotos intraorais." }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| is_create_modal_open.set(true),
                        IconUpload { size: 16, color: "#ffffff".to_string() }
                        span { " Novo Exame / Laudo" }
                    }
                }
            }

            if exams.is_empty() {
                div { class: "empty-state-card",
                    div { class: "empty-state-icon-box",
                        IconEye { size: 32, color: "currentColor".to_string() }
                    }
                    h3 { "Nenhum exame ou laudo anexado" }
                    p { "Adicione radiografias panorâmicas, periapicais, fotos intraorais ou tomografias ao prontuário deste paciente." }
                }
            } else {
                div { class: "exams-gallery-grid",
                    for exam in &exams {
                        {
                            let first_url = exam.file_urls.first().cloned();
                            let first_url_prev = first_url.clone();
                            let has_file = first_url.is_some();

                            rsx! {
                                div { key: "{exam.id}", class: "exam-gallery-card",
                                    div { class: "exam-card-preview",
                                        if let Some(ref u) = first_url {
                                            img {
                                                src: "{u}",
                                                alt: "{exam.title}",
                                                class: "exam-thumbnail-img",
                                                onclick: move |_| selected_preview_url.set(first_url_prev.clone())
                                            }
                                        } else {
                                            div { class: "exam-no-img",
                                                IconUpload { size: 32, color: "#94a3b8".to_string() }
                                            }
                                        }
                                    }
                                    div { class: "exam-card-info",
                                        h4 { class: "exam-title", "{exam.title}" }
                                        span { class: "badge-outline", "{exam.exam_type}" }
                                        p { class: "text-muted font-xs mt-1", "Data: {exam.requested_date}" }
                                        if let Some(ref notes) = exam.clinical_interpretation {
                                            p { class: "exam-notes font-xs mt-2", "{notes}" }
                                        }
                                    }
                                    if has_file {
                                        div { class: "p-3 pt-0",
                                            button {
                                                class: "btn-secondary btn-sm w-full flex items-center justify-center gap-1",
                                                onclick: move |_| selected_preview_url.set(first_url.clone()),
                                                IconEye { size: 14, color: "currentColor".to_string() }
                                                span { "Visualizar Imagem" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ref preview_url) = *selected_preview_url.read() {
                div { class: "modal-overlay", onclick: move |_| selected_preview_url.set(None),
                    div { class: "preview-lightbox-container", onclick: move |e| e.stop_propagation(),
                        button { class: "lightbox-close-btn", onclick: move |_| selected_preview_url.set(None), "×" }
                        img { src: "{preview_url}", class: "lightbox-image", alt: "Visualização do Exame" }
                    }
                }
            }

            // Modal Aprimorado: Anexar Exame / Foto ao Prontuário
            if is_create_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal", style: "max-width: 620px;",
                        div { class: "settings-header",
                            div {
                                h2 { class: "settings-title", "Anexar Exame / Foto ao Prontuário" }
                                p { class: "text-muted font-xs mt-1",
                                    "Faça upload de radiografias panorâmicas, fotos intraorais ou tomografias."
                                }
                            }
                            button { class: "close-btn", onclick: move |_| is_create_modal_open.set(false), "×" }
                        }
                        div { class: "settings-content",
                            div { class: "form-grid-2",
                                div { class: "form-group",
                                    label { "Título do Exame *" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Radiografia Panorâmica Inicial",
                                        value: "{exam_title}",
                                        oninput: move |e| exam_title.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Tipo de Exame" }
                                    select {
                                        class: "form-input",
                                        value: "{exam_type}",
                                        onchange: move |e| exam_type.set(e.value()),
                                        option { value: "Radiografia Panorâmica", "Radiografia Panorâmica / Periapical" }
                                        option { value: "Fotografia Intraoral", "Fotografia Intraoral" }
                                        option { value: "Tomografia Computadorizada (TC)", "Tomografia Computadorizada (TC)" }
                                        option { value: "Exame Laboratorial / Biópsia", "Exame Laboratorial / Biópsia" }
                                        option { value: "Outro Documento de Imagem", "Outro Documento de Imagem" }
                                    }
                                }
                            }

                            // Custom Dropzone
                            div { class: "form-group",
                                label { "Arquivo do Exame (Imagem ou PDF)" }
                                div {
                                    class: if !uploaded_filename().is_empty() { "patient-upload-dropzone has-file" } else { "patient-upload-dropzone" },
                                    input {
                                        class: "patient-upload-hidden-input",
                                        r#type: "file",
                                        accept: "image/*,.pdf",
                                        onchange: move |e| handle_file_change(e)
                                    }
                                    div { class: "patient-upload-icon-wrap",
                                        if !uploaded_filename().is_empty() {
                                            IconCheckCircle { size: 22, color: "currentColor".to_string() }
                                        } else {
                                            IconUpload { size: 22, color: "currentColor".to_string() }
                                        }
                                    }
                                    if is_uploading_file() {
                                        p { class: "patient-upload-main-text text-primary", "Enviando arquivo..." }
                                    } else if !uploaded_filename().is_empty() {
                                        p { class: "patient-upload-main-text text-success", "✓ Arquivo: {uploaded_filename}" }
                                        span { class: "patient-upload-sub-text", "Clique ou arraste outro arquivo para substituir" }
                                    } else {
                                        p { class: "patient-upload-main-text", "Clique para selecionar ou arraste o arquivo até aqui" }
                                        span { class: "patient-upload-sub-text", "Suporta imagens (PNG, JPG, JPEG) ou documentos PDF (até 15MB)" }
                                    }
                                }
                            }

                            div { class: "form-group",
                                label { "Interpretação Clínica / Laudo" }
                                textarea {
                                    class: "form-input",
                                    style: "min-height: 85px; resize: vertical;",
                                    placeholder: "Ex: Presença de lesão periapical no elemento 36, sem reabsorção óssea severa...",
                                    value: "{exam_notes}",
                                    oninput: move |e| exam_notes.set(e.value())
                                }
                            }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| is_create_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_submitting() || is_uploading_file(),
                                onclick: move |e| handle_submit(e),
                                if is_submitting() { "Salvando..." } else { "Salvar Exame" }
                            }
                        }
                    }
                }
            }
        }
    }
}
