use crate::api::{
    create_patient, create_patient_document, create_patient_exam, create_patient_treatment,
    delete_patient, fetch_patient_details, fetch_patients, fetch_templates, save_patient_anamnesis,
};
use crate::components::icons::{
    IconCheckCircle, IconExternalLink, IconEye, IconFile, IconFolder, IconHeartPulse, IconLock,
    IconPhone, IconQrCode, IconRefresh, IconSearch, IconShieldCheck, IconSignature, IconTooth,
    IconTrash, IconUpload, IconUsers,
};
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use qrcode::QrCode;
use qrcode::render::svg;
use shared::documents::{ContractTemplate, CreatePatientDocumentRequest, PatientDocument};
use shared::patients::{
    CreatePatientExamRequest, CreatePatientRequest, CreatePatientTreatmentRequest, Patient,
    PatientDetailsResponse, PatientKpis, SaveAnamnesisRequest,
};

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
pub fn PatientsView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = permissions::has_permission(&sess, &clinic, "patients:read");
    let can_write = permissions::has_permission(&sess, &clinic, "patients:write");
    let can_delete = permissions::has_permission(&sess, &clinic, "patients:delete");

    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();
    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();
    let clinic_name = clinic
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_default();

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar os prontuários desta unidade." }
            }
        };
    }

    let mut patients_list = use_signal(Vec::<Patient>::new);
    let mut kpis = use_signal(PatientKpis::default);
    let mut is_loading = use_signal(|| true);
    let mut search_query = use_signal(String::new);
    let mut active_filter = use_signal(|| "all".to_string());
    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    // Dedicated Full-Page Patient View State
    let mut selected_patient_id = use_signal(|| None::<String>);
    let mut patient_details = use_signal(|| None::<PatientDetailsResponse>);
    let mut details_loading = use_signal(|| false);
    let mut active_patient_tab = use_signal(|| "overview".to_string());

    // Modals
    let mut is_create_patient_open = use_signal(|| false);
    let mut is_edit_patient_open = use_signal(|| false);
    let mut is_add_exam_open = use_signal(|| false);
    let mut is_add_treatment_open = use_signal(|| false);
    let mut is_emit_contract_open = use_signal(|| false);
    let mut qr_modal_doc = use_signal(|| None::<PatientDocument>);
    let mut pdf_preview_target = use_signal(|| None::<(String, String)>);

    // Contract templates list
    let mut templates_list = use_signal(Vec::<ContractTemplate>::new);

    // Form inputs: Patient Create/Edit
    let mut form_full_name = use_signal(String::new);
    let mut form_cpf = use_signal(String::new);
    let mut form_phone = use_signal(String::new);
    let mut form_email = use_signal(String::new);
    let mut form_birth_date = use_signal(String::new);
    let mut form_gender = use_signal(|| "Masculino".to_string());
    let mut form_marital_status = use_signal(|| "Solteiro(a)".to_string());
    let mut form_profession = use_signal(String::new);
    let mut form_em_name = use_signal(String::new);
    let mut form_em_phone = use_signal(String::new);
    let mut form_street = use_signal(String::new);
    let mut form_num = use_signal(String::new);
    let mut form_comp = use_signal(String::new);
    let mut form_neigh = use_signal(String::new);
    let mut form_city = use_signal(|| "São Paulo".to_string());
    let mut form_state = use_signal(|| "SP".to_string());
    let mut form_zip = use_signal(String::new);
    let mut form_insurance = use_signal(|| "Particular".to_string());
    let mut form_insurance_num = use_signal(String::new);
    let mut form_signature_pwd = use_signal(String::new);

    // Form inputs: Exam
    let mut exam_title = use_signal(String::new);
    let mut exam_type = use_signal(|| "radiography_panoramic".to_string());
    let mut exam_notes = use_signal(String::new);
    let mut exam_file_url = use_signal(String::new);

    // Form inputs: Treatment
    let mut treat_procedure = use_signal(String::new);
    let mut treat_tooth = use_signal(String::new);
    let mut treat_status = use_signal(|| "planned".to_string());
    let mut treat_cost_reais = use_signal(|| "0,00".to_string());
    let mut treat_notes = use_signal(String::new);

    // Form inputs: Emit Contract
    let mut emit_template_id = use_signal(String::new);
    let mut emit_doc_title = use_signal(String::new);
    let mut emit_doc_type = use_signal(|| "contract".to_string());
    let mut emit_static_pdf_url = use_signal(String::new);

    // Anamnesis inputs
    let mut anam_allergies_penicillin = use_signal(|| false);
    let mut anam_allergies_dipyrone = use_signal(|| false);
    let mut anam_allergies_latex = use_signal(|| false);
    let mut anam_allergies_anesthetic = use_signal(|| false);
    let mut anam_medications = use_signal(String::new);
    let mut anam_disease_diabetes = use_signal(|| false);
    let mut anam_disease_hypertension = use_signal(|| false);
    let mut anam_disease_cardiac = use_signal(|| false);
    let mut anam_is_pregnant = use_signal(|| false);
    let mut anam_bleeding_disorder = use_signal(|| false);
    let mut anam_smoker = use_signal(|| false);
    let mut anam_bruxism = use_signal(|| false);
    let mut anam_complaint = use_signal(String::new);
    let mut anam_clinical_notes = use_signal(String::new);

    let load_patients_data = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        move || {
            let t = token.clone();
            let cid = clinic_id.clone();
            let search = search_query();
            spawn(async move {
                is_loading.set(true);
                match fetch_patients(
                    &t,
                    &cid,
                    if search.is_empty() {
                        None
                    } else {
                        Some(&search)
                    },
                )
                .await
                {
                    Ok(resp) => {
                        patients_list.set(resp.items);
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

    let load_patient_details = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        move |pid: String| {
            let t = token.clone();
            let cid = clinic_id.clone();
            spawn(async move {
                details_loading.set(true);
                match fetch_patient_details(&t, &pid, &cid).await {
                    Ok(details) => {
                        if let Some(ref a) = details.anamnesis {
                            anam_allergies_penicillin
                                .set(a.allergies.contains(&"Penicilina".to_string()));
                            anam_allergies_dipyrone
                                .set(a.allergies.contains(&"Dipirona".to_string()));
                            anam_allergies_latex.set(a.allergies.contains(&"Látex".to_string()));
                            anam_allergies_anesthetic
                                .set(a.allergies.contains(&"Anestésico Local".to_string()));
                            anam_medications
                                .set(a.continuous_medications.clone().unwrap_or_default());
                            anam_disease_diabetes
                                .set(a.systemic_diseases.contains(&"Diabetes".to_string()));
                            anam_disease_hypertension
                                .set(a.systemic_diseases.contains(&"Hipertensão".to_string()));
                            anam_disease_cardiac
                                .set(a.systemic_diseases.contains(&"Cardiopatia".to_string()));
                            anam_is_pregnant.set(a.is_pregnant);
                            anam_bleeding_disorder.set(a.has_bleeding_disorder);
                            anam_smoker.set(a.smoker);
                            anam_bruxism.set(a.bruxism);
                            anam_complaint.set(a.chief_complaint.clone().unwrap_or_default());
                            anam_clinical_notes.set(a.clinical_notes.clone().unwrap_or_default());
                        } else {
                            anam_allergies_penicillin.set(false);
                            anam_allergies_dipyrone.set(false);
                            anam_allergies_latex.set(false);
                            anam_allergies_anesthetic.set(false);
                            anam_medications.set(String::new());
                            anam_disease_diabetes.set(false);
                            anam_disease_hypertension.set(false);
                            anam_disease_cardiac.set(false);
                            anam_is_pregnant.set(false);
                            anam_bleeding_disorder.set(false);
                            anam_smoker.set(false);
                            anam_bruxism.set(false);
                            anam_complaint.set(String::new());
                            anam_clinical_notes.set(String::new());
                        }

                        patient_details.set(Some(details));
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
                details_loading.set(false);
            });
        }
    };

    let load_templates = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        move || {
            let t = token.clone();
            let cid = clinic_id.clone();
            spawn(async move {
                if let Ok(tpls) = fetch_templates(&t, &cid).await {
                    templates_list.set(tpls);
                }
            });
        }
    };

    use_effect({
        let lp = load_patients_data.clone();
        let lt = load_templates.clone();
        move || {
            lp();
            lt();
        }
    });

    let open_create_modal = move |_| {
        form_full_name.set(String::new());
        form_cpf.set(String::new());
        form_phone.set(String::new());
        form_email.set(String::new());
        form_birth_date.set(String::new());
        form_gender.set("Masculino".to_string());
        form_marital_status.set("Solteiro(a)".to_string());
        form_profession.set(String::new());
        form_em_name.set(String::new());
        form_em_phone.set(String::new());
        form_street.set(String::new());
        form_num.set(String::new());
        form_comp.set(String::new());
        form_neigh.set(String::new());
        form_city.set("São Paulo".to_string());
        form_state.set("SP".to_string());
        form_zip.set(String::new());
        form_insurance.set("Particular".to_string());
        form_insurance_num.set(String::new());
        form_signature_pwd.set(String::new());
        is_create_patient_open.set(true);
    };

    let on_submit_create_patient = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        let lp = load_patients_data.clone();
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let lp_call = lp.clone();

            let req = CreatePatientRequest {
                clinic_id: cid,
                full_name: form_full_name(),
                document_cpf: form_cpf(),
                phone: form_phone(),
                email: if form_email().is_empty() {
                    None
                } else {
                    Some(form_email())
                },
                birth_date: if form_birth_date().is_empty() {
                    None
                } else {
                    Some(form_birth_date())
                },
                gender: Some(form_gender()),
                marital_status: Some(form_marital_status()),
                profession: if form_profession().is_empty() {
                    None
                } else {
                    Some(form_profession())
                },
                emergency_contact_name: if form_em_name().is_empty() {
                    None
                } else {
                    Some(form_em_name())
                },
                emergency_contact_phone: if form_em_phone().is_empty() {
                    None
                } else {
                    Some(form_em_phone())
                },
                address_street: if form_street().is_empty() {
                    None
                } else {
                    Some(form_street())
                },
                address_number: if form_num().is_empty() {
                    None
                } else {
                    Some(form_num())
                },
                address_complement: if form_comp().is_empty() {
                    None
                } else {
                    Some(form_comp())
                },
                address_neighborhood: if form_neigh().is_empty() {
                    None
                } else {
                    Some(form_neigh())
                },
                address_city: if form_city().is_empty() {
                    None
                } else {
                    Some(form_city())
                },
                address_state: if form_state().is_empty() {
                    None
                } else {
                    Some(form_state())
                },
                address_zip: if form_zip().is_empty() {
                    None
                } else {
                    Some(form_zip())
                },
                insurance_plan: Some(form_insurance()),
                insurance_number: if form_insurance_num().is_empty() {
                    None
                } else {
                    Some(form_insurance_num())
                },
                signature_password: if form_signature_pwd().is_empty() {
                    None
                } else {
                    Some(form_signature_pwd())
                },
            };

            spawn(async move {
                match create_patient(&t, req).await {
                    Ok(p) => {
                        toast_msg.set(Some(format!(
                            "Paciente {} cadastrado com sucesso!",
                            p.full_name
                        )));
                        is_create_patient_open.set(false);
                        lp_call();
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
            });
        }
    };

    let on_submit_anamnesis = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        let lpd = load_patient_details.clone();
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let lpd_call = lpd.clone();
            let pid = selected_patient_id().unwrap_or_default();

            let mut allergies = Vec::new();
            if anam_allergies_penicillin() {
                allergies.push("Penicilina".to_string());
            }
            if anam_allergies_dipyrone() {
                allergies.push("Dipirona".to_string());
            }
            if anam_allergies_latex() {
                allergies.push("Látex".to_string());
            }
            if anam_allergies_anesthetic() {
                allergies.push("Anestésico Local".to_string());
            }

            let mut systemic = Vec::new();
            if anam_disease_diabetes() {
                systemic.push("Diabetes".to_string());
            }
            if anam_disease_hypertension() {
                systemic.push("Hipertensão".to_string());
            }
            if anam_disease_cardiac() {
                systemic.push("Cardiopatia".to_string());
            }

            let req = SaveAnamnesisRequest {
                clinic_id: cid,
                allergies,
                continuous_medications: if anam_medications().is_empty() {
                    None
                } else {
                    Some(anam_medications())
                },
                systemic_diseases: systemic,
                is_pregnant: anam_is_pregnant(),
                has_bleeding_disorder: anam_bleeding_disorder(),
                smoker: anam_smoker(),
                bruxism: anam_bruxism(),
                chief_complaint: if anam_complaint().is_empty() {
                    None
                } else {
                    Some(anam_complaint())
                },
                clinical_notes: if anam_clinical_notes().is_empty() {
                    None
                } else {
                    Some(anam_clinical_notes())
                },
            };

            spawn(async move {
                match save_patient_anamnesis(&t, &pid, req).await {
                    Ok(_) => {
                        toast_msg.set(Some("Ficha médica de anamnese salva com sucesso!".into()));
                        lpd_call(pid);
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
            });
        }
    };

    let on_submit_exam = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        let lpd = load_patient_details.clone();
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let lpd_call = lpd.clone();
            let pid = selected_patient_id().unwrap_or_default();

            let files = if exam_file_url().is_empty() {
                vec![
                    "https://placehold.co/600x400/1e293b/ffffff?text=Exame+Radiologico".to_string(),
                ]
            } else {
                vec![exam_file_url()]
            };

            let req = CreatePatientExamRequest {
                clinic_id: cid,
                title: exam_title(),
                exam_type: exam_type(),
                requested_date: None,
                result_date: None,
                file_urls: files,
                clinical_interpretation: if exam_notes().is_empty() {
                    None
                } else {
                    Some(exam_notes())
                },
            };

            spawn(async move {
                match create_patient_exam(&t, &pid, req).await {
                    Ok(_) => {
                        toast_msg.set(Some("Exame registrado com sucesso!".into()));
                        is_add_exam_open.set(false);
                        lpd_call(pid);
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
            });
        }
    };

    let on_submit_treatment = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        let lpd = load_patient_details.clone();
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let lpd_call = lpd.clone();
            let pid = selected_patient_id().unwrap_or_default();

            let cost_cents = (treat_cost_reais()
                .replace('.', "")
                .replace(',', ".")
                .parse::<f64>()
                .unwrap_or(0.0)
                * 100.0) as i64;

            let req = CreatePatientTreatmentRequest {
                clinic_id: cid,
                dentist_user_id: None,
                appointment_id: None,
                procedure_name: treat_procedure(),
                tooth_number: if treat_tooth().is_empty() {
                    None
                } else {
                    Some(treat_tooth())
                },
                status: treat_status(),
                cost_cents,
                clinical_notes: if treat_notes().is_empty() {
                    None
                } else {
                    Some(treat_notes())
                },
            };

            spawn(async move {
                match create_patient_treatment(&t, &pid, req).await {
                    Ok(_) => {
                        toast_msg.set(Some("Procedimento clínico registrado!".into()));
                        is_add_treatment_open.set(false);
                        lpd_call(pid);
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
            });
        }
    };

    let on_submit_emit_contract = {
        let token = token.clone();
        let clinic_id = clinic_id.clone();
        let lpd = load_patient_details.clone();
        let lp = load_patients_data.clone();
        move |_| {
            let t = token.clone();
            let cid = clinic_id.clone();
            let lpd_call = lpd.clone();
            let lp_call = lp.clone();
            let pid = selected_patient_id().unwrap_or_default();

            let tpl_id = if emit_template_id().is_empty() {
                None
            } else {
                Some(emit_template_id())
            };
            let pdf = if emit_static_pdf_url().is_empty() {
                None
            } else {
                Some(emit_static_pdf_url())
            };

            let req = CreatePatientDocumentRequest {
                clinic_id: cid,
                patient_id: pid.clone(),
                template_id: tpl_id,
                doctor_user_id: None,
                appointment_id: None,
                title: emit_doc_title(),
                document_type: emit_doc_type(),
                pdf_url: pdf,
            };

            spawn(async move {
                match create_patient_document(&t, req).await {
                    Ok(doc) => {
                        toast_msg.set(Some("Documento emitido com sucesso!".into()));
                        is_emit_contract_open.set(false);
                        qr_modal_doc.set(Some(doc));
                        lpd_call(pid);
                        lp_call();
                    }
                    Err(e) => {
                        error_toast.set(Some(e));
                    }
                }
            });
        }
    };

    let ld_input = load_patients_data.clone();
    let ld_refresh = load_patients_data.clone();

    rsx! {
        div { class: "patients-view-container",
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

            if let Some(ref pid) = selected_patient_id() {
                // =========================================================================
                // FULL DEDICATED PATIENT PROFILE / PRONTUÁRIO SCREEN
                // =========================================================================
                if details_loading() {
                    div { class: "patient-loading-card",
                        div { class: "loading-spinner" }
                        p { "Carregando prontuário completo do paciente..." }
                    }
                } else if let Some(det) = patient_details() {
                    div { class: "patient-details-page",
                        // Top Breadcrumb & Actions Bar
                        div { class: "patient-top-actions",
                            button {
                                class: "btn-back-link",
                                onclick: move |_| {
                                    selected_patient_id.set(None);
                                    patient_details.set(None);
                                },
                                "← Voltar para Lista de Pacientes"
                            }
                            div { class: "patient-page-quick-actions",
                                if can_write {
                                    button {
                                        class: "btn-action-primary",
                                        onclick: {
                                            let pname = det.patient.full_name.clone();
                                            move |_| {
                                                emit_doc_title.set(format!("Contrato de Prestação de Serviços - {}", pname));
                                                emit_template_id.set(String::new());
                                                emit_doc_type.set("contract".to_string());
                                                emit_static_pdf_url.set(String::new());
                                                is_emit_contract_open.set(true);
                                            }
                                        },
                                        IconSignature { size: 16, color: "#ffffff".to_string() }
                                        " Emitir Contrato / Termo"
                                    }
                                }
                            }
                        }

                        // Patient Hero Profile Banner
                        div { class: "patient-profile-banner",
                            div { class: "patient-avatar-large",
                                "{det.patient.full_name.chars().next().unwrap_or('P')}"
                            }
                            div { class: "patient-hero-info",
                                div { class: "patient-hero-title-row",
                                    h1 { class: "patient-hero-name", "{det.patient.full_name}" }
                                    span { class: "badge-insurance", "{det.patient.insurance_plan.as_deref().unwrap_or(\"Particular\")}" }
                                    if let Some(ref a) = det.anamnesis {
                                        if !a.allergies.is_empty() {
                                            span { class: "badge-allergy-alert",
                                                IconHeartPulse { size: 14, color: "#ef4444".to_string() }
                                                " Alerta: Alergias Cadastradas"
                                            }
                                        }
                                    }
                                }
                                div { class: "patient-hero-chips",
                                    span { class: "hero-chip",
                                        IconShieldCheck { size: 14, color: "#0052cc".to_string() }
                                        " CPF: {det.patient.document_cpf}"
                                    }
                                    span { class: "hero-chip", "📞 {det.patient.phone}" }
                                    if let Some(ref em) = det.patient.email {
                                        span { class: "hero-chip", "✉️ {em}" }
                                    }
                                    if let Some(ref b) = det.patient.birth_date {
                                        span { class: "hero-chip", "🎂 Nasc: {crate::utils::format_date_br(b)}" }
                                    }
                                }
                            }
                        }

                        // Patient Profile Tabs Navigation
                        div { class: "patient-tabs-nav",
                            button {
                                class: if active_patient_tab() == "overview" { "patient-tab-item active" } else { "patient-tab-item" },
                                onclick: move |_| active_patient_tab.set("overview".to_string()),
                                IconUsers { size: 16, color: "currentColor".to_string() }
                                " Visão Geral"
                            }
                            button {
                                class: if active_patient_tab() == "anamnesis" { "patient-tab-item active" } else { "patient-tab-item" },
                                onclick: move |_| active_patient_tab.set("anamnesis".to_string()),
                                IconHeartPulse { size: 16, color: "currentColor".to_string() }
                                " Anamnese & Ficha Médica"
                            }
                            button {
                                class: if active_patient_tab() == "exams" { "patient-tab-item active" } else { "patient-tab-item" },
                                onclick: move |_| active_patient_tab.set("exams".to_string()),
                                IconEye { size: 16, color: "currentColor".to_string() }
                                " Exames & Laudos ({det.exams.len()})"
                            }
                            button {
                                class: if active_patient_tab() == "treatments" { "patient-tab-item active" } else { "patient-tab-item" },
                                onclick: move |_| active_patient_tab.set("treatments".to_string()),
                                IconTooth { size: 16, color: "currentColor".to_string() }
                                " Histórico de Tratamentos ({det.treatments.len()})"
                            }
                            button {
                                class: if active_patient_tab() == "documents" { "patient-tab-item active" } else { "patient-tab-item" },
                                onclick: move |_| active_patient_tab.set("documents".to_string()),
                                IconSignature { size: 16, color: "currentColor".to_string() }
                                " Contratos & Documentos ({det.documents.len()})"
                            }
                        }

                        // Tab 1: Visão Geral
                        if active_patient_tab() == "overview" {
                            div { class: "patient-tab-pane",
                                div { class: "overview-grid",
                                    div { class: "overview-card",
                                        h3 { class: "overview-card-title", "Dados Cadastrais e Contatos" }
                                        div { class: "overview-field-list",
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Nome Completo" }
                                                span { class: "field-val", "{det.patient.full_name}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "CPF (Protegido por Criptografia)" }
                                                span { class: "field-val", "{det.patient.document_cpf}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Telefone / WhatsApp" }
                                                span { class: "field-val", "{det.patient.phone}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "E-mail" }
                                                span { class: "field-val", "{det.patient.email.as_deref().unwrap_or(\"Não informado\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Sexo / Estado Civil" }
                                                span { class: "field-val", "{det.patient.gender.as_deref().unwrap_or(\"-\")} / {det.patient.marital_status.as_deref().unwrap_or(\"-\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Profissão" }
                                                span { class: "field-val", "{det.patient.profession.as_deref().unwrap_or(\"Não informada\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Contato de Emergência" }
                                                span { class: "field-val", "{det.patient.emergency_contact_name.as_deref().unwrap_or(\"-\")} ({det.patient.emergency_contact_phone.as_deref().unwrap_or(\"-\")})" }
                                            }
                                        }
                                    }

                                    div { class: "overview-card",
                                        h3 { class: "overview-card-title", "Endereço e Convênio" }
                                        div { class: "overview-field-list",
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Logradouro" }
                                                span { class: "field-val", "{det.patient.address_street.as_deref().unwrap_or(\"-\")}, {det.patient.address_number.as_deref().unwrap_or(\"S/N\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Complemento / Bairro" }
                                                span { class: "field-val", "{det.patient.address_complement.as_deref().unwrap_or(\"-\")} - {det.patient.address_neighborhood.as_deref().unwrap_or(\"-\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Cidade / UF" }
                                                span { class: "field-val", "{det.patient.address_city.as_deref().unwrap_or(\"São Paulo\")} - {det.patient.address_state.as_deref().unwrap_or(\"SP\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "CEP" }
                                                span { class: "field-val", "{det.patient.address_zip.as_deref().unwrap_or(\"-\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Plano / Convênio" }
                                                span { class: "field-val", "{det.patient.insurance_plan.as_deref().unwrap_or(\"Particular\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Nº da Carteirinha" }
                                                span { class: "field-val", "{det.patient.insurance_number.as_deref().unwrap_or(\"-\")}" }
                                            }
                                            div { class: "overview-field-item",
                                                span { class: "field-label", "Senha de Assinatura Digital" }
                                                span { class: "field-val", if det.patient.has_signature_password { "✓ Cadastrada e Ativa" } else { "⚠️ Não definida" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Tab 2: Anamnese Odontológica
                        if active_patient_tab() == "anamnesis" {
                            div { class: "patient-tab-pane",
                                div { class: "anamnesis-form-card",
                                    div { class: "anamnesis-section",
                                        h3 { class: "anamnesis-section-title", "1. Alergias Conhecidas" }
                                        div { class: "checkbox-grid",
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_allergies_penicillin(),
                                                    onchange: move |e| anam_allergies_penicillin.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Penicilina / Antibióticos"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_allergies_dipyrone(),
                                                    onchange: move |e| anam_allergies_dipyrone.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Dipirona / Anti-inflamatórios"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_allergies_latex(),
                                                    onchange: move |e| anam_allergies_latex.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Látex"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_allergies_anesthetic(),
                                                    onchange: move |e| anam_allergies_anesthetic.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Anestésicos Locais"
                                            }
                                        }
                                    }

                                    div { class: "anamnesis-section",
                                        h3 { class: "anamnesis-section-title", "2. Doenças Sistêmicas & Condições Especiais" }
                                        div { class: "checkbox-grid",
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_disease_diabetes(),
                                                    onchange: move |e| anam_disease_diabetes.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Diabetes"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_disease_hypertension(),
                                                    onchange: move |e| anam_disease_hypertension.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Hipertensão Arterial"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_disease_cardiac(),
                                                    onchange: move |e| anam_disease_cardiac.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Cardiopatia"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_bleeding_disorder(),
                                                    onchange: move |e| anam_bleeding_disorder.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Distúrbio Hemorrágico / Sangramento"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_is_pregnant(),
                                                    onchange: move |e| anam_is_pregnant.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Gestante"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_smoker(),
                                                    onchange: move |e| anam_smoker.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Fumante"
                                            }
                                            label { class: "checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: anam_bruxism(),
                                                    onchange: move |e| anam_bruxism.set(e.value().parse().unwrap_or(false)),
                                                }
                                                "Bruxismo / Apertamento Dental"
                                            }
                                        }
                                    }

                                    div { class: "anamnesis-section",
                                        h3 { class: "anamnesis-section-title", "3. Medicamentos de Uso Contínuo" }
                                        input {
                                            r#type: "text",
                                            class: "input-field",
                                            placeholder: "Ex: Losartana 50mg, Metformina 850mg, Anticoagulantes...",
                                            value: "{anam_medications}",
                                            oninput: move |e| anam_medications.set(e.value()),
                                        }
                                    }

                                    div { class: "anamnesis-section",
                                        h3 { class: "anamnesis-section-title", "4. Queixa Principal e Observações Clínicas" }
                                        div { class: "form-group",
                                            label { class: "form-label", "Queixa Principal relatada pelo Paciente" }
                                            textarea {
                                                class: "textarea-field",
                                                placeholder: "Descreva a dor, desconforto ou motivo da consulta...",
                                                value: "{anam_complaint}",
                                                oninput: move |e| anam_complaint.set(e.value()),
                                            }
                                        }
                                        div { class: "form-group",
                                            label { class: "form-label", "Notas Clínicas Privadas do Cirurgião-Dentista" }
                                            textarea {
                                                class: "textarea-field",
                                                placeholder: "Anotações adicionais e histórico odontológico prévio...",
                                                value: "{anam_clinical_notes}",
                                                oninput: move |e| anam_clinical_notes.set(e.value()),
                                            }
                                        }
                                    }

                                    if can_write {
                                        div { class: "anamnesis-actions",
                                            button {
                                                class: "btn-primary",
                                                onclick: on_submit_anamnesis,
                                                IconCheckCircle { size: 18, color: "#ffffff".to_string() }
                                                " Salvar Ficha de Anamnese"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Tab 3: Exames & Laudos
                        if active_patient_tab() == "exams" {
                            div { class: "patient-tab-pane",
                                div { class: "tab-header-row",
                                    div {
                                        h3 { class: "tab-pane-title", "Galeria de Exames e Radiografias" }
                                        p { class: "tab-pane-subtitle", "Radiografias panorâmicas, periapicais, tomografias e fotos intraorais." }
                                    }
                                    if can_write {
                                        button {
                                            class: "btn-primary",
                                            onclick: move |_| {
                                                exam_title.set(String::new());
                                                exam_notes.set(String::new());
                                                exam_file_url.set(String::new());
                                                is_add_exam_open.set(true);
                                            },
                                            IconUpload { size: 16, color: "#ffffff".to_string() }
                                            " Novo Exame / Laudo"
                                        }
                                    }
                                }

                                if det.exams.is_empty() {
                                    div { class: "empty-state-card",
                                        IconEye { size: 40, color: "#94a3b8".to_string() }
                                        p { "Nenhum exame cadastrado para este paciente." }
                                    }
                                } else {
                                    div { class: "exams-gallery-grid",
                                        for exam in det.exams.iter() {
                                            div { class: "exam-card",
                                                div { class: "exam-preview-thumb",
                                                    if !exam.file_urls.is_empty() {
                                                        img { src: "{exam.file_urls[0]}", alt: "Exame", class: "exam-img" }
                                                    } else {
                                                        div { class: "exam-no-thumb", IconEye { size: 32, color: "#94a3b8".to_string() } }
                                                    }
                                                }
                                                div { class: "exam-content",
                                                    div { class: "exam-header",
                                                        h4 { class: "exam-title", "{exam.title}" }
                                                        span { class: "exam-type-badge", "{exam.exam_type}" }
                                                    }
                                                    p { class: "exam-date", "Data da Solicitação: {crate::utils::format_date_br(&exam.requested_date)}" }
                                                    if let Some(ref note) = exam.clinical_interpretation {
                                                        p { class: "exam-notes", "Laudo: {note}" }
                                                    }
                                                    if !exam.file_urls.is_empty() {
                                                        a {
                                                            href: "{exam.file_urls[0]}",
                                                            target: "_blank",
                                                            class: "btn-view-exam",
                                                            IconExternalLink { size: 14, color: "#0052cc".to_string() }
                                                            " Abrir Arquivo Completo"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Tab 4: Histórico de Tratamentos & Evolução Clínica
                        if active_patient_tab() == "treatments" {
                            div { class: "patient-tab-pane",
                                div { class: "tab-header-row",
                                    div {
                                        h3 { class: "tab-pane-title", "Histórico de Procedimentos e Evolução" }
                                        p { class: "tab-pane-subtitle", "Registro detalhado de intervenções clínicas odontológicas." }
                                    }
                                    if can_write {
                                        button {
                                            class: "btn-primary",
                                            onclick: move |_| {
                                                treat_procedure.set(String::new());
                                                treat_tooth.set(String::new());
                                                treat_notes.set(String::new());
                                                treat_cost_reais.set("0,00".to_string());
                                                is_add_treatment_open.set(true);
                                            },
                                            IconTooth { size: 16, color: "#ffffff".to_string() }
                                            " Registrar Procedimento"
                                        }
                                    }
                                }

                                if det.treatments.is_empty() {
                                    div { class: "empty-state-card",
                                        IconTooth { size: 40, color: "#94a3b8".to_string() }
                                        p { "Nenhum procedimento registrado no histórico." }
                                    }
                                } else {
                                    div { class: "treatments-timeline",
                                        for t in det.treatments.iter() {
                                            div { class: "timeline-item",
                                                div { class: "timeline-dot" }
                                                div { class: "timeline-card",
                                                    div { class: "timeline-card-header",
                                                        div {
                                                            h4 { class: "timeline-procedure", "{t.procedure_name}" }
                                                            if let Some(ref tooth) = t.tooth_number {
                                                                span { class: "tooth-badge", "Dente / Região: {tooth}" }
                                                            }
                                                        }
                                                        div { class: "timeline-badges",
                                                            span { class: if t.status == "completed" { "status-badge-ok" } else { "status-badge-progress" },
                                                                if t.status == "completed" { "Concluído" } else if t.status == "in_progress" { "Em Andamento" } else { "Planejado" }
                                                            }
                                                            span { class: "cost-badge", "{crate::utils::format_currency_br(t.cost_cents)}" }
                                                        }
                                                    }
                                                    if let Some(ref note) = t.clinical_notes {
                                                        p { class: "timeline-notes", "{note}" }
                                                    }
                                                    p { class: "timeline-date", "Registrado em: {format_br_date(&t.created_at)}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Tab 5: Documentos & Contratos do Paciente
                        if active_patient_tab() == "documents" {
                            div { class: "patient-tab-pane",
                                div { class: "tab-header-row",
                                    div {
                                        h3 { class: "tab-pane-title", "Documentos & Contratos Digitais" }
                                        p { class: "tab-pane-subtitle", "Termos assinados com validação de integridade e link para assinatura móvel." }
                                    }
                                    if can_write {
                                        button {
                                            class: "btn-primary",
                                            onclick: move |_| {
                                                emit_doc_title.set(format!("Contrato Odontológico - {}", det.patient.full_name));
                                                emit_template_id.set(String::new());
                                                emit_doc_type.set("contract".to_string());
                                                emit_static_pdf_url.set(String::new());
                                                is_emit_contract_open.set(true);
                                            },
                                            IconSignature { size: 16, color: "#ffffff".to_string() }
                                            " Emitir Contrato / Termo"
                                        }
                                    }
                                }

                                if det.documents.is_empty() {
                                    div { class: "empty-state-card",
                                        IconSignature { size: 40, color: "#94a3b8".to_string() }
                                        p { "Nenhum documento emitido para este paciente." }
                                    }
                                } else {
                                    div { class: "patient-docs-table-wrapper",
                                        table { class: "modern-table",
                                            thead {
                                                tr {
                                                    th { "Título do Documento" }
                                                    th { "Tipo" }
                                                    th { "Data de Emissão" }
                                                    th { "Status de Assinatura" }
                                                    th { class: "text-right", "Ações e QR Code" }
                                                }
                                            }
                                            tbody {
                                                for doc in det.documents.iter() {
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
                                                            if doc.status == "signed" || doc.status == "completed" {
                                                                span { class: "badge-status-completed",
                                                                    IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                                    " Assinado"
                                                                }
                                                            } else {
                                                                span { class: "badge-status-pending",
                                                                    IconSignature { size: 14, color: "#f59e0b".to_string() }
                                                                    " Pendente de Assinatura"
                                                                }
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
            } else {
                // =========================================================================
                // MAIN PATIENTS LIST VIEW
                // =========================================================================
                div { class: "patients-list-view",
                    // KPIs Row
                    div { class: "kpi-grid",
                        div { class: "kpi-card",
                            div { class: "kpi-icon-wrap bg-blue-light",
                                IconUsers { size: 24, color: "#0052cc".to_string() }
                            }
                            div { class: "kpi-content",
                                span { class: "kpi-label", "Total de Pacientes" }
                                h3 { class: "kpi-value", "{kpis().total_patients}" }
                            }
                        }
                        div { class: "kpi-card",
                            div { class: "kpi-icon-wrap bg-emerald-light",
                                IconCheckCircle { size: 24, color: "#10b981".to_string() }
                            }
                            div { class: "kpi-content",
                                span { class: "kpi-label", "Novos no Mês" }
                                h3 { class: "kpi-value", "{kpis().new_this_month}" }
                            }
                        }
                        div { class: "kpi-card",
                            div { class: "kpi-icon-wrap bg-amber-light",
                                IconSignature { size: 24, color: "#f59e0b".to_string() }
                            }
                            div { class: "kpi-content",
                                span { class: "kpi-label", "Docs. Pendentes de Assinatura" }
                                h3 { class: "kpi-value", "{kpis().pending_documents_count}" }
                            }
                        }
                        div { class: "kpi-card",
                            div { class: "kpi-icon-wrap bg-purple-light",
                                IconTooth { size: 24, color: "#8b5cf6".to_string() }
                            }
                            div { class: "kpi-content",
                                span { class: "kpi-label", "Em Tratamento Ativo" }
                                h3 { class: "kpi-value", "{kpis().active_treatments_count}" }
                            }
                        }
                    }

                    // Toolbar
                    div { class: "view-toolbar",
                        div { class: "search-input-wrap",
                            IconSearch { size: 18, color: "#94a3b8".to_string() }
                            input {
                                r#type: "text",
                                class: "search-input",
                                placeholder: "Buscar paciente por nome, CPF ou telefone...",
                                value: "{search_query}",
                                oninput: move |e| {
                                    search_query.set(e.value());
                                    ld_input();
                                },
                            }
                        }

                        div { class: "toolbar-actions",
                            button {
                                class: "btn-refresh",
                                title: "Recarregar Lista",
                                onclick: move |_| ld_refresh(),
                                IconRefresh { size: 16, color: "#475569".to_string() }
                            }
                            if can_write {
                                button {
                                    class: "btn-primary",
                                    onclick: open_create_modal,
                                    IconUsers { size: 16, color: "#ffffff".to_string() }
                                    " Novo Paciente"
                                }
                            }
                        }
                    }

                    // Quick Filter Pills
                    div { class: "patient-filter-pills-row",
                        button {
                            class: if active_filter() == "all" { "filter-pill active" } else { "filter-pill" },
                            onclick: move |_| active_filter.set("all".to_string()),
                            "Todos os Pacientes ({patients_list().len()})"
                        }
                        button {
                            class: if active_filter() == "particular" { "filter-pill active" } else { "filter-pill" },
                            onclick: move |_| active_filter.set("particular".to_string()),
                            "Particular"
                        }
                        button {
                            class: if active_filter() == "insurance" { "filter-pill active" } else { "filter-pill" },
                            onclick: move |_| active_filter.set("insurance".to_string()),
                            "Com Convênio"
                        }
                    }

                    // Patients Table
                    if is_loading() {
                        div { class: "loading-card",
                            div { class: "loading-spinner" }
                            p { "Carregando pacientes..." }
                        }
                    } else if patients_list().is_empty() {
                        div { class: "empty-state-card",
                            IconUsers { size: 48, color: "#94a3b8".to_string() }
                            h3 { "Nenhum paciente cadastrado" }
                            p { "Cadastre pacientes para gerenciar prontuários, exames e termos de consentimento." }
                        }
                    } else {
                        div { class: "table-container",
                            table { class: "modern-table",
                                thead {
                                    tr {
                                        th { "Paciente" }
                                        th { "CPF (Protegido)" }
                                        th { "Telefone / WhatsApp" }
                                        th { "Plano / Convênio" }
                                        th { "Cadastro" }
                                        th { class: "text-right", "Ações" }
                                    }
                                }
                                tbody {
                                    for p in patients_list().iter().filter(|pat| {
                                        let filter = active_filter();
                                        if filter == "particular" {
                                            pat.insurance_plan.as_deref().unwrap_or("Particular").to_lowercase().contains("particular")
                                        } else if filter == "insurance" {
                                            let ins = pat.insurance_plan.as_deref().unwrap_or("Particular").to_lowercase();
                                            !ins.contains("particular") && !ins.is_empty()
                                        } else {
                                            true
                                        }
                                    }) {
                                        {
                                            let clean_phone = p.phone.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
                                            let wa_url = format!("https://wa.me/55{}", clean_phone);
                                            let is_particular = p.insurance_plan.as_deref().unwrap_or("Particular").to_lowercase().contains("particular");
                                            let first_char = p.full_name.chars().next().unwrap_or('P');
                                            let avatar_bg = match first_char.to_ascii_uppercase() {
                                                'A'..='E' => "linear-gradient(135deg, #0052cc 0%, #2563eb 100%)",
                                                'F'..='J' => "linear-gradient(135deg, #059669 0%, #10b981 100%)",
                                                'K'..='O' => "linear-gradient(135deg, #7c3aed 0%, #8b5cf6 100%)",
                                                'P'..='T' => "linear-gradient(135deg, #d97706 0%, #f59e0b 100%)",
                                                _ => "linear-gradient(135deg, #0284c7 0%, #38bdf8 100%)",
                                            };

                                            rsx! {
                                                tr {
                                                    td {
                                                        div { class: "patient-cell-info",
                                                            div {
                                                                class: "patient-avatar-circle",
                                                                style: "background: {avatar_bg};",
                                                                "{first_char}"
                                                            }
                                                            div {
                                                                p { class: "patient-name-text", "{p.full_name}" }
                                                                if let Some(ref em) = p.email {
                                                                    span { class: "patient-email-sub", "{em}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    td {
                                                        span { class: "cpf-badge",
                                                            IconLock { size: 12, color: "#64748b".to_string() }
                                                            " {p.document_cpf}"
                                                        }
                                                    }
                                                    td {
                                                        div { class: "patient-phone-cell",
                                                            span { "{p.phone}" }
                                                            if !clean_phone.is_empty() {
                                                                a {
                                                                    href: "{wa_url}",
                                                                    target: "_blank",
                                                                    class: "btn-wa-link",
                                                                    title: "Abrir WhatsApp",
                                                                    IconPhone { size: 13, color: "#16a34a".to_string() }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    td {
                                                        if is_particular {
                                                            span { class: "badge-insurance-particular", "Particular" }
                                                        } else {
                                                            span { class: "badge-insurance-plan", "{p.insurance_plan.as_deref().unwrap_or(\"Convênio\")}" }
                                                        }
                                                    }
                                                    td { "{format_br_date(&p.created_at)}" }
                                                    td { class: "text-right",
                                                        div { class: "table-actions-row",
                                                            button {
                                                                class: "btn-open-prontuario",
                                                                onclick: {
                                                                    let pid = p.id.clone();
                                                                    let lpd = load_patient_details.clone();
                                                                    move |_| {
                                                                        selected_patient_id.set(Some(pid.clone()));
                                                                        lpd(pid.clone());
                                                                    }
                                                                },
                                                                IconFolder { size: 14, color: "#ffffff".to_string() }
                                                                " Prontuário"
                                                            }
                                                            if can_delete {
                                                                button {
                                                                    class: "btn-action-icon text-danger",
                                                                    title: "Excluir Paciente",
                                                                    onclick: {
                                                                        let pid = p.id.clone();
                                                                        let t = token.clone();
                                                                        let cid = clinic_id.clone();
                                                                        let lp = load_patients_data.clone();
                                                                        move |_| {
                                                                            let t_call = t.clone();
                                                                            let cid_call = cid.clone();
                                                                            let pid_call = pid.clone();
                                                                            let lp_call = lp.clone();
                                                                            spawn(async move {
                                                                                if delete_patient(&t_call, &pid_call, &cid_call).await.is_ok() {
                                                                                    toast_msg.set(Some("Paciente excluído com sucesso.".into()));
                                                                                    lp_call();
                                                                                }
                                                                            });
                                                                        }
                                                                    },
                                                                    IconTrash { size: 15, color: "#ef4444".to_string() }
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
                }
            }

            // =========================================================================
            // MODAL: CADASTRAR PACIENTE
            // =========================================================================
            if is_create_patient_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Cadastrar Novo Paciente" }
                                p { class: "modal-subtitle", "Preencha os dados do paciente. O CPF será armazenado com criptografia determinística." }
                            }
                            button { class: "modal-close", onclick: move |_| is_create_patient_open.set(false), "×" }
                        }

                        div { class: "modal-body",
                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Nome Completo *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "Nome do paciente",
                                        value: "{form_full_name}",
                                        oninput: move |e| form_full_name.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "CPF *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "000.000.000-00",
                                        value: "{form_cpf}",
                                        oninput: move |e| form_cpf.set(e.value()),
                                    }
                                }
                            }

                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "WhatsApp / Celular *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "(11) 90000-0000",
                                        value: "{form_phone}",
                                        oninput: move |e| form_phone.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "E-mail" }
                                    input {
                                        r#type: "email",
                                        class: "input-field",
                                        placeholder: "paciente@email.com",
                                        value: "{form_email}",
                                        oninput: move |e| form_email.set(e.value()),
                                    }
                                }
                            }

                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Data de Nascimento" }
                                    input {
                                        r#type: "date",
                                        class: "input-field",
                                        value: "{form_birth_date}",
                                        oninput: move |e| form_birth_date.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Sexo" }
                                    select {
                                        class: "select-field",
                                        value: "{form_gender}",
                                        onchange: move |e| form_gender.set(e.value()),
                                        option { value: "Masculino", "Masculino" }
                                        option { value: "Feminino", "Feminino" }
                                        option { value: "Outro", "Outro" }
                                    }
                                }
                            }

                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Plano / Convênio" }
                                    select {
                                        class: "select-field",
                                        value: "{form_insurance}",
                                        onchange: move |e| form_insurance.set(e.value()),
                                        option { value: "Particular", "Particular" }
                                        option { value: "Unimed", "Unimed Odonto" }
                                        option { value: "Amil", "Amil Dental" }
                                        option { value: "Bradesco", "Bradesco Dental" }
                                        option { value: "SulAmerica", "SulAmérica" }
                                        option { value: "Porto", "Porto Seguro" }
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Senha de Assinatura Digital do Paciente" }
                                    input {
                                        r#type: "password",
                                        class: "input-field",
                                        placeholder: "Defina uma senha para o paciente assinar documentos",
                                        value: "{form_signature_pwd}",
                                        oninput: move |e| form_signature_pwd.set(e.value()),
                                    }
                                }
                            }

                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Endereço (Rua/Av)" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "Ex: Av. Paulista",
                                        value: "{form_street}",
                                        oninput: move |e| form_street.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Número / Complemento" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "Ex: 1000, Apto 42",
                                        value: "{form_num}",
                                        oninput: move |e| form_num.set(e.value()),
                                    }
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_create_patient_open.set(false), "Cancelar" }
                            button { class: "btn-primary", onclick: on_submit_create_patient, "Salvar Paciente" }
                        }
                    }
                }
            }

            // =========================================================================
            // MODAL: EMITIR CONTRATO / ASSINATURA DIGITAL
            // =========================================================================
            if is_emit_contract_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Emitir Contrato / Termo de Assinatura" }
                                p { class: "modal-subtitle", "Vincule um modelo de contrato para assinatura digital imediata ou anexe um documento pronto." }
                            }
                            button { class: "modal-close", onclick: move |_| is_emit_contract_open.set(false), "×" }
                        }

                        div { class: "modal-body",
                            if let Some(ref det) = patient_details() {
                                div { class: "patient-autofill-card",
                                    div { class: "patient-autofill-item",
                                        span { class: "patient-autofill-label", "Paciente" }
                                        span { class: "patient-autofill-val", "{det.patient.full_name}" }
                                    }
                                    div { class: "patient-autofill-item",
                                        span { class: "patient-autofill-label", "CPF Protegido" }
                                        span { class: "patient-autofill-val", "{det.patient.document_cpf}" }
                                    }
                                    div { class: "patient-autofill-item",
                                        span { class: "patient-autofill-label", "WhatsApp" }
                                        span { class: "patient-autofill-val", "{det.patient.phone}" }
                                    }
                                    div { class: "patient-autofill-item",
                                        span { class: "patient-autofill-label", "Convênio" }
                                        span { class: "patient-autofill-val", "{det.patient.insurance_plan.as_deref().unwrap_or(\"Particular\")}" }
                                    }
                                }
                            }

                            div { class: "form-group",
                                label { class: "form-label", "Título do Documento *" }
                                input {
                                    r#type: "text",
                                    class: "input-field",
                                    placeholder: "Ex: Contrato de Tratamento Ortodôntico",
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
                                        option { value: "contract", "Contrato Odontológico (E-Sign)" }
                                        option { value: "consent", "Termo de Consentimento TCLE" }
                                        option { value: "budget", "Orçamento Aprovado" }
                                        option { value: "static_upload", "Upload de Documento Já Assinado (Estático)" }
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Modelo Base de Contrato" }
                                    select {
                                        class: "select-field",
                                        value: "{emit_template_id}",
                                        onchange: move |e| {
                                            let val = e.value();
                                            emit_template_id.set(val.clone());
                                            if let Some(t) = templates_list().iter().find(|t| t.id == val) {
                                                if let Some(ref det) = patient_details() {
                                                    emit_doc_title.set(format!("{} - {}", t.title, det.patient.full_name));
                                                } else {
                                                    emit_doc_title.set(t.title.clone());
                                                }
                                            }
                                        },
                                        option { value: "", "Documento em Branco / Padrão" }
                                        for tpl in templates_list().iter() {
                                            option { value: "{tpl.id}", "{tpl.title}" }
                                        }
                                    }
                                }
                            }

                            div { class: "form-group",
                                label { class: "form-label", "URL do PDF do Documento (ou Anexo)" }
                                input {
                                    r#type: "text",
                                    class: "input-field",
                                    placeholder: "https://... (ou deixe vazio para usar o PDF do modelo)",
                                    value: "{emit_static_pdf_url}",
                                    oninput: move |e| emit_static_pdf_url.set(e.value()),
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_emit_contract_open.set(false), "Cancelar" }
                            button { class: "btn-primary", onclick: on_submit_emit_contract, "Emitir e Gerar QR Code de Assinatura" }
                        }
                    }
                }
            }

            // =========================================================================
            // MODAL: QR CODE E LINK DE ASSINATURA PÚBLICA (SVG OFFLINE)
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
                    onclick: move |_| pdf_preview_target.set(None),
                    div { class: "action-modal pdf-viewer-modal",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "{title}" }
                                p { class: "modal-subtitle", "Documento PDF" }
                            }
                            button {
                                class: "modal-close",
                                onclick: move |_| pdf_preview_target.set(None),
                                "×"
                            }
                        }
                        div { class: "modal-body",
                            div { class: "pdf-desktop-viewer",
                                div { class: "pdf-desktop-icon",
                                    IconFile { size: 56, color: "#0052cc".to_string() }
                                }
                                p { class: "pdf-desktop-title", "{title}" }
                                p { class: "pdf-desktop-hint",
                                    "Clique abaixo para abrir o documento no navegador."
                                }
                                a {
                                    href: "{url}",
                                    target: "_blank",
                                    class: "btn-primary pdf-open-btn",
                                    IconExternalLink { size: 16, color: "white".to_string() }
                                    " Abrir PDF no Navegador"
                                }
                            }
                        }
                    }
                }
            }

            // =========================================================================
            // MODAL: NOVO EXAME
            // =========================================================================
            if is_add_exam_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Registrar Exame / Anexo" }
                                p { class: "modal-subtitle", "Adicione radiografias, tomografias ou laudos ao prontuário." }
                            }
                            button { class: "modal-close", onclick: move |_| is_add_exam_open.set(false), "×" }
                        }

                        div { class: "modal-body",
                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Título do Exame *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "Ex: Radiografia Panorâmica Inicial",
                                        value: "{exam_title}",
                                        oninput: move |e| exam_title.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Tipo de Exame" }
                                    select {
                                        class: "select-field",
                                        value: "{exam_type}",
                                        onchange: move |e| exam_type.set(e.value()),
                                        option { value: "radiography_panoramic", "Radiografia Panorâmica" }
                                        option { value: "radiography_periapical", "Radiografia Periapical" }
                                        option { value: "tomography", "Tomografia Cone Beam" }
                                        option { value: "intraoral_photo", "Foto Intraoral" }
                                        option { value: "lab_report", "Laudo Laboratorial" }
                                        option { value: "other", "Outro" }
                                    }
                                }
                            }

                            div { class: "form-group",
                                label { class: "form-label", "URL da Imagem / Arquivo" }
                                input {
                                    r#type: "text",
                                    class: "input-field",
                                    placeholder: "https://placehold.co/... ou link do arquivo",
                                    value: "{exam_file_url}",
                                    oninput: move |e| exam_file_url.set(e.value()),
                                }
                            }

                            div { class: "form-group",
                                label { class: "form-label", "Interpretação e Laudo Clínico" }
                                textarea {
                                    class: "textarea-field",
                                    placeholder: "Observações diagnósticas observadas no exame...",
                                    value: "{exam_notes}",
                                    oninput: move |e| exam_notes.set(e.value()),
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_add_exam_open.set(false), "Cancelar" }
                            button { class: "btn-primary", onclick: on_submit_exam, "Salvar Exame" }
                        }
                    }
                }
            }

            // =========================================================================
            // MODAL: NOVO PROCEDIMENTO
            // =========================================================================
            if is_add_treatment_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Registrar Procedimento Odontológico" }
                                p { class: "modal-subtitle", "Adicione a evolução clínica e procedimentos executados ou planejados." }
                            }
                            button { class: "modal-close", onclick: move |_| is_add_treatment_open.set(false), "×" }
                        }

                        div { class: "modal-body",
                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Procedimento *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "Ex: Restauração Resina Fotopolimerizável",
                                        value: "{treat_procedure}",
                                        oninput: move |e| treat_procedure.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Dente / Região" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "Ex: 16 (MOD), Arcada Superior...",
                                        value: "{treat_tooth}",
                                        oninput: move |e| treat_tooth.set(e.value()),
                                    }
                                }
                            }

                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Status" }
                                    select {
                                        class: "select-field",
                                        value: "{treat_status}",
                                        onchange: move |e| treat_status.set(e.value()),
                                        option { value: "planned", "Planejado" }
                                        option { value: "in_progress", "Em Andamento" }
                                        option { value: "completed", "Concluído" }
                                    }
                                }
                                div { class: "form-group",
                                    label { class: "form-label", "Valor do Procedimento (R$)" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        placeholder: "250,00",
                                        value: "{treat_cost_reais}",
                                        oninput: move |e| treat_cost_reais.set(e.value()),
                                    }
                                }
                            }

                            div { class: "form-group",
                                label { class: "form-label", "Anotações Clínicas" }
                                textarea {
                                    class: "textarea-field",
                                    placeholder: "Detalhes do material utilizado, anestesia, resposta do paciente...",
                                    value: "{treat_notes}",
                                    oninput: move |e| treat_notes.set(e.value()),
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_add_treatment_open.set(false), "Cancelar" }
                            button { class: "btn-primary", onclick: on_submit_treatment, "Salvar Procedimento" }
                        }
                    }
                }
            }
        }
    }
}
