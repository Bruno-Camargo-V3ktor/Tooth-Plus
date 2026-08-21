//! # Aba de Histórico de Procedimentos e Tratamentos Odontológicos (Frontend)
//!
//! Controla os procedimentos realizados, planejados ou em andamento,
//! com registro de dente/região, superfícies, materiais, orientações pós-operatórias,
//! vínculos com agendamento/documento/exame, observações clínicas e edição completa.

use crate::api::{create_patient_treatment, delete_patient_treatment, update_patient_treatment};
use crate::components::icons::{IconEdit, IconTooth, IconTrash};
use dioxus::prelude::*;
use shared::patients::{
    CreatePatientTreatmentRequest, PatientTreatment, UpdatePatientTreatmentRequest,
};

/// Formata valor em centavos para moeda BRL.
fn format_currency(cents: i64) -> String {
    let reals = cents / 100;
    let centavos = cents % 100;
    format!("R$ {},{:02}", reals, centavos)
}

/// Componente da aba de procedimentos e tratamentos odontológicos.
#[component]
pub fn PatientOdontogramTab(
    patient_id: String,
    clinic_id: String,
    token: String,
    treatments: Vec<PatientTreatment>,
    can_write: bool,
    can_delete: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut is_add_modal_open = use_signal(|| false);
    let mut editing_treatment = use_signal(|| None::<PatientTreatment>);
    let mut delete_target_id = use_signal(|| None::<(String, String)>);
    let mut is_deleting = use_signal(|| false);

    // Form de Criação
    let mut form_category = use_signal(|| "Dentística".to_string());
    let mut form_procedure_name = use_signal(String::new);
    let mut form_tooth = use_signal(String::new);
    let mut form_surf_m = use_signal(|| false);
    let mut form_surf_d = use_signal(|| false);
    let mut form_surf_o = use_signal(|| false);
    let mut form_surf_v = use_signal(|| false);
    let mut form_surf_l = use_signal(|| false);
    let mut form_status = use_signal(|| "completed".to_string());
    let mut form_cost = use_signal(String::new);
    let mut form_materials = use_signal(String::new);
    let mut form_post_care = use_signal(String::new);
    let mut form_notes = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    // Form de Edição
    let mut edit_category = use_signal(|| "Dentística".to_string());
    let mut edit_procedure_name = use_signal(String::new);
    let mut edit_tooth = use_signal(String::new);
    let mut edit_surf_m = use_signal(|| false);
    let mut edit_surf_d = use_signal(|| false);
    let mut edit_surf_o = use_signal(|| false);
    let mut edit_surf_v = use_signal(|| false);
    let mut edit_surf_l = use_signal(|| false);
    let mut edit_status = use_signal(|| "completed".to_string());
    let mut edit_cost = use_signal(String::new);
    let mut edit_materials = use_signal(String::new);
    let mut edit_post_care = use_signal(String::new);
    let mut edit_notes = use_signal(String::new);
    let mut is_edit_submitting = use_signal(|| false);

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();
    let on_reload = reload_patient_details.clone();

    let mut handle_submit = move |_| {
        let proc_name = form_procedure_name().trim().to_string();
        if proc_name.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o nome do procedimento realizado.".into()));
            return;
        }

        let cost_clean = form_cost().trim().replace(',', ".").replace("R$", "").replace(' ', "");
        let cost_cents = if let Ok(val) = cost_clean.parse::<f64>() {
            (val * 100.0).round() as i64
        } else {
            0
        };

        let mut surfaces = Vec::new();
        if form_surf_m() { surfaces.push("Mesial (M)".to_string()); }
        if form_surf_d() { surfaces.push("Distal (D)".to_string()); }
        if form_surf_o() { surfaces.push("Oclusal/Incisal (O/I)".to_string()); }
        if form_surf_v() { surfaces.push("Vestibular (V)".to_string()); }
        if form_surf_l() { surfaces.push("Lingual/Palatina (L/P)".to_string()); }

        let materials_list = if form_materials().trim().is_empty() {
            None
        } else {
            Some(form_materials().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        };

        let req = CreatePatientTreatmentRequest {
            clinic_id: cid.clone(),
            dentist_user_id: None,
            appointment_id: None,
            document_id: None,
            exam_id: None,
            treatment_plan_id: None,
            transaction_id: None,
            procedure_category: Some(form_category()),
            procedure_name: proc_name,
            tooth_number: if form_tooth().trim().is_empty() { None } else { Some(form_tooth().trim().to_string()) },
            surfaces: if surfaces.is_empty() { None } else { Some(surfaces) },
            materials_used: materials_list,
            status: form_status(),
            cost_cents,
            post_care_instructions: if form_post_care().trim().is_empty() { None } else { Some(form_post_care().trim().to_string()) },
            clinical_notes: if form_notes().trim().is_empty() { None } else { Some(form_notes().trim().to_string()) },
            performed_at: None,
        };

        let t = tok.clone();
        let p = pat_id.clone();
        let mut open_sig = is_add_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_submitting;
        let reload = on_reload.clone();

        sub_sig.set(true);
        spawn(async move {
            match create_patient_treatment(&t, &p, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    toast.set(Some("Procedimento registrado com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao registrar procedimento: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    let tok_edit = token.clone();
    let pat_id_edit = patient_id.clone();
    let cid_edit = clinic_id.clone();
    let on_reload_edit = reload_patient_details.clone();

    let mut handle_edit_submit = move |_| {
        let Some(ref current_treat) = *editing_treatment.read() else { return; };
        let treat_id = current_treat.id.clone();
        let proc_name = edit_procedure_name().trim().to_string();
        if proc_name.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o nome do procedimento realizado.".into()));
            return;
        }

        let cost_clean = edit_cost().trim().replace(',', ".").replace("R$", "").replace(' ', "");
        let cost_cents = if let Ok(val) = cost_clean.parse::<f64>() {
            (val * 100.0).round() as i64
        } else {
            0
        };

        let mut surfaces = Vec::new();
        if edit_surf_m() { surfaces.push("Mesial (M)".to_string()); }
        if edit_surf_d() { surfaces.push("Distal (D)".to_string()); }
        if edit_surf_o() { surfaces.push("Oclusal/Incisal (O/I)".to_string()); }
        if edit_surf_v() { surfaces.push("Vestibular (V)".to_string()); }
        if edit_surf_l() { surfaces.push("Lingual/Palatina (L/P)".to_string()); }

        let materials_list = if edit_materials().trim().is_empty() {
            None
        } else {
            Some(edit_materials().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        };

        let req = UpdatePatientTreatmentRequest {
            clinic_id: cid_edit.clone(),
            dentist_user_id: None,
            appointment_id: None,
            document_id: None,
            exam_id: None,
            treatment_plan_id: current_treat.treatment_plan_id.clone(),
            transaction_id: current_treat.transaction_id.clone(),
            procedure_category: Some(edit_category()),
            procedure_name: proc_name,
            tooth_number: if edit_tooth().trim().is_empty() { None } else { Some(edit_tooth().trim().to_string()) },
            surfaces: if surfaces.is_empty() { None } else { Some(surfaces) },
            materials_used: materials_list,
            status: edit_status(),
            cost_cents,
            post_care_instructions: if edit_post_care().trim().is_empty() { None } else { Some(edit_post_care().trim().to_string()) },
            clinical_notes: if edit_notes().trim().is_empty() { None } else { Some(edit_notes().trim().to_string()) },
            performed_at: None,
        };

        let t = tok_edit.clone();
        let p = pat_id_edit.clone();
        let mut edit_modal_sig = editing_treatment;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_edit_submitting;
        let reload = on_reload_edit.clone();

        sub_sig.set(true);
        spawn(async move {
            match update_patient_treatment(&t, &p, &treat_id, req).await {
                Ok(_) => {
                    edit_modal_sig.set(None);
                    toast.set(Some("Procedimento atualizado com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao atualizar procedimento: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    let mut open_edit_modal = move |treat: PatientTreatment| {
        edit_category.set(treat.procedure_category.clone().unwrap_or_else(|| "Dentística".into()));
        edit_procedure_name.set(treat.procedure_name.clone());
        edit_tooth.set(treat.tooth_number.clone().unwrap_or_default());

        let surfs = treat.surfaces.clone().unwrap_or_default();
        edit_surf_m.set(surfs.iter().any(|s| s.contains("Mesial") || s == "M"));
        edit_surf_d.set(surfs.iter().any(|s| s.contains("Distal") || s == "D"));
        edit_surf_o.set(surfs.iter().any(|s| s.contains("Oclusal") || s == "O" || s == "I"));
        edit_surf_v.set(surfs.iter().any(|s| s.contains("Vestibular") || s == "V"));
        edit_surf_l.set(surfs.iter().any(|s| s.contains("Lingual") || s == "L" || s == "P"));

        edit_status.set(treat.status.clone());
        let reals = treat.cost_cents as f64 / 100.0;
        edit_cost.set(format!("{:.2}", reals).replace('.', ","));
        edit_materials.set(treat.materials_used.as_ref().map(|m| m.join(", ")).unwrap_or_default());
        edit_post_care.set(treat.post_care_instructions.clone().unwrap_or_default());
        edit_notes.set(treat.clinical_notes.clone().unwrap_or_default());
        editing_treatment.set(Some(treat));
    };

    let tok_del = token.clone();
    let cid_del = clinic_id.clone();
    let pat_id_del = patient_id.clone();
    let on_reload_del = reload_patient_details.clone();

    let mut handle_confirm_delete = move |_| {
        let Some((ref t_id, _)) = *delete_target_id.read() else { return; };
        let t_id_clone = t_id.clone();
        let t = tok_del.clone();
        let p = pat_id_del.clone();
        let c = cid_del.clone();
        let mut target_sig = delete_target_id;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut del_sig = is_deleting;
        let reload = on_reload_del.clone();

        del_sig.set(true);
        spawn(async move {
            match delete_patient_treatment(&t, &p, &t_id_clone, &c).await {
                Ok(_) => {
                    target_sig.set(None);
                    toast.set(Some("Procedimento removido do histórico.".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao remover procedimento: {}", e)));
                }
            }
            del_sig.set(false);
        });
    };

    rsx! {
        div { class: "patient-tab-content",
            div { class: "tab-header-actions-row",
                div { class: "tab-header-title-group",
                    h3 { class: "tab-header-title", "Histórico de Procedimentos e Evolução Clínica" }
                    p { class: "tab-header-desc", "Acompanhamento detalhado da evolução clínica, materiais empregados, superfícies e cuidados pós-operatórios." }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            form_procedure_name.set(String::new());
                            form_tooth.set(String::new());
                            form_cost.set(String::new());
                            form_materials.set(String::new());
                            form_post_care.set(String::new());
                            form_notes.set(String::new());
                            is_add_modal_open.set(true);
                        },
                        IconTooth { size: 16, color: "#ffffff".to_string() }
                        span { " Registrar Procedimento" }
                    }
                }
            }

            if treatments.is_empty() {
                div { class: "empty-state-card",
                    div { class: "empty-state-icon-box",
                        IconTooth { size: 32, color: "currentColor".to_string() }
                    }
                    h3 { "Nenhum procedimento registrado" }
                    p { "Registre restaurações, procedimentos cirúrgicos, manutenções ortodônticas ou evoluções clínicas." }
                }
            } else {
                div { class: "table-container",
                    table { class: "modern-table",
                        thead {
                            tr {
                                th { "DATA / HORA" }
                                th { "CATEGORIA" }
                                th { "PROCEDIMENTO" }
                                th { "DENTE / FACES" }
                                th { "STATUS" }
                                th { "VALOR" }
                                th { "DETALHES / CUIDADOS" }
                                if can_write || can_delete {
                                    th { class: "text-right", "AÇÕES" }
                                }
                            }
                        }
                        tbody {
                            for treat in &treatments {
                                {
                                    let dt = treat.performed_at.as_deref().unwrap_or(&treat.created_at).chars().take(10).collect::<String>();
                                    let cost_brl = format_currency(treat.cost_cents);
                                    let is_completed = treat.status == "completed";
                                    let is_in_progress = treat.status == "in_progress";
                                    let category_lbl = treat.procedure_category.as_deref().unwrap_or("Geral");
                                    let surfaces_str = treat.surfaces.as_ref().map(|s| s.join(", ")).unwrap_or_default();
                                    let treat_id = treat.id.clone();
                                    let treat_name = treat.procedure_name.clone();
                                    let treat_full = treat.clone();

                                    rsx! {
                                        tr { key: "{treat.id}",
                                            td { class: "font-mono font-xs", "{dt}" }
                                            td {
                                                span { class: "badge-insurance-plan", "{category_lbl}" }
                                            }
                                            td {
                                                p { style: "font-weight: 600; color: #1e293b; margin: 0;", "{treat.procedure_name}" }
                                                if let Some(ref mats) = treat.materials_used {
                                                    if !mats.is_empty() {
                                                        p { style: "font-size: 11px; color: #64748b; margin: 2px 0 0 0;", "Materiais: {mats.join(\", \")}" }
                                                    }
                                                }
                                            }
                                            td {
                                                if let Some(ref tooth) = treat.tooth_number {
                                                    span { class: "badge-outline", "Dente {tooth}" }
                                                } else {
                                                    span { class: "text-muted font-xs", "Região Geral" }
                                                }
                                                if !surfaces_str.is_empty() {
                                                    span { style: "display: block; font-size: 10px; color: #64748b; margin-top: 2px;", "Faces: {surfaces_str}" }
                                                }
                                            }
                                            td {
                                                if is_completed {
                                                    span { class: "badge-completed", "Concluído" }
                                                } else if is_in_progress {
                                                    span { class: "badge-pending", "Em Andamento" }
                                                } else {
                                                    span { class: "badge-outline", "Planejado" }
                                                }
                                            }
                                            td { class: "font-mono font-bold", "{cost_brl}" }
                                            td { style: "max-width: 250px;",
                                                if let Some(ref post) = treat.post_care_instructions {
                                                    p { style: "font-size: 11px; color: #0284c7; margin: 0 0 2px 0;", "Pós-op: {post}" }
                                                }
                                                p { class: "text-muted font-xs", style: "margin: 0;",
                                                    "{treat.clinical_notes.as_deref().unwrap_or(\"-\")}"
                                                }
                                            }
                                            if can_write || can_delete {
                                                td { class: "text-right", style: "white-space: nowrap;",
                                                    div { style: "display: inline-flex; align-items: center; justify-content: flex-end; gap: 8px;",
                                                        if can_write {
                                                            button {
                                                                class: "btn-secondary btn-sm",
                                                                style: "padding: 6px 8px; color: #0052cc; border-color: #bfdbfe; background: #eff6ff;",
                                                                title: "Editar procedimento",
                                                                onclick: {
                                                                    let t_edit = treat_full.clone();
                                                                    move |_| open_edit_modal(t_edit.clone())
                                                                },
                                                                IconEdit { size: 14, color: "#0052cc".to_string() }
                                                            }
                                                        }
                                                        if can_delete {
                                                            button {
                                                                class: "btn-delete-row-table",
                                                                title: "Remover procedimento",
                                                                onclick: move |_| delete_target_id.set(Some((treat_id.clone(), treat_name.clone()))),
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
                }
            }

            // Modal: Registrar Novo Procedimento
            if is_add_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal", style: "max-width: 680px;",
                        div { class: "settings-header",
                            div {
                                h2 { class: "settings-title", "Registrar Procedimento e Evolução Clínica" }
                                p { class: "text-muted font-xs mt-1",
                                    "Adicione procedimentos realizados ou planejados com materiais, faces e cuidados pós-operatórios."
                                }
                            }
                            button { class: "close-btn", onclick: move |_| is_add_modal_open.set(false), "×" }
                        }
                        div { class: "settings-content", style: "max-height: 65vh; overflow-y: auto;",
                            div { class: "form-grid-2",
                                div { class: "form-group",
                                    label { "Categoria do Procedimento *" }
                                    select {
                                        class: "form-input",
                                        value: "{form_category}",
                                        onchange: move |e| form_category.set(e.value()),
                                        option { value: "Dentística", "Dentística / Restauração" }
                                        option { value: "Endodontia", "Endodontia / Canal" }
                                        option { value: "Cirurgia", "Cirurgia / Extração" }
                                        option { value: "Periodontia", "Periodontia / Raspagem" }
                                        option { value: "Ortodontia", "Ortodontia / Alinhadores" }
                                        option { value: "Prótese", "Prótese / Reabilitação" }
                                        option { value: "Implantodontia", "Implantodontia" }
                                        option { value: "Profilaxia", "Profilaxia / Prevenção" }
                                        option { value: "Odontopediatria", "Odontopediatria" }
                                        option { value: "Outro", "Outro" }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Nome do Procedimento *" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Restauração em Resina Composta",
                                        value: "{form_procedure_name}",
                                        oninput: move |e| form_procedure_name.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Dente / Região" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: 16, 21, Arcada Superior",
                                        value: "{form_tooth}",
                                        oninput: move |e| form_tooth.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Status do Procedimento" }
                                    select {
                                        class: "form-input",
                                        value: "{form_status}",
                                        onchange: move |e| form_status.set(e.value()),
                                        option { value: "completed", "Concluído (Realizado)" }
                                        option { value: "in_progress", "Em Andamento (Sessão)" }
                                        option { value: "planned", "Planejado (Orçamento)" }
                                    }
                                }
                            }

                            // Faces / Superfícies Dentárias
                            div { class: "form-group", style: "margin-top: 10px;",
                                label { "Faces / Superfícies Tratadas" }
                                div { style: "display: flex; gap: 14px; flex-wrap: wrap; margin-top: 4px;",
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: form_surf_m(),
                                            onchange: move |e| form_surf_m.set(e.checked()),
                                        }
                                        span { "Mesial (M)" }
                                    }
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: form_surf_d(),
                                            onchange: move |e| form_surf_d.set(e.checked()),
                                        }
                                        span { "Distal (D)" }
                                    }
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: form_surf_o(),
                                            onchange: move |e| form_surf_o.set(e.checked()),
                                        }
                                        span { "Oclusal/Incisal (O/I)" }
                                    }
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: form_surf_v(),
                                            onchange: move |e| form_surf_v.set(e.checked()),
                                        }
                                        span { "Vestibular (V)" }
                                    }
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: form_surf_l(),
                                            onchange: move |e| form_surf_l.set(e.checked()),
                                        }
                                        span { "Lingual/Palatina (L/P)" }
                                    }
                                }
                            }

                            div { class: "form-grid-2", style: "margin-top: 10px;",
                                div { class: "form-group",
                                    label { "Valor (R$)" }
                                    div { class: "currency-input-wrapper",
                                        span { class: "currency-prefix", "R$" }
                                        input {
                                            class: "form-input currency-input-field",
                                            placeholder: "0,00",
                                            value: "{form_cost}",
                                            oninput: move |e| form_cost.set(e.value())
                                        }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Materiais Utilizados (separados por vírgula)" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Resina Z350 A2, Adesivo Universal, Ácido Fosfórico",
                                        value: "{form_materials}",
                                        oninput: move |e| form_materials.set(e.value())
                                    }
                                }
                            }

                            div { class: "form-group", style: "margin-top: 10px;",
                                label { "Orientações e Cuidados Pós-Operatórios" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: Evitar mastigar alimentos duros por 24h, higiene com escova macia...",
                                    value: "{form_post_care}",
                                    oninput: move |e| form_post_care.set(e.value())
                                }
                            }

                            div { class: "form-group", style: "margin-top: 10px;",
                                label { "Observações Clínicas e Evolução" }
                                textarea {
                                    class: "form-input",
                                    style: "min-height: 75px; resize: vertical;",
                                    placeholder: "Ex: Procedimento realizado sob anestesia infiltrativa, isolamento absoluto, paciente relatou conforto...",
                                    value: "{form_notes}",
                                    oninput: move |e| form_notes.set(e.value())
                                }
                            }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| is_add_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_submitting(),
                                onclick: move |e| handle_submit(e),
                                if is_submitting() { "Salvando..." } else { "Salvar Procedimento" }
                            }
                        }
                    }
                }
            }

            // Modal: Editar Procedimento e Evolução Clínica
            if editing_treatment().is_some() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal", style: "max-width: 680px;",
                        div { class: "settings-header",
                            div {
                                h2 { class: "settings-title", "Editar Procedimento e Evolução Clínica" }
                                p { class: "text-muted font-xs mt-1",
                                    "Atualize os detalhes do procedimento, superfícies tratadas, status ou valor."
                                }
                            }
                            button { class: "close-btn", onclick: move |_| editing_treatment.set(None), "×" }
                        }
                        div { class: "settings-content", style: "max-height: 65vh; overflow-y: auto;",
                            div { class: "form-grid-2",
                                div { class: "form-group",
                                    label { "Categoria do Procedimento *" }
                                    select {
                                        class: "form-input",
                                        value: "{edit_category}",
                                        onchange: move |e| edit_category.set(e.value()),
                                        option { value: "Dentística", "Dentística / Restauração" }
                                        option { value: "Endodontia", "Endodontia / Canal" }
                                        option { value: "Cirurgia", "Cirurgia / Extração" }
                                        option { value: "Periodontia", "Periodontia / Raspagem" }
                                        option { value: "Ortodontia", "Ortodontia / Alinhadores" }
                                        option { value: "Prótese", "Prótese / Reabilitação" }
                                        option { value: "Implantodontia", "Implantodontia" }
                                        option { value: "Profilaxia", "Profilaxia / Prevenção" }
                                        option { value: "Odontopediatria", "Odontopediatria" }
                                        option { value: "Outro", "Outro" }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Nome do Procedimento *" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Restauração em Resina Composta",
                                        value: "{edit_procedure_name}",
                                        oninput: move |e| edit_procedure_name.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Dente / Região" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: 16, 21, Arcada Superior",
                                        value: "{edit_tooth}",
                                        oninput: move |e| edit_tooth.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Status do Procedimento" }
                                    select {
                                        class: "form-input",
                                        value: "{edit_status}",
                                        onchange: move |e| edit_status.set(e.value()),
                                        option { value: "completed", "Concluído (Realizado)" }
                                        option { value: "in_progress", "Em Andamento (Sessão)" }
                                        option { value: "planned", "Planejado (Orçamento)" }
                                    }
                                }
                            }

                            // Faces / Superfícies Dentárias
                            div { class: "form-group", style: "margin-top: 10px;",
                                label { "Faces / Superfícies Tratadas" }
                                div { style: "display: flex; gap: 14px; flex-wrap: wrap; margin-top: 4px;",
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: edit_surf_m(),
                                            onchange: move |e| edit_surf_m.set(e.checked()),
                                        }
                                        span { "Mesial (M)" }
                                    }
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: edit_surf_d(),
                                            onchange: move |e| edit_surf_d.set(e.checked()),
                                        }
                                        span { "Distal (D)" }
                                    }
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: edit_surf_o(),
                                            onchange: move |e| edit_surf_o.set(e.checked()),
                                        }
                                        span { "Oclusal/Incisal (O/I)" }
                                    }
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: edit_surf_v(),
                                            onchange: move |e| edit_surf_v.set(e.checked()),
                                        }
                                        span { "Vestibular (V)" }
                                    }
                                    label { class: "anamnese-checkbox-label",
                                        input {
                                            r#type: "checkbox",
                                            checked: edit_surf_l(),
                                            onchange: move |e| edit_surf_l.set(e.checked()),
                                        }
                                        span { "Lingual/Palatina (L/P)" }
                                    }
                                }
                            }

                            div { class: "form-grid-2", style: "margin-top: 10px;",
                                div { class: "form-group",
                                    label { "Valor (R$)" }
                                    div { class: "currency-input-wrapper",
                                        span { class: "currency-prefix", "R$" }
                                        input {
                                            class: "form-input currency-input-field",
                                            placeholder: "0,00",
                                            value: "{edit_cost}",
                                            oninput: move |e| edit_cost.set(e.value())
                                        }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Materiais Utilizados (separados por vírgula)" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Resina Z350 A2, Adesivo Universal, Ácido Fosfórico",
                                        value: "{edit_materials}",
                                        oninput: move |e| edit_materials.set(e.value())
                                    }
                                }
                            }

                            div { class: "form-group", style: "margin-top: 10px;",
                                label { "Orientações e Cuidados Pós-Operatórios" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: Evitar mastigar alimentos duros por 24h, higiene com escova macia...",
                                    value: "{edit_post_care}",
                                    oninput: move |e| edit_post_care.set(e.value())
                                }
                            }

                            div { class: "form-group", style: "margin-top: 10px;",
                                label { "Observações Clínicas e Evolução" }
                                textarea {
                                    class: "form-input",
                                    style: "min-height: 75px; resize: vertical;",
                                    placeholder: "Ex: Procedimento realizado sob anestesia infiltrativa, isolamento absoluto, paciente relatou conforto...",
                                    value: "{edit_notes}",
                                    oninput: move |e| edit_notes.set(e.value())
                                }
                            }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| editing_treatment.set(None), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_edit_submitting(),
                                onclick: move |e| handle_edit_submit(e),
                                if is_edit_submitting() { "Salvando..." } else { "Salvar Alterações" }
                            }
                        }
                    }
                }
            }

            // Modal de Exclusão de Procedimento
            if let Some((_, ref t_name)) = *delete_target_id.read() {
                div { class: "modal-overlay",
                    div { class: "action-modal delete-modal-card",
                        div { class: "settings-header",
                            h2 { class: "settings-title text-danger", "Remover Procedimento" }
                            button { class: "close-btn", onclick: move |_| delete_target_id.set(None), "×" }
                        }
                        div { class: "settings-content",
                            p { "Tem certeza que deseja excluir o registro de ", strong { "{t_name}" }, " do prontuário?" }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| delete_target_id.set(None), "Cancelar" }
                            button {
                                class: "btn-danger",
                                disabled: is_deleting(),
                                onclick: move |e| handle_confirm_delete(e),
                                if is_deleting() { "Removendo..." } else { "Confirmar Remoção" }
                            }
                        }
                    }
                }
            }
        }
    }
}
