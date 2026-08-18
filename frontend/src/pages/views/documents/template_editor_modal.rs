//! # Editor Modal de Modelos de Contratos e Termos com Tags (Frontend)
//!
//! Permite cadastrar e editar modelos em PDF com substituição automática de dados e assinaturas via tags dinâmicas.

use crate::api::{create_template, update_template, upload_document_pdf};
use crate::components::icons::{IconCheckCircle, IconSignature, IconUpload};
use dioxus::prelude::*;
use shared::documents::{
    ContractTemplate, CreateContractTemplateRequest, UpdateContractTemplateRequest,
};

/// Modal para criação e edição de modelos de contratos e termos clínicos via tags dinâmicas.
#[component]
pub fn TemplateEditorModal(
    token: String,
    clinic_id: String,
    editing_template: Option<ContractTemplate>,
    is_open: Signal<bool>,
    reload_trigger: Signal<usize>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let initial_tpl = editing_template.clone();
    let is_editing = initial_tpl.is_some();
    let edit_id = initial_tpl.as_ref().map(|t| t.id.clone()).unwrap_or_default();

    let mut tpl_title = use_signal(|| initial_tpl.as_ref().map(|t| t.title.clone()).unwrap_or_default());
    let mut tpl_category = use_signal(|| initial_tpl.as_ref().map(|t| t.category.clone()).unwrap_or_else(|| "contract".into()));
    let mut tpl_desc = use_signal(|| initial_tpl.as_ref().and_then(|t| t.description.clone()).unwrap_or_default());
    let mut tpl_pdf_url = use_signal(|| initial_tpl.as_ref().map(|t| t.pdf_url.clone()).unwrap_or_default());

    let mut is_uploading_tpl_pdf = use_signal(|| false);
    let mut uploaded_tpl_pdf_name = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let mut copied_tag = use_signal(|| None::<String>);

    if !is_open() {
        return rsx! {};
    }

    let tok = token.clone();
    let handle_tpl_pdf_upload = move |evt: FormEvent| {
        let t = tok.clone();
        let mut uploading_sig = is_uploading_tpl_pdf;
        let mut pdf_url_sig = tpl_pdf_url;
        let mut fname_sig = uploaded_tpl_pdf_name;
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
        let title = tpl_title().trim().to_string();
        if title.is_empty() {
            let mut err = error_toast;
            err.set(Some("Por favor, informe o título do modelo.".into()));
            return;
        }

        let desc_opt = if tpl_desc().trim().is_empty() {
            None
        } else {
            Some(tpl_desc().trim().to_string())
        };

        let t = tok_sub.clone();
        let c = cid_sub.clone();
        let e_id = edit_id.clone();
        let mut open_sig = is_open;
        let mut rel_sig = reload_trigger;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_submitting;

        sub_sig.set(true);
        spawn(async move {
            if is_editing {
                let req = UpdateContractTemplateRequest {
                    clinic_id: c,
                    title,
                    category: tpl_category(),
                    description: desc_opt,
                    pdf_url: tpl_pdf_url(),
                    signature_fields: vec![],
                };
                match update_template(&t, &e_id, req).await {
                    Ok(_) => {
                        toast.set(Some("Modelo de contrato atualizado com sucesso!".into()));
                        open_sig.set(false);
                        rel_sig.set(rel_sig() + 1);
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("Erro ao atualizar modelo: {}", e)));
                    }
                }
            } else {
                let req = CreateContractTemplateRequest {
                    clinic_id: c,
                    title,
                    category: tpl_category(),
                    description: desc_opt,
                    pdf_url: tpl_pdf_url(),
                    signature_fields: vec![],
                };
                match create_template(&t, req).await {
                    Ok(_) => {
                        toast.set(Some("Modelo de contrato cadastrado com sucesso!".into()));
                        open_sig.set(false);
                        rel_sig.set(rel_sig() + 1);
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("Erro ao criar modelo: {}", e)));
                    }
                }
            }
            sub_sig.set(false);
        });
    };

    let mut copy_tag = move |tag_str: &'static str| {
        let _ = document::eval(&format!(
            r#"
            if (navigator.clipboard) {{
                navigator.clipboard.writeText("{}");
            }}
        "#,
            tag_str
        ));
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal template-modal-form", style: "width: 760px; max-width: 95vw;",
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", if is_editing { "Editar Modelo de Contrato" } else { "Novo Modelo de Contrato" } }
                        p { class: "modal-subtitle", "Cadastre o modelo em PDF e utilize as tags dinâmicas no seu documento para preenchimento e posicionamento automático das assinaturas." }
                    }
                    button { class: "modal-close", onclick: move |_| { let mut o = is_open; o.set(false); }, "×" }
                }

                div { class: "modal-body scrollable",
                    // Row 1: Título e Categoria alinhados perfeitamente lado a lado
                    div { class: "form-row-2",
                        div { class: "form-group",
                            label { class: "form-label", "Título do Modelo *" }
                            input {
                                r#type: "text",
                                class: "input-field",
                                placeholder: "Ex: Contrato de Prestação de Serviços Ortodônticos",
                                value: "{tpl_title}",
                                oninput: move |e| tpl_title.set(e.value()),
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Categoria do Documento" }
                            select {
                                class: "select-field",
                                value: "{tpl_category}",
                                onchange: move |e| tpl_category.set(e.value()),
                                option { value: "contract", "Contrato de Prestação de Serviços" }
                                option { value: "consent", "Termo de Consentimento Livre e Esclarecido (TCLE)" }
                                option { value: "orthodontics", "Contrato de Ortodontia / Alinhadores" }
                                option { value: "implant", "Contrato de Implantodontia / Cirurgia" }
                                option { value: "prescription", "Receituário / Atestado Odontológico" }
                                option { value: "other", "Outro Termo / Declaração" }
                            }
                        }
                    }

                    // Row 2: Descrição / Finalidade em linha inteira (100% largura)
                    div { class: "form-group",
                        label { class: "form-label", "Descrição / Finalidade" }
                        input {
                            r#type: "text",
                            class: "input-field",
                            placeholder: "Ex: Modelo padrão para procedimentos cirúrgicos e implantes com cláusulas contratuais",
                            value: "{tpl_desc}",
                            oninput: move |e| tpl_desc.set(e.value()),
                        }
                    }

                    // Row 3: Upload do Arquivo PDF do Modelo Base
                    div { class: "form-group",
                        label { class: "form-label", "Arquivo PDF do Modelo Base" }
                        div { class: "doc-upload-dropzone",
                            input {
                                r#type: "file",
                                accept: ".pdf",
                                class: "file-input-hidden",
                                style: "display: none !important;",
                                id: "tpl-pdf-upload",
                                onchange: handle_tpl_pdf_upload,
                            }
                            label {
                                r#for: "tpl-pdf-upload",
                                class: "upload-dropzone-label",
                                if is_uploading_tpl_pdf() {
                                    div { class: "upload-loading-spin" }
                                    span { "Enviando PDF do modelo..." }
                                } else if !uploaded_tpl_pdf_name().is_empty() {
                                    div { class: "upload-title-row text-success",
                                        IconCheckCircle { size: 16, color: "#10b981".to_string() }
                                        span { "{uploaded_tpl_pdf_name()}" }
                                    }
                                    span { class: "upload-subtitle", "PDF base pronto para o modelo." }
                                } else if !tpl_pdf_url().is_empty() {
                                    div { class: "upload-title-row text-success",
                                        IconCheckCircle { size: 16, color: "#10b981".to_string() }
                                        span { "PDF Base Anexado ✓" }
                                    }
                                    span { class: "upload-subtitle", "Clique para alterar o arquivo PDF base." }
                                } else {
                                    div { class: "upload-title-row",
                                        IconUpload { size: 16, color: "#0052cc".to_string() }
                                        span { "Fazer Upload do PDF do Modelo" }
                                    }
                                    span { class: "upload-subtitle", "Arquivo com layout e tags para preenchimento" }
                                }
                            }
                        }
                    }

                    // Catálogo de Tags Dinâmicas Dividido por Categorias
                    div { class: "tags-catalog-container",
                        div { class: "tags-catalog-header",
                            div {
                                h4 { class: "tags-catalog-title", "🏷️ Catálogo de Tags Dinâmicas" }
                                p { class: "tags-catalog-subtitle", "Clique sobre qualquer tag para copiá-la para a área de transferência." }
                            }
                        }

                        div { class: "tags-categories-grid",
                            // Categoria 1: Dados do Paciente
                            div { class: "tag-category-card",
                                span { class: "tag-category-label", "👤 Dados do Paciente" }
                                div { class: "tag-chips-flex",
                                    for tag_name in &[
                                        "{{paciente_nome}}",
                                        "{{paciente_cpf}}",
                                        "{{paciente_rg}}",
                                        "{{paciente_telefone}}",
                                        "{{paciente_email}}",
                                        "{{paciente_endereco}}",
                                        "{{paciente_convenio}}",
                                        "{{paciente_data_nascimento}}",
                                    ] {
                                        button {
                                            r#type: "button",
                                            key: "{tag_name}",
                                            class: "dynamic-tag-badge",
                                            onclick: move |_| copy_tag(tag_name),
                                            title: "Clique para copiar {tag_name}",
                                            "{tag_name}"
                                        }
                                    }
                                }
                            }

                            // Categoria 2: Dados do Dentista / Profissional
                            div { class: "tag-category-card",
                                span { class: "tag-category-label", "🩺 Cirurgião-Dentista" }
                                div { class: "tag-chips-flex",
                                    for tag_name in &[
                                        "{{dentista_nome}}",
                                        "{{dentista_cro}}",
                                        "{{dentista_especialidade}}",
                                        "{{dentista_cpf}}",
                                        "{{dentista_email}}",
                                    ] {
                                        button {
                                            r#type: "button",
                                            key: "{tag_name}",
                                            class: "dynamic-tag-badge badge-doctor",
                                            onclick: move |_| copy_tag(tag_name),
                                            title: "Clique para copiar {tag_name}",
                                            "{tag_name}"
                                        }
                                    }
                                }
                            }

                            // Categoria 3: Dados da Clínica
                            div { class: "tag-category-card",
                                span { class: "tag-category-label", "🏥 Dados da Clínica" }
                                div { class: "tag-chips-flex",
                                    for tag_name in &[
                                        "{{clinica_nome}}",
                                        "{{clinica_cnpj}}",
                                        "{{clinica_cro_responsavel}}",
                                        "{{clinica_endereco}}",
                                        "{{clinica_telefone}}",
                                        "{{clinica_cidade_uf}}",
                                    ] {
                                        button {
                                            r#type: "button",
                                            key: "{tag_name}",
                                            class: "dynamic-tag-badge badge-clinic",
                                            onclick: move |_| copy_tag(tag_name),
                                            title: "Clique para copiar {tag_name}",
                                            "{tag_name}"
                                        }
                                    }
                                }
                            }

                            // Categoria 4: Assinaturas Digitais e Datas
                            div { class: "tag-category-card",
                                span { class: "tag-category-label", "✍️ Assinaturas & Datas" }
                                div { class: "tag-chips-flex",
                                    for tag_name in &[
                                        "{{assinatura_paciente}}",
                                        "{{assinatura_doutor}}",
                                        "{{data_hoje}}",
                                        "{{hora_atual}}",
                                    ] {
                                        button {
                                            r#type: "button",
                                            key: "{tag_name}",
                                            class: "dynamic-tag-badge badge-signature",
                                            onclick: move |_| copy_tag(tag_name),
                                            title: "Clique para copiar {tag_name}",
                                            "{tag_name}"
                                        }
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
                        disabled: is_submitting() || is_uploading_tpl_pdf(),
                        onclick: move |e| handle_submit(e),
                        IconCheckCircle { size: 16, color: "#ffffff".to_string() }
                        span { if is_submitting() { "Salvando..." } else { "Salvar Modelo de Contrato" } }
                    }
                }
            }
        }
    }
}
