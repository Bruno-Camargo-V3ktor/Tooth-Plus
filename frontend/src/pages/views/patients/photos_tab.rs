//! # Aba de Exames e Galeria de Fotos Intraorais / Radiografias (Frontend)
//!
//! Exibe a galeria de imagens radiográficas, laudos e fotos odontológicas do paciente,
//! com visualizador em tela cheia (lightbox), upload de novos arquivos e edição completa.

use crate::api::{create_patient_exam, delete_patient_exam, update_patient_exam, upload_document_pdf};
use crate::components::icons::{
    IconCheckCircle, IconEdit, IconEye, IconFile, IconTrash, IconUpload,
};
use crate::utils::resolve_file_url;
use dioxus::prelude::*;
use shared::patients::{CreatePatientExamRequest, PatientExam, UpdatePatientExamRequest};

/// Formata a data ISO para o padrão brasileiro DD/MM/YYYY.
fn format_exam_date(iso_str: &str) -> String {
    if iso_str.len() >= 10 {
        let parts: Vec<&str> = iso_str[0..10].split('-').collect();
        if parts.len() == 3 {
            let date_formatted = format!("{}/{}/{}", parts[2], parts[1], parts[0]);
            if iso_str.len() >= 16 && iso_str.contains('T') {
                let time = &iso_str[11..16];
                return format!("{} às {}", date_formatted, time);
            }
            return date_formatted;
        }
    }
    iso_str.to_string()
}

/// Componente da aba de exames, laudos e fotos clínicas do paciente.
#[component]
pub fn PatientPhotosTab(
    patient_id: String,
    clinic_id: String,
    exams: Vec<PatientExam>,
    can_write: bool,
    can_delete: bool,
    token: String,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut is_create_modal_open = use_signal(|| false);
    let mut editing_exam = use_signal(|| None::<PatientExam>);
    let mut selected_preview_url = use_signal(|| None::<String>);
    let mut deleting_exam_id = use_signal(|| None::<String>);
    let mut is_deleting = use_signal(|| false);

    // Estado do formulário de criação
    let mut exam_title = use_signal(String::new);
    let mut exam_type = use_signal(|| "Radiografia Panorâmica".to_string());
    let mut exam_notes = use_signal(String::new);
    let mut exam_file_url = use_signal(String::new);
    let mut is_uploading_file = use_signal(|| false);
    let mut uploaded_filename = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    // Estado do formulário de edição
    let mut edit_title = use_signal(String::new);
    let mut edit_type = use_signal(|| "Radiografia Panorâmica".to_string());
    let mut edit_notes = use_signal(String::new);
    let mut edit_file_url = use_signal(String::new);
    let mut is_edit_uploading = use_signal(|| false);
    let mut edit_uploaded_filename = use_signal(String::new);
    let mut is_edit_submitting = use_signal(|| false);

    let tok_upload = token.clone();
    let handle_file_change = move |evt: FormEvent| {
        let t = tok_upload.clone();
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

    let tok_edit_upload = token.clone();
    let handle_edit_file_change = move |evt: FormEvent| {
        let t = tok_edit_upload.clone();
        let mut uploading_sig = is_edit_uploading;
        let mut file_url_sig = edit_file_url;
        let mut fname_sig = edit_uploaded_filename;
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

    let tok_edit_sub = token.clone();
    let pat_id_edit_sub = patient_id.clone();
    let cid_edit_sub = clinic_id.clone();
    let on_reload_edit = reload_patient_details.clone();

    let mut handle_edit_submit = move |_| {
        let Some(ref current_exam) = *editing_exam.read() else { return; };
        let exam_id = current_exam.id.clone();
        let title = edit_title().trim().to_string();
        if title.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o título do exame/foto.".into()));
            return;
        }

        let file_url = edit_file_url().trim().to_string();
        let file_urls = if file_url.is_empty() {
            vec![]
        } else {
            vec![file_url]
        };

        let req = UpdatePatientExamRequest {
            clinic_id: cid_edit_sub.clone(),
            title,
            exam_type: edit_type(),
            status: Some("received".to_string()),
            requested_date: None,
            result_date: None,
            file_urls,
            clinical_interpretation: if edit_notes().trim().is_empty() { None } else { Some(edit_notes().trim().to_string()) },
        };

        let t = tok_edit_sub.clone();
        let p = pat_id_edit_sub.clone();
        let mut edit_modal_sig = editing_exam;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_edit_submitting;
        let reload = on_reload_edit.clone();

        sub_sig.set(true);
        spawn(async move {
            match update_patient_exam(&t, &p, &exam_id, req).await {
                Ok(_) => {
                    edit_modal_sig.set(None);
                    toast.set(Some("Exame atualizado com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao atualizar exame: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    let mut open_edit_modal = move |exam: PatientExam| {
        edit_title.set(exam.title.clone());
        edit_type.set(exam.exam_type.clone());
        edit_notes.set(exam.clinical_interpretation.clone().unwrap_or_default());
        edit_file_url.set(exam.file_urls.first().cloned().unwrap_or_default());
        edit_uploaded_filename.set(String::new());
        editing_exam.set(Some(exam));
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
                        onclick: move |_| {
                            exam_title.set(String::new());
                            exam_notes.set(String::new());
                            exam_file_url.set(String::new());
                            uploaded_filename.set(String::new());
                            is_create_modal_open.set(true);
                        },
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
                div { class: "exams-gallery-grid", style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 20px; margin-top: 16px;",
                    for exam in &exams {
                        {
                            let first_url_raw = exam.file_urls.first().cloned();
                            let first_url = first_url_raw.as_deref().map(resolve_file_url);
                            let first_url_prev = first_url.clone();
                            let has_file = first_url.is_some();
                            let is_pdf = first_url_raw.as_deref().map(|u| u.to_lowercase().ends_with(".pdf")).unwrap_or(false);
                            let exam_clone = exam.clone();
                            let dt_label = format_exam_date(&exam.requested_date);

                            rsx! {
                                div { key: "{exam.id}", class: "exam-gallery-card", style: "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; overflow: hidden; display: flex; flex-direction: column; box-shadow: 0 2px 6px rgba(15, 23, 42, 0.04); transition: transform 0.2s ease, box-shadow 0.2s ease;",
                                    // Thumbnail Box
                                    div {
                                        class: "exam-preview-thumb",
                                        style: "height: 160px; background: #0f172a; display: flex; align-items: center; justify-content: center; overflow: hidden; position: relative; cursor: pointer;",
                                        onclick: {
                                            let u_click = first_url_prev.clone();
                                            move |_| {
                                                if let Some(ref u) = u_click {
                                                    selected_preview_url.set(Some(u.clone()));
                                                }
                                            }
                                        },
                                        if let Some(ref u) = first_url {
                                            if is_pdf {
                                                div { style: "display: flex; flex-direction: column; align-items: center; gap: 8px; color: #f8fafc;",
                                                    IconFile { size: 36, color: "#38bdf8".to_string() }
                                                    span { style: "font-size: 11px; font-weight: 600;", "Documento PDF" }
                                                }
                                            } else {
                                                img {
                                                    src: "{u}",
                                                    alt: "{exam.title}",
                                                    style: "width: 100%; height: 100%; object-fit: cover; transition: transform 0.2s;",
                                                }
                                            }
                                        } else {
                                            div { style: "display: flex; flex-direction: column; align-items: center; gap: 4px; color: #64748b;",
                                                IconUpload { size: 32, color: "#64748b".to_string() }
                                                span { style: "font-size: 11px; font-weight: 500;", "Sem imagem anexada" }
                                            }
                                        }
                                    }

                                    // Content Info
                                    div { class: "exam-card-info", style: "padding: 14px; display: flex; flex-direction: column; gap: 8px; flex: 1;",
                                        div { style: "display: flex; align-items: flex-start; justify-content: space-between; gap: 8px;",
                                            h4 { style: "font-size: 14px; font-weight: 700; color: #0f172a; margin: 0; line-height: 1.3;", "{exam.title}" }
                                            span { class: "badge-outline", style: "font-size: 10px; padding: 2px 6px; border-radius: 4px; white-space: nowrap; flex-shrink: 0;", "{exam.exam_type}" }
                                        }

                                        div { style: "font-size: 11px; color: #64748b; margin: 0; display: flex; align-items: center; gap: 4px;",
                                            span { "Data: " }
                                            strong { style: "color: #334155;", "{dt_label}" }
                                        }

                                        if let Some(ref notes) = exam.clinical_interpretation {
                                            if !notes.trim().is_empty() {
                                                div { style: "background: #f8fafc; border-left: 3px solid #0052cc; padding: 6px 10px; border-radius: 4px; font-size: 11px; color: #334155; line-height: 1.4; margin-top: 4px; max-height: 60px; overflow-y: auto;",
                                                    "{notes}"
                                                }
                                            }
                                        }
                                    }

                                    // Action Buttons Footer
                                    div { style: "padding: 10px 14px; border-top: 1px solid #f1f5f9; background: #fafafa; display: flex; align-items: center; justify-content: space-between; gap: 8px;",
                                        if has_file {
                                            button {
                                                class: "btn-secondary btn-sm",
                                                style: "flex: 1; display: inline-flex; align-items: center; justify-content: center; gap: 6px; font-size: 12px; padding: 6px 10px;",
                                                onclick: {
                                                    let u_prev = first_url.clone();
                                                    move |_| selected_preview_url.set(u_prev.clone())
                                                },
                                                IconEye { size: 14, color: "currentColor".to_string() }
                                                span { "Visualizar" }
                                            }
                                        }
                                        if can_write {
                                            button {
                                                class: "btn-secondary btn-sm",
                                                style: "display: inline-flex; align-items: center; justify-content: center; gap: 6px; font-size: 12px; padding: 6px 10px; color: #0052cc; border-color: #bfdbfe; background: #eff6ff;",
                                                title: "Editar exame/laudo",
                                                onclick: {
                                                    let ex_to_edit = exam_clone.clone();
                                                    move |_| open_edit_modal(ex_to_edit.clone())
                                                },
                                                IconEdit { size: 14, color: "#0052cc".to_string() }
                                                span { "Editar" }
                                            }
                                        }
                                        if can_delete {
                                            {
                                                let e_id = exam.id.clone();
                                                rsx! {
                                                    button {
                                                        class: "btn-danger-ghost btn-sm",
                                                        style: "padding: 6px 8px;",
                                                        title: "Excluir exame",
                                                        onclick: move |_| deleting_exam_id.set(Some(e_id.clone())),
                                                        IconTrash { size: 14, color: "#ef4444".to_string() }
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

            // Modal de Confirmação de Exclusão
            if let Some(ref e_id) = *deleting_exam_id.read() {
                {
                    let exam_id_val = e_id.clone();
                    let t_del = token.clone();
                    let p_del = patient_id.clone();
                    let c_del = clinic_id.clone();
                    let on_rel = reload_patient_details.clone();

                    rsx! {
                        div { class: "modal-overlay",
                            div { class: "action-modal confirm-delete-modal", style: "max-width: 440px;",
                                div { class: "confirm-delete-body",
                                    div { class: "confirm-delete-icon-box", "🗑️" }
                                    h3 { class: "confirm-delete-title", "Excluir Exame / Foto" }
                                    p { class: "confirm-delete-text",
                                        "Tem certeza que deseja excluir este exame do prontuário? Esta ação não pode ser desfeita."
                                    }
                                }
                                div { class: "confirm-delete-actions",
                                    button {
                                        class: "btn-secondary",
                                        disabled: is_deleting(),
                                        onclick: move |_| deleting_exam_id.set(None),
                                        "Cancelar"
                                    }
                                    button {
                                        class: "btn-danger",
                                        disabled: is_deleting(),
                                        onclick: move |_| {
                                            let mut is_del = is_deleting;
                                            let mut del_id = deleting_exam_id;
                                            let mut toast = toast_msg;
                                            let mut err_sig = error_toast;
                                            let rel = on_rel.clone();
                                            let t = t_del.clone();
                                            let p = p_del.clone();
                                            let c = c_del.clone();
                                            let eid = exam_id_val.clone();

                                            is_del.set(true);
                                            spawn(async move {
                                                match delete_patient_exam(&t, &p, &eid, &c).await {
                                                    Ok(_) => {
                                                        del_id.set(None);
                                                        toast.set(Some("Exame excluído com sucesso!".into()));
                                                        rel.call(());
                                                    }
                                                    Err(e) => {
                                                        err_sig.set(Some(format!("Erro ao excluir exame: {}", e)));
                                                    }
                                                }
                                                is_del.set(false);
                                            });
                                        },
                                        if is_deleting() { "Excluindo..." } else { "Sim, Excluir" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Lightbox Preview
            if let Some(ref preview_url) = *selected_preview_url.read() {
                div { class: "modal-overlay", onclick: move |_| selected_preview_url.set(None),
                    div { class: "preview-lightbox-container", onclick: move |e| e.stop_propagation(),
                        button { class: "lightbox-close-btn", onclick: move |_| selected_preview_url.set(None), "×" }
                        img { src: "{preview_url}", class: "lightbox-image", alt: "Visualização do Exame" }
                    }
                }
            }

            // Modal: Anexar Novo Exame / Foto ao Prontuário
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
                                        option { value: "Radiografia Panorâmica", "Radiografia Panorâmica" }
                                        option { value: "Radiografia Periapical", "Radiografia Periapical" }
                                        option { value: "Fotografia Intraoral", "Fotografia Intraoral" }
                                        option { value: "Tomografia Computadorizada (TC)", "Tomografia Computadorizada (TC)" }
                                        option { value: "Telerradiografia", "Telerradiografia Lateral / Frontal" }
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

            // Modal: Editar Exame / Foto do Prontuário
            if editing_exam().is_some() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal", style: "max-width: 620px;",
                        div { class: "settings-header",
                            div {
                                h2 { class: "settings-title", "Editar Dados do Exame / Laudo" }
                                p { class: "text-muted font-xs mt-1",
                                    "Atualize o título, categoria, interpretação clínica ou anexo."
                                }
                            }
                            button { class: "close-btn", onclick: move |_| editing_exam.set(None), "×" }
                        }
                        div { class: "settings-content",
                            div { class: "form-grid-2",
                                div { class: "form-group",
                                    label { "Título do Exame *" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Radiografia Panorâmica Inicial",
                                        value: "{edit_title}",
                                        oninput: move |e| edit_title.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Tipo de Exame" }
                                    select {
                                        class: "form-input",
                                        value: "{edit_type}",
                                        onchange: move |e| edit_type.set(e.value()),
                                        option { value: "Radiografia Panorâmica", "Radiografia Panorâmica" }
                                        option { value: "Radiografia Periapical", "Radiografia Periapical" }
                                        option { value: "Fotografia Intraoral", "Fotografia Intraoral" }
                                        option { value: "Tomografia Computadorizada (TC)", "Tomografia Computadorizada (TC)" }
                                        option { value: "Telerradiografia", "Telerradiografia Lateral / Frontal" }
                                        option { value: "Exame Laboratorial / Biópsia", "Exame Laboratorial / Biópsia" }
                                        option { value: "Outro Documento de Imagem", "Outro Documento de Imagem" }
                                    }
                                }
                            }

                            // Custom Dropzone de Substituição
                            div { class: "form-group",
                                label { "Substituir Arquivo do Exame (Opcional)" }
                                div {
                                    class: if !edit_uploaded_filename().is_empty() { "patient-upload-dropzone has-file" } else { "patient-upload-dropzone" },
                                    input {
                                        class: "patient-upload-hidden-input",
                                        r#type: "file",
                                        accept: "image/*,.pdf",
                                        onchange: move |e| handle_edit_file_change(e)
                                    }
                                    div { class: "patient-upload-icon-wrap",
                                        if !edit_uploaded_filename().is_empty() {
                                            IconCheckCircle { size: 22, color: "currentColor".to_string() }
                                        } else {
                                            IconUpload { size: 22, color: "currentColor".to_string() }
                                        }
                                    }
                                    if is_edit_uploading() {
                                        p { class: "patient-upload-main-text text-primary", "Enviando novo arquivo..." }
                                    } else if !edit_uploaded_filename().is_empty() {
                                        p { class: "patient-upload-main-text text-success", "✓ Novo Arquivo: {edit_uploaded_filename}" }
                                        span { class: "patient-upload-sub-text", "Clique ou arraste outro para substituir" }
                                    } else if !edit_file_url().is_empty() {
                                        p { class: "patient-upload-main-text text-primary", "Arquivo atual já anexado" }
                                        span { class: "patient-upload-sub-text", "Clique ou arraste um novo arquivo se desejar substituir" }
                                    } else {
                                        p { class: "patient-upload-main-text", "Clique para selecionar ou arraste o arquivo" }
                                        span { class: "patient-upload-sub-text", "Suporta imagens (PNG, JPG, JPEG) ou PDFs (até 15MB)" }
                                    }
                                }
                            }

                            div { class: "form-group",
                                label { "Interpretação Clínica / Laudo" }
                                textarea {
                                    class: "form-input",
                                    style: "min-height: 85px; resize: vertical;",
                                    placeholder: "Ex: Presença de lesão periapical no elemento 36...",
                                    value: "{edit_notes}",
                                    oninput: move |e| edit_notes.set(e.value())
                                }
                            }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| editing_exam.set(None), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_edit_submitting() || is_edit_uploading(),
                                onclick: move |e| handle_edit_submit(e),
                                if is_edit_submitting() { "Salvando..." } else { "Salvar Alterações" }
                            }
                        }
                    }
                }
            }
        }
    }
}
