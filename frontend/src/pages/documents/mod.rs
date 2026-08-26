pub mod components;

use crate::api::documents::DocumentsApi;
use crate::api::patients::PatientsApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use shared::documents::{ContractTemplate, CreatePatientDocumentRequest, PatientDocument};
use shared::patients::Patient;
use dioxus::prelude::*;

pub use components::*;

const STYLE: Asset = asset!("/src/pages/documents/style.css");

#[component]
pub fn DocumentsView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let clinic_name = active_clinic
        .read()
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "SmilePlus Odontologia".to_string());

    let mut active_tab = use_signal(|| DocumentTab::Issued);
    let mut documents_list = use_signal(Vec::<PatientDocument>::new);
    let mut templates_list = use_signal(Vec::<ContractTemplate>::new);
    let mut patients_list = use_signal(Vec::<Patient>::new);

    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "ALL".to_string());
    let mut reload_trigger = use_signal(|| 0);

    let mut is_issue_modal_open = use_signal(|| false);
    let mut qr_doc_id = use_signal(|| None::<String>);
    let mut preview_doc_id = use_signal(|| None::<String>);

    let mut issue_patient_id = use_signal(String::new);
    let mut issue_template_id = use_signal(String::new);
    let mut issue_title = use_signal(String::new);
    let mut issue_type = use_signal(|| "contract".to_string());
    let mut req_pat_sig = use_signal(|| true);
    let mut req_doc_sig = use_signal(|| false);
    let mut is_already_signed = use_signal(|| false);

    let cid_effect = clinic_id.clone();
    use_effect(move || {
        let _ = reload_trigger.read();
        let cid = cid_effect.clone();
        spawn(async move {
            if let Ok(docs) = DocumentsApi::list_documents(&cid).await {
                documents_list.set(docs);
            }
            if let Ok(tpls) = DocumentsApi::list_templates(&cid).await {
                templates_list.set(tpls);
            }
            if let Ok(pats) = PatientsApi::list_patients(None).await {
                patients_list.set(pats.items);
            }
        });
    });

    let handle_issue_submit = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut modal_sig = is_issue_modal_open;
        let mut reload_sig = reload_trigger;

        let pid_sig = issue_patient_id.clone();
        let tid_sig = issue_template_id.clone();
        let title_sig = issue_title.clone();
        let type_sig = issue_type.clone();
        let ps_sig = req_pat_sig;
        let ds_sig = req_doc_sig;
        let als_sig = is_already_signed;

        move |_| {
            let pid = pid_sig.read().trim().to_string();
            let title = title_sig.read().trim().to_string();

            if pid.is_empty() {
                toast_c.show("Selecione o paciente.", ToastVariant::Error);
                return;
            }
            if title.is_empty() {
                toast_c.show("Informe o título do documento.", ToastVariant::Error);
                return;
            }

            let tid_val = if tid_sig.read().is_empty() { None } else { Some(tid_sig.read().clone()) };

            let req = CreatePatientDocumentRequest {
                clinic_id: cid.clone(),
                patient_id: pid,
                template_id: tid_val,
                doctor_user_id: Some("usr:dr_lucas".to_string()),
                appointment_id: None,
                title,
                document_type: type_sig.read().clone(),
                pdf_url: Some("/docs/modelo.pdf".to_string()),
                signed_pdf_url: None,
                is_already_signed: Some(*als_sig.read()),
                requires_patient_signature: Some(*ps_sig.read()),
                requires_doctor_signature: Some(*ds_sig.read()),
                allow_any_dentist_signature: Some(true),
            };

            let mut toast_resp = toast_c.clone();
            let mut modal_c = modal_sig;
            let mut reload_c = reload_sig;

            spawn(async move {
                match DocumentsApi::create_document(req).await {
                    Ok(_) => {
                        toast_resp.show("Documento emitido com sucesso!", ToastVariant::Success);
                        modal_c.set(false);
                        reload_c.set(reload_c() + 1);
                    }
                    Err(err) => toast_resp.show(err, ToastVariant::Error),
                }
            });
        }
    };

    let filtered_documents: Vec<PatientDocument> = documents_list.read().iter().filter(|d| {
        let sf = status_filter.read().clone();
        if sf == "pending" && d.status != "pending" { return false; }
        if sf == "signed" && d.status != "signed" { return false; }

        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() { return true; }
        d.title.to_lowercase().contains(&q)
            || d.patient_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
            || d.doctor_user_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
    }).cloned().collect();

    let filtered_templates: Vec<ContractTemplate> = templates_list.read().iter().filter(|t| {
        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() { return true; }
        t.title.to_lowercase().contains(&q)
            || t.category.to_lowercase().contains(&q)
    }).cloned().collect();

    let qr_doc = qr_doc_id.read().as_ref().and_then(|did| {
        documents_list.read().iter().find(|d| d.id == *did).cloned()
    });

    let mut toast_del = toast.clone();
    let mut toast_cp1 = toast.clone();
    let mut toast_cp2 = toast.clone();
    let mut toast_tmpl = toast.clone();
    let mut reload_d = reload_trigger;

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "documents-page",
            if let Some(doc_id) = preview_doc_id() {
                {
                    let doc_opt = documents_list.read().iter().find(|d| d.id == doc_id).cloned();
                    let p_name = doc_opt.as_ref().and_then(|d| d.patient_name.clone()).unwrap_or_else(|| "Mariana Castro".to_string());
                    let d_name = doc_opt.as_ref().and_then(|d| d.doctor_user_name.clone()).unwrap_or_else(|| "Dr. Lucas Mendes - CRO 12345".to_string());
                    let p_sig = use_signal(move || p_name.clone());
                    let d_sig = use_signal(move || d_name.clone());

                    rsx! {
                        PaperPreview {
                            template_id: "contrato",
                            clinic_name,
                            patient_name: p_sig,
                            doctor_name: d_sig,
                            on_back: move |_| preview_doc_id.set(None),
                        }
                    }
                }
            } else {
                div { style: "display: flex; flex-direction: column; gap: 4px;",
                    h1 { style: "font-size: 22px; font-weight: 800; color: #f8fafc; margin: 0;", "Documentos, Contratos & Termos Clínicos" }
                    p { style: "font-size: 13.5px; color: #94a3b8; margin: 0 0 8px 0;",
                        "Gerencie contratos odontológicos, termos de consentimento (TCLE), atestados e prescrições com assinatura digital via QR Code."
                    }
                }

                DocumentsToolbar {
                    active_tab,
                    search_query,
                    status_filter,
                    on_issue_document: move |_| {
                        issue_patient_id.set(String::new());
                        issue_template_id.set(String::new());
                        issue_title.set(String::new());
                        is_issue_modal_open.set(true);
                    },
                    on_new_template: move |_| {
                        toast_tmpl.show("Criador de modelos de contrato aberto.", ToastVariant::Info);
                    },
                }

                if *active_tab.read() == DocumentTab::Issued {
                    IssuedDocumentsTable {
                        documents: filtered_documents,
                        on_preview: move |did| preview_doc_id.set(Some(did)),
                        on_qr_code: move |did| qr_doc_id.set(Some(did)),
                        on_copy_link: move |_did: String| {
                            toast_cp1.show("Link de assinatura digital copiado para a área de transferência!", ToastVariant::Success);
                        },
                        on_delete: move |did: String| {
                            let mut toast_resp = toast_del.clone();
                            let mut reload_c = reload_d;
                            spawn(async move {
                                if let Ok(_) = DocumentsApi::delete_document(&did).await {
                                    toast_resp.show("Documento excluído com sucesso.", ToastVariant::Success);
                                    reload_c.set(reload_c() + 1);
                                }
                            });
                        },
                    }
                } else {
                    TemplatesGrid {
                        templates: filtered_templates,
                        on_use_template: move |tid: String| {
                            issue_template_id.set(tid.clone());
                            if let Some(t) = templates_list.read().iter().find(|t| t.id == tid) {
                                issue_title.set(t.title.clone());
                            }
                            is_issue_modal_open.set(true);
                        },
                    }
                }

                IssueDocumentModal {
                    is_open: is_issue_modal_open(),
                    templates: templates_list(),
                    patients: patients_list(),
                    selected_patient_id: issue_patient_id,
                    selected_template_id: issue_template_id,
                    document_title: issue_title,
                    document_type: issue_type,
                    requires_patient_sig: req_pat_sig,
                    requires_doctor_sig: req_doc_sig,
                    is_already_signed,
                    on_close: move |_| is_issue_modal_open.set(false),
                    on_submit: handle_issue_submit,
                }

                if let Some(doc) = qr_doc {
                    QrCodeModal {
                        is_open: true,
                        document_title: doc.title,
                        signing_token: doc.signing_token,
                        on_close: move |_| qr_doc_id.set(None),
                        on_copied: move |_| {
                            toast_cp2.show("Link de assinatura copiado!", ToastVariant::Success);
                        },
                    }
                }
            }
        }
    }
}
