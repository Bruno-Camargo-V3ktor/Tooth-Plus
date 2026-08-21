//! # Aba de Procedimentos Clínicos Realizados e Evolução do Paciente (Frontend)
//!
//! Controla os procedimentos individuais no Prontuário do paciente com seus status
//! clínicos oficiais (Pendente, Agendado, Em Consulta, Concluído), filtros clínicos e financeiros (Pago/Não Pago),
//! carregamento de modelos do catálogo de tratamentos ou cadastro avulso com todos os campos clínicos,
//! herança de status financeiro do orçamento, agendamento completo e baixa automática no estoque.

use crate::api::{
    create_appointment, create_patient_treatment, create_transaction, delete_patient_treatment,
    fetch_agenda_resources, fetch_stock_data, fetch_treatment_templates, update_patient_treatment,
};
use crate::components::icons::{
    IconBox, IconCalendar, IconCheck, IconClock, IconEdit, IconFinance, IconPlus, IconSearch,
    IconTool, IconTooth, IconTrash,
};
use dioxus::prelude::*;
use shared::appointments::{
    AgendaResourcesResponse, AppointmentType, AssignedUserDto, CreateAppointmentRequest,
};
use shared::finance::{CreateTransactionRequest, TransactionDirection, TransactionStatus};
use shared::patients::{
    CreatePatientTreatmentRequest, PatientTreatment, UpdatePatientTreatmentRequest,
};
use shared::stock::{InventoryItem, ItemType};
use shared::treatments::TreatmentTemplate;

/// Item de material com controle explícito de quantidade e unidade.
#[derive(Clone, PartialEq, Debug)]
pub struct SelectedMaterialItem {
    pub name: String,
    pub quantity: i32,
    pub unit: String,
}

fn parse_stored_material(raw: &str) -> SelectedMaterialItem {
    let raw = raw.trim();
    if let Some(rest) = raw.strip_prefix(|c: char| c.is_ascii_digit()) {
        let parts: Vec<&str> = raw.splitn(2, "x ").collect();
        if parts.len() == 2 {
            if let Ok(qty) = parts[0].trim().parse::<i32>() {
                let name_and_unit = parts[1].trim();
                if name_and_unit.ends_with(')') && name_and_unit.contains('(') {
                    if let Some(open_idx) = name_and_unit.rfind('(') {
                        let name = name_and_unit[..open_idx].trim().to_string();
                        let unit = name_and_unit[open_idx + 1..name_and_unit.len() - 1].trim().to_string();
                        return SelectedMaterialItem { name, quantity: qty.max(1), unit };
                    }
                }
                return SelectedMaterialItem {
                    name: name_and_unit.to_string(),
                    quantity: qty.max(1),
                    unit: "unidade".to_string(),
                };
            }
        }
    }

    SelectedMaterialItem {
        name: raw.to_string(),
        quantity: 1,
        unit: "unidade".to_string(),
    }
}

fn format_currency(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reals = abs_cents / 100;
    let centavos = abs_cents % 100;

    let reals_str = reals.to_string();
    let mut formatted_reals = String::new();
    let len = reals_str.len();
    for (i, ch) in reals_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted_reals.push('.');
        }
        formatted_reals.push(ch);
    }

    if is_negative {
        format!("- R$ {},{:02}", formatted_reals, centavos)
    } else {
        format!("R$ {},{:02}", formatted_reals, centavos)
    }
}

fn financial_status_badge(status: Option<&str>) -> (&'static str, &'static str) {
    match status {
        Some("paid") => ("badge-completed", "Pago"),
        Some("partial") => ("badge-active", "Parcial"),
        _ => ("badge-danger", "Não Pago"),
    }
}

#[component]
pub fn PatientTreatmentsTab(
    patient_id: String,
    patient_name: Option<String>,
    clinic_id: String,
    token: String,
    treatments: Vec<PatientTreatment>,
    can_write: bool,
    can_delete: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "all".to_string());

    let categories = vec![
        "Dentística",
        "Cirurgia",
        "Endodontia",
        "Ortodontia",
        "Periodontia",
        "Prótese",
        "Estética",
        "Implantodontia",
        "Odontopediatria",
        "Geral",
    ];

    // Modal state for single clinical procedure record (Complete and Rich like TreatmentTemplateModal)
    let mut is_proc_modal_open = use_signal(|| false);
    let mut editing_proc = use_signal(|| None::<PatientTreatment>);
    let mut selected_template_id = use_signal(String::new);
    let mut proc_name = use_signal(String::new);
    let mut proc_category = use_signal(|| "Dentística".to_string());
    let mut proc_price_str = use_signal(|| "0,00".to_string());
    let mut proc_status = use_signal(|| "pending".to_string());
    
    // Regiões e Dentes
    let mut selected_regions = use_signal(|| Vec::<String>::new());
    let mut selected_teeth = use_signal(|| Vec::<String>::new());
    let mut custom_tooth_input = use_signal(String::new);
    let mut proc_surfaces = use_signal(String::new);

    // Materiais e Insumos do Estoque
    let mut selected_materials = use_signal(|| Vec::<SelectedMaterialItem>::new());
    let mut chosen_stock_mat_idx = use_signal(|| "".to_string());
    let mut chosen_mat_qty = use_signal(|| 1i32);
    let mut custom_mat_name = use_signal(String::new);
    let mut custom_mat_unit = use_signal(|| "unidade".to_string());

    // Equipamentos
    let mut selected_equipment = use_signal(|| Vec::<String>::new());
    let mut custom_equipment_input = use_signal(String::new);

    // Pós-atendimento e Notas
    let mut proc_post_care = use_signal(String::new);
    let mut proc_notes = use_signal(String::new);
    let mut is_proc_saving = use_signal(|| false);

    // Modal state for scheduling a procedure
    let mut is_sched_modal_open = use_signal(|| false);
    let mut sched_target_proc = use_signal(|| None::<PatientTreatment>);
    let mut sched_title = use_signal(String::new);
    let mut sched_date = use_signal(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let mut sched_time = use_signal(|| "09:00".to_string());
    let mut sched_duration = use_signal(|| 45i32);
    let mut sched_dentist_id = use_signal(String::new);
    let mut sched_split = use_signal(|| 100i32);
    let mut sched_notes = use_signal(String::new);
    let mut is_scheduling = use_signal(|| false);

    // Modal state for financial charge on manual procedure
    let mut is_charge_modal_open = use_signal(|| false);
    let mut charge_target_proc = use_signal(|| None::<PatientTreatment>);
    let mut charge_amount_str = use_signal(String::new);
    let mut charge_is_paid = use_signal(|| true);
    let mut charge_method = use_signal(|| "Pix".to_string());
    let mut charge_notes = use_signal(String::new);
    let mut is_charging = use_signal(|| false);

    // Delete confirmation state
    let mut delete_proc_target = use_signal(|| None::<PatientTreatment>);
    let mut is_delete_proc_modal_open = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    // Fetch agenda resources (dentists/team members) for scheduling
    let tok_res = token.clone();
    let cid_res = clinic_id.clone();
    let resources_res = use_resource(move || {
        let t = tok_res.clone();
        let c = cid_res.clone();
        async move {
            fetch_agenda_resources(&t, &c).await.unwrap_or(AgendaResourcesResponse {
                team_members: vec![],
                patients: vec![],
                inventory_items: vec![],
                equipment_items: vec![],
                pending_treatments: vec![],
            })
        }
    });

    let agenda_resources = resources_res.read().clone().unwrap_or(AgendaResourcesResponse {
        team_members: vec![],
        patients: vec![],
        inventory_items: vec![],
        equipment_items: vec![],
        pending_treatments: vec![],
    });

    // Fetch treatment templates for procedure creation modal
    let tok_tpl = token.clone();
    let cid_tpl = clinic_id.clone();
    let templates_res = use_resource(move || {
        let t = tok_tpl.clone();
        let c = cid_tpl.clone();
        async move { fetch_treatment_templates(&t, &c).await.unwrap_or_default() }
    });

    let templates_list: Vec<TreatmentTemplate> = templates_res.read().clone().unwrap_or_default();

    // Fetch stock data for materials and equipments
    let tok_stock = token.clone();
    let cid_stock = clinic_id.clone();
    let stock_res = use_resource(move || {
        let t = tok_stock.clone();
        let c = cid_stock.clone();
        async move {
            if t.is_empty() || c.is_empty() {
                return None;
            }
            fetch_stock_data(&t, &c, None, None).await.ok()
        }
    });

    let (stock_materials, stock_equipments) = match &*stock_res.read() {
        Some(Some(data)) => {
            let mats: Vec<InventoryItem> = data
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::Material || i.item_type == ItemType::Chemical)
                .cloned()
                .collect();
            let eqs: Vec<InventoryItem> = data
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::Equipment)
                .cloned()
                .collect();
            (mats, eqs)
        }
        _ => (vec![], vec![]),
    };

    // KPIs dos Procedimentos
    let total_treatments = treatments.len();
    let total_cost_cents: i64 = treatments.iter().map(|t| t.cost_cents).sum();
    let pending_count = treatments
        .iter()
        .filter(|t| t.status == "pending" || t.status == "planned")
        .count();
    let scheduled_count = treatments.iter().filter(|t| t.status == "scheduled").count();
    let completed_count = treatments
        .iter()
        .filter(|t| t.status == "completed" || t.status == "done")
        .count();

    // Filtro dos procedimentos realizados (Clínico e Financeiro)
    let q = search_query().trim().to_lowercase();
    let filtered_treatments: Vec<PatientTreatment> = treatments
        .iter()
        .filter(|t| {
            let filter = status_filter();
            let matches_status = match filter.as_str() {
                "pending" => t.status == "pending" || t.status == "planned",
                "scheduled" => t.status == "scheduled",
                "in_consultation" => t.status == "in_consultation" || t.status == "in_progress",
                "completed" => t.status == "completed" || t.status == "done",
                "paid" => t.financial_status.as_deref() == Some("paid"),
                "unpaid" => t.financial_status.as_deref() != Some("paid"),
                _ => true,
            };

            let matches_search = if q.is_empty() {
                true
            } else {
                t.procedure_name.to_lowercase().contains(&q)
                    || t.procedure_category.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || t.tooth_number.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || t.clinical_notes.as_deref().unwrap_or("").to_lowercase().contains(&q)
            };

            matches_status && matches_search
        })
        .cloned()
        .collect();

    // Handler de Salvamento / Edição de Procedimento
    let tok_for_save = token.clone();
    let pid_for_save = patient_id.clone();
    let cid_for_save = clinic_id.clone();
    let on_reload_for_save = reload_patient_details.clone();

    let handle_save_proc = move |_| {
        let name = proc_name().trim().to_string();
        if name.is_empty() {
            error_toast.set(Some("Informe o nome do procedimento.".into()));
            return;
        }

        let raw_clean = proc_price_str().replace("R$", "").replace(' ', "").trim().to_string();
        let normalized = if raw_clean.contains(',') && raw_clean.contains('.') {
            raw_clean.replace('.', "").replace(',', ".")
        } else if raw_clean.contains(',') {
            raw_clean.replace(',', ".")
        } else {
            raw_clean
        };

        let price_float: f64 = normalized.parse().unwrap_or(0.0);
        let cost_cents = (price_float * 100.0).round() as i64;

        // Combina dentes selecionados ou input
        let mut all_teeth = selected_teeth();
        if !custom_tooth_input().trim().is_empty() {
            for piece in custom_tooth_input().split(',') {
                let clean = piece.trim().to_string();
                if !clean.is_empty() && !all_teeth.contains(&clean) {
                    all_teeth.push(clean);
                }
            }
        }
        let tooth_opt = if !all_teeth.is_empty() {
            Some(all_teeth.join(", "))
        } else if !selected_regions().is_empty() {
            Some(selected_regions().join(", "))
        } else {
            None
        };

        let surfaces_opt = if proc_surfaces().trim().is_empty() {
            None
        } else {
            let surfs: Vec<String> = proc_surfaces()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if surfs.is_empty() { None } else { Some(surfs) }
        };

        let formatted_materials: Vec<String> = selected_materials()
            .into_iter()
            .map(|m| format!("{}x {} ({})", m.quantity, m.name, m.unit))
            .collect();
        let materials_opt = if formatted_materials.is_empty() { None } else { Some(formatted_materials) };

        let post_care_opt = if proc_post_care().trim().is_empty() {
            None
        } else {
            Some(proc_post_care().trim().to_string())
        };

        let notes_opt = if proc_notes().trim().is_empty() {
            None
        } else {
            Some(proc_notes().trim().to_string())
        };

        let t = tok_for_save.clone();
        let pid = pid_for_save.clone();
        let cid = cid_for_save.clone();
        let on_r = on_reload_for_save.clone();
        let editing = editing_proc();
        let status = proc_status();
        let category = proc_category();

        is_proc_saving.set(true);
        spawn(async move {
            let res = if let Some(ref edit) = editing {
                let req = UpdatePatientTreatmentRequest {
                    clinic_id: cid,
                    dentist_user_id: edit.dentist_user_id.clone(),
                    appointment_id: edit.appointment_id.clone(),
                    document_id: edit.document_id.clone(),
                    exam_id: edit.exam_id.clone(),
                    treatment_plan_id: edit.treatment_plan_id.clone(),
                    treatment_plan_item_id: edit.treatment_plan_item_id.clone(),
                    transaction_id: edit.transaction_id.clone(),
                    financial_status: edit.financial_status.clone(),
                    procedure_category: Some(category),
                    procedure_name: name,
                    tooth_number: tooth_opt,
                    surfaces: surfaces_opt.or_else(|| edit.surfaces.clone()),
                    materials_used: materials_opt.or_else(|| edit.materials_used.clone()),
                    status,
                    cost_cents,
                    post_care_instructions: post_care_opt.or_else(|| edit.post_care_instructions.clone()),
                    clinical_notes: notes_opt,
                    performed_at: edit.appointment_date.clone(),
                };
                update_patient_treatment(&t, &pid, &edit.id, req).await.map(|_| ())
            } else {
                let req = CreatePatientTreatmentRequest {
                    clinic_id: cid,
                    dentist_user_id: None,
                    appointment_id: None,
                    document_id: None,
                    exam_id: None,
                    treatment_plan_id: None,
                    treatment_plan_item_id: None,
                    transaction_id: None,
                    financial_status: if cost_cents == 0 { Some("paid".into()) } else { Some("unpaid".into()) },
                    procedure_category: Some(category),
                    procedure_name: name,
                    tooth_number: tooth_opt,
                    surfaces: surfaces_opt,
                    materials_used: materials_opt,
                    status,
                    cost_cents,
                    post_care_instructions: post_care_opt,
                    clinical_notes: notes_opt,
                    performed_at: None,
                };
                create_patient_treatment(&t, &pid, req).await.map(|_| ())
            };

            is_proc_saving.set(false);
            match res {
                Ok(_) => {
                    toast_msg.set(Some("Procedimento salvo com sucesso no prontuário!".into()));
                    is_proc_modal_open.set(false);
                    editing_proc.set(None);
                    on_r.call(());
                }
                Err(e) => {
                    error_toast.set(Some(format!("Erro ao salvar procedimento: {}", e)));
                }
            }
        });
    };

    // Handler para Agendamento Completo
    let tok_sched = token.clone();
    let cid_sched = clinic_id.clone();
    let pid_sched = patient_id.clone();
    let pat_name_sched = patient_name.clone();
    let on_reload_sched = reload_patient_details.clone();
    let team_members_for_sched = agenda_resources.team_members.clone();

    let handle_confirm_schedule = move |_| {
        let Some(proc) = sched_target_proc() else {
            return;
        };

        let title = sched_title().trim().to_string();
        if title.is_empty() {
            error_toast.set(Some("Informe um título para o agendamento.".into()));
            return;
        }

        let scheduled_for = match chrono::NaiveDateTime::parse_from_str(
            &format!("{} {}:00", sched_date(), sched_time()),
            "%Y-%m-%d %H:%M:%S",
        ) {
            Ok(ndt) => chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc).to_rfc3339(),
            Err(_) => {
                error_toast.set(Some("Data ou horário inválido.".into()));
                return;
            }
        };

        let selected_dentist = sched_dentist_id();
        let assigned_users = if !selected_dentist.is_empty() {
            let d_name = team_members_for_sched
                .iter()
                .find(|m| m.id == selected_dentist)
                .map(|m| m.name.clone());
            vec![AssignedUserDto {
                user_id: selected_dentist,
                user_name: d_name,
                role_in_appointment: "Dentista Responsável".to_string(),
                split_percentage: sched_split(),
            }]
        } else if let Some(first) = team_members_for_sched.first() {
            vec![AssignedUserDto {
                user_id: first.id.clone(),
                user_name: Some(first.name.clone()),
                role_in_appointment: "Dentista Responsável".to_string(),
                split_percentage: sched_split(),
            }]
        } else {
            vec![]
        };

        let req = CreateAppointmentRequest {
            clinic_id: cid_sched.clone(),
            patient_id: Some(pid_sched.clone()),
            patient_name: pat_name_sched.clone(),
            treatment_id: Some(proc.id.clone()),
            treatment_plan_id: proc.treatment_plan_id.clone(),
            appointment_type: AppointmentType::Treatment,
            title,
            scheduled_for,
            duration_minutes: sched_duration(),
            assigned_users,
            assigned_equipment: None,
            consumed_items: vec![],
            notes: if sched_notes().trim().is_empty() {
                None
            } else {
                Some(sched_notes().trim().to_string())
            },
            financial_amount_cents: if proc.cost_cents > 0 {
                Some(proc.cost_cents)
            } else {
                None
            },
            financial_type: Some("income".to_string()),
        };

        let t = tok_sched.clone();
        let on_r = on_reload_sched.clone();

        is_scheduling.set(true);
        spawn(async move {
            match create_appointment(&t, req).await {
                Ok(_) => {
                    toast_msg.set(Some("Procedimento agendado na Agenda com sucesso!".into()));
                    is_sched_modal_open.set(false);
                    sched_target_proc.set(None);
                    on_r.call(());
                }
                Err(e) => {
                    error_toast.set(Some(format!("Falha ao criar agendamento: {}", e)));
                }
            }
            is_scheduling.set(false);
        });
    };

    // Handler de Lançamento Financeiro para Procedimento Avulso
    let tok_charge = token.clone();
    let cid_charge = clinic_id.clone();
    let pid_charge = patient_id.clone();
    let p_name_charge = patient_name.clone();
    let on_reload_charge = reload_patient_details.clone();

    let handle_confirm_charge = move |_| {
        let Some(proc) = charge_target_proc() else {
            return;
        };

        let raw_clean = charge_amount_str().replace("R$", "").replace(' ', "").trim().to_string();
        let normalized = if raw_clean.contains(',') && raw_clean.contains('.') {
            raw_clean.replace('.', "").replace(',', ".")
        } else if raw_clean.contains(',') {
            raw_clean.replace(',', ".")
        } else {
            raw_clean
        };

        let amount_float: f64 = match normalized.parse() {
            Ok(v) if v > 0.0 => v,
            _ => {
                error_toast.set(Some("Informe um valor válido maior que zero.".into()));
                return;
            }
        };

        let amount_cents = (amount_float * 100.0).round() as i64;
        let is_paid = charge_is_paid();
        let method = charge_method().trim().to_string();

        let req = CreateTransactionRequest {
            clinic_id: cid_charge.clone(),
            appointment_id: proc.appointment_id.clone(),
            patient_id: Some(pid_charge.clone()),
            patient_name: p_name_charge.clone(),
            user_id: proc.dentist_user_id.clone(),
            treatment_plan_id: None,
            direction: TransactionDirection::Income,
            amount_cents,
            description: format!("Procedimento: {}", proc.procedure_name),
            category: "Tratamento Odontológico".to_string(),
            due_date: chrono::Utc::now().to_rfc3339(),
            paid_date: if is_paid {
                Some(chrono::Utc::now().to_rfc3339())
            } else {
                None
            },
            payment_method: if is_paid { Some(method) } else { None },
            status: if is_paid {
                TransactionStatus::Paid
            } else {
                TransactionStatus::Pending
            },
            installment_current: Some(1),
            installment_total: Some(1),
        };

        let t = tok_charge.clone();
        let on_r = on_reload_charge.clone();

        let pid_val = pid_charge.clone();
        let cid_val = cid_charge.clone();
        let proc_target = proc.clone();

        is_charging.set(true);
        spawn(async move {
            match create_transaction(&t, req).await {
                Ok(created_tx) => {
                    let update_req = UpdatePatientTreatmentRequest {
                        clinic_id: cid_val,
                        dentist_user_id: proc_target.dentist_user_id.clone(),
                        appointment_id: proc_target.appointment_id.clone(),
                        document_id: proc_target.document_id.clone(),
                        exam_id: proc_target.exam_id.clone(),
                        treatment_plan_id: proc_target.treatment_plan_id.clone(),
                        treatment_plan_item_id: proc_target.treatment_plan_item_id.clone(),
                        transaction_id: proc_target.transaction_id.clone(),
                        financial_status: if is_paid { Some("paid".to_string()) } else { Some("pending".to_string()) },
                        procedure_category: proc_target.procedure_category.clone(),
                        procedure_name: proc_target.procedure_name.clone(),
                        tooth_number: proc_target.tooth_number.clone(),
                        surfaces: proc_target.surfaces.clone(),
                        materials_used: proc_target.materials_used.clone(),
                        status: proc_target.status.clone(),
                        cost_cents: proc_target.cost_cents,
                        post_care_instructions: proc_target.post_care_instructions.clone(),
                        clinical_notes: proc_target.clinical_notes.clone(),
                        performed_at: proc_target.appointment_date.clone(),
                    };
                    let _ = update_patient_treatment(&t, &pid_val, &proc_target.id, update_req).await;

                    toast_msg.set(Some("Cobrança financeira registrada com sucesso no Financeiro!".into()));
                    is_charge_modal_open.set(false);
                    charge_target_proc.set(None);
                    on_r.call(());
                }
                Err(e) => {
                    error_toast.set(Some(format!("Falha ao registrar cobrança: {}", e)));
                }
            }
            is_charging.set(false);
        });
    };

    // Handler de exclusão de procedimento
    let tok_del = token.clone();
    let pid_del = patient_id.clone();
    let cid_del = clinic_id.clone();
    let on_reload_del = reload_patient_details.clone();

    let handle_confirm_delete_proc = move |_| {
        let Some(target) = delete_proc_target() else {
            return;
        };
        let target_id = target.id.clone();
        let t = tok_del.clone();
        let pid = pid_del.clone();
        let cid = cid_del.clone();

        is_deleting.set(true);
        spawn(async move {
            match delete_patient_treatment(&t, &pid, &target_id, &cid).await {
                Ok(_) => {
                    toast_msg.set(Some("Procedimento excluído com sucesso do prontuário.".into()));
                    is_delete_proc_modal_open.set(false);
                    delete_proc_target.set(None);
                    on_reload_del.call(());
                }
                Err(e) => {
                    error_toast.set(Some(format!("Falha ao excluir procedimento: {}", e)));
                }
            }
            is_deleting.set(false);
        });
    };

    rsx! {
        div { class: "patient-subtab-container",
            // 1. Cabeçalho de Ações da Aba
            div { class: "tab-header-actions-row",
                div {
                    h3 { class: "tab-title-text", "Procedimentos & Evolução Clínica" }
                    p { class: "tab-subtitle-text",
                        "Histórico de procedimentos do prontuário, status de execução, agendamentos vinculados e baixa no estoque."
                    }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            editing_proc.set(None);
                            selected_template_id.set(String::new());
                            proc_name.set(String::new());
                            proc_category.set("Dentística".to_string());
                            proc_price_str.set("0,00".to_string());
                            proc_status.set("pending".to_string());
                            selected_regions.set(vec![]);
                            selected_teeth.set(vec![]);
                            custom_tooth_input.set(String::new());
                            proc_surfaces.set(String::new());
                            selected_materials.set(vec![]);
                            selected_equipment.set(vec![]);
                            proc_post_care.set(String::new());
                            proc_notes.set(String::new());
                            is_proc_modal_open.set(true);
                        },
                        IconPlus { size: 16, color: "currentColor".to_string() }
                        span { " Adicionar Procedimento" }
                    }
                }
            }

            // 2. Compact Horizontal KPIs (3 Colunas Balanceadas)
            div { class: "patient-subtab-kpis",
                // 1. TOTAL DE PROCEDIMENTOS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                        IconTooth { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Procedimentos no Prontuário" }
                        span { class: "agenda-kpi-sublbl", "{total_treatments} lançados • Total {format_currency(total_cost_cents)}" }
                    }
                    div { class: "agenda-kpi-val", "{total_treatments}" }
                }

                // 2. PENDENTES & AGENDADOS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                        IconClock { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Pendentes & Agendados" }
                        span { class: "agenda-kpi-sublbl", "{pending_count} pendentes, {scheduled_count} agendados" }
                    }
                    div { class: "agenda-kpi-val kpi-pending", "{pending_count + scheduled_count}" }
                }

                // 3. CONCLUÍDOS / REALIZADOS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                        IconCheck { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Concluídos / Realizados" }
                        span { class: "agenda-kpi-sublbl", "Com baixa automática no estoque" }
                    }
                    div { class: "agenda-kpi-val kpi-completed", "{completed_count}" }
                }
            }

            // 3. Toolbar de Ações & Filtros
            div { class: "patient-subtab-toolbar",
                div { class: "search-input-wrap",
                    IconSearch { size: 16, color: "#94a3b8".to_string() }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Buscar por procedimento, dente ou evolução...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                div { class: "flex items-center gap-2",
                    select {
                        class: "select-field",
                        value: "{status_filter}",
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "all", "Todos os Procedimentos" }
                        option { value: "pending", "Status Clínico: Pendentes" }
                        option { value: "scheduled", "Status Clínico: Agendados" }
                        option { value: "in_consultation", "Status Clínico: Em Consulta" }
                        option { value: "completed", "Status Clínico: Concluídos" }
                        option { value: "paid", "Status Financeiro: Pagos (100%)" }
                        option { value: "unpaid", "Status Financeiro: Não Pagos / Pendentes" }
                    }
                }
            }

            // 4. Tabela de Procedimentos
            if filtered_treatments.is_empty() {
                div { class: "empty-state-card mt-4",
                    IconTooth { size: 42, color: "#cbd5e1".to_string() }
                    h4 { class: "empty-state-title", "Nenhum procedimento encontrado" }
                    p { class: "empty-state-desc",
                        "Aprove um orçamento para importar os procedimentos planejados ou adicione um avulso acima."
                    }
                }
            } else {
                div { class: "table-container",
                    table { class: "modern-table",
                        thead {
                            tr {
                                th { style: "width: 26%;", "Procedimento / Tratamento" }
                                th { style: "width: 12%;", "Dente / Faces" }
                                th { style: "width: 14%;", "Status Clínico" }
                                th { style: "width: 11%;", "Status Financeiro" }
                                th { style: "width: 11%;", "Valor" }
                                th { style: "width: 12%;", "Data / Evolução" }
                                th { class: "text-right", style: "width: 14%;", "Ações" }
                            }
                        }
                        tbody {
                            for treat in filtered_treatments {
                                {
                                    let treat_clone_status = treat.clone();
                                    let treat_clone_edit = treat.clone();
                                    let treat_clone_del = treat.clone();
                                    let treat_clone_sched = treat.clone();
                                    let treat_clone_charge = treat.clone();

                                    let is_manual = treat.treatment_plan_id.is_none();
                                    let fin_badge = financial_status_badge(treat.financial_status.as_deref());

                                    let tok_inline = token.clone();
                                    let pid_inline = patient_id.clone();
                                    let cid_inline = clinic_id.clone();
                                    let on_r_inline = reload_patient_details.clone();
                                    let pat_name_inline = patient_name.clone();
                                    let first_dentist_opt = agenda_resources.team_members.first().map(|m| m.id.clone());

                                    rsx! {
                                        tr { key: "{treat.id}",
                                            td {
                                                div { style: "display: flex; flex-direction: column; gap: 4px;",
                                                    div { style: "display: flex; align-items: center; gap: 8px; flex-wrap: wrap;",
                                                        span { style: "font-weight: 700; color: #0f172a; font-size: 14px;", "{treat.procedure_name}" }
                                                        if treat.treatment_plan_id.is_some() {
                                                            span { class: "badge-status-neutral font-xs", style: "background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; font-size: 10.5px; padding: 1px 6px; border-radius: 4px;",
                                                                "Orçamento"
                                                            }
                                                        } else {
                                                            span { class: "badge-status-neutral font-xs", style: "background: #f8fafc; color: #64748b; border: 1px solid #e2e8f0; font-size: 10.5px; padding: 1px 6px; border-radius: 4px;",
                                                                "Avulso"
                                                            }
                                                        }
                                                    }
                                                    if let Some(ref cat) = treat.procedure_category {
                                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                                            span { style: "display: inline-block; width: 6px; height: 6px; border-radius: 50%; background: #0284c7;" }
                                                            span { style: "font-size: 12px; color: #64748b; font-weight: 500;", "{cat}" }
                                                        }
                                                    }
                                                }
                                            }
                                            td {
                                                if let Some(ref tooth) = treat.tooth_number {
                                                    span { class: "badge-status-neutral", "{tooth}" }
                                                } else {
                                                    span { class: "text-muted font-sm", "Geral" }
                                                }
                                            }
                                            td {
                                                // Seletor inline de Status Clínico
                                                select {
                                                    class: "status-select-inline",
                                                    disabled: !can_write,
                                                    value: "{treat.status}",
                                                    onchange: move |e| {
                                                        let new_st = e.value();
                                                        let t = tok_inline.clone();
                                                        let pid = pid_inline.clone();
                                                        let cid = cid_inline.clone();
                                                        let on_r = on_r_inline.clone();
                                                        let curr = treat_clone_status.clone();
                                                        spawn(async move {
                                                            let req = UpdatePatientTreatmentRequest {
                                                                clinic_id: cid,
                                                                dentist_user_id: curr.dentist_user_id,
                                                                appointment_id: curr.appointment_id,
                                                                document_id: curr.document_id,
                                                                exam_id: curr.exam_id,
                                                                treatment_plan_id: curr.treatment_plan_id,
                                                                treatment_plan_item_id: curr.treatment_plan_item_id,
                                                                transaction_id: curr.transaction_id,
                                                                financial_status: curr.financial_status,
                                                                procedure_category: curr.procedure_category,
                                                                procedure_name: curr.procedure_name,
                                                                tooth_number: curr.tooth_number,
                                                                surfaces: curr.surfaces,
                                                                materials_used: curr.materials_used,
                                                                status: new_st,
                                                                cost_cents: curr.cost_cents,
                                                                post_care_instructions: curr.post_care_instructions,
                                                                clinical_notes: curr.clinical_notes,
                                                                performed_at: curr.appointment_date,
                                                            };
                                                            let _ = update_patient_treatment(&t, &pid, &curr.id, req).await;
                                                            on_r.call(());
                                                        });
                                                    },
                                                    option { value: "pending", "Pendente" }
                                                    option { value: "scheduled", "Agendado" }
                                                    option { value: "in_consultation", "Em Consulta" }
                                                    option { value: "completed", "Concluído" }
                                                }
                                            }
                                            td {
                                                span { class: "badge-status {fin_badge.0}", "{fin_badge.1}" }
                                            }
                                            td { class: "font-semibold text-slate-800 font-mono",
                                                "{format_currency(treat.cost_cents)}"
                                            }
                                            td { class: "font-xs text-muted",
                                                "{treat.created_at.chars().take(10).collect::<String>()}"
                                                if let Some(ref notes) = treat.clinical_notes {
                                                    if !notes.trim().is_empty() {
                                                        span { class: "block text-slate-600 truncate mt-0.5", style: "max-width: 140px;", title: "{notes}", "{notes}" }
                                                    }
                                                }
                                            }
                                            td { class: "text-right",
                                                div { style: "display: flex; align-items: center; justify-content: flex-end; gap: 8px;",
                                                    // Botão Agendar
                                                    if can_write && treat.status != "completed" {
                                                        button {
                                                            class: "btn-secondary btn-sm",
                                                            style: "height: 32px; padding: 0 10px; font-size: 12px; font-weight: 600; display: flex; align-items: center; gap: 4px;",
                                                            title: "Agendar este procedimento na agenda",
                                                            onclick: move |_| {
                                                                sched_target_proc.set(Some(treat_clone_sched.clone()));
                                                                sched_title.set(format!("{} - {}", treat_clone_sched.procedure_name, pat_name_inline.clone().unwrap_or_default()));
                                                                sched_date.set(chrono::Local::now().format("%Y-%m-%d").to_string());
                                                                sched_time.set("09:00".to_string());
                                                                sched_duration.set(45);
                                                                sched_split.set(100);
                                                                sched_notes.set(treat_clone_sched.clinical_notes.clone().unwrap_or_default());
                                                                if let Some(ref fid) = first_dentist_opt {
                                                                    sched_dentist_id.set(fid.clone());
                                                                }
                                                                is_sched_modal_open.set(true);
                                                            },
                                                            IconCalendar { size: 14, color: "#475569".to_string() }
                                                            span { "Agendar" }
                                                        }
                                                    }

                                                    // Botão Cobrar / Lançar no Financeiro para Procedimentos Manuais
                                                    if can_write && is_manual && treat.financial_status.as_deref() != Some("paid") && treat.cost_cents > 0 {
                                                        button {
                                                            class: "btn-secondary btn-sm",
                                                            style: "height: 32px; padding: 0 10px; font-size: 12px; font-weight: 600; display: flex; align-items: center; gap: 4px; color: #0284c7; border-color: #bae6fd; background: #f0f9ff;",
                                                            title: "Lançar cobrança deste procedimento avulso no Financeiro",
                                                            onclick: move |_| {
                                                                let val = (treat_clone_charge.cost_cents as f64) / 100.0;
                                                                charge_amount_str.set(format!("{:.2}", val));
                                                                charge_is_paid.set(true);
                                                                charge_method.set("Pix".to_string());
                                                                charge_notes.set(String::new());
                                                                charge_target_proc.set(Some(treat_clone_charge.clone()));
                                                                is_charge_modal_open.set(true);
                                                            },
                                                            IconFinance { size: 14, color: "#0284c7".to_string() }
                                                            span { "Cobrar" }
                                                        }
                                                    }

                                                    if can_write {
                                                        button {
                                                            class: "btn-action-icon",
                                                            style: "width: 32px; height: 32px;",
                                                            title: "Editar Procedimento",
                                                            onclick: move |_| {
                                                                let p = treat_clone_edit.clone();
                                                                selected_template_id.set(String::new());
                                                                proc_name.set(p.procedure_name.clone());
                                                                proc_category.set(p.procedure_category.clone().unwrap_or_else(|| "Dentística".into()));
                                                                proc_surfaces.set(p.surfaces.clone().map(|s| s.join(", ")).unwrap_or_default());
                                                                proc_price_str.set(format!("{:.2}", (p.cost_cents as f64) / 100.0).replace('.', ","));
                                                                
                                                                let mut teeth_parsed = vec![];
                                                                if let Some(ref t_str) = p.tooth_number {
                                                                    for piece in t_str.split(',') {
                                                                        let clean = piece.trim().to_string();
                                                                        if !clean.is_empty() {
                                                                            teeth_parsed.push(clean);
                                                                        }
                                                                    }
                                                                }
                                                                selected_teeth.set(teeth_parsed);
                                                                selected_regions.set(vec![]);
                                                                custom_tooth_input.set(String::new());

                                                                let mats = p.materials_used.clone().unwrap_or_default()
                                                                    .iter().map(|s| parse_stored_material(s)).collect();
                                                                selected_materials.set(mats);
                                                                selected_equipment.set(vec![]);

                                                                proc_post_care.set(p.post_care_instructions.clone().unwrap_or_default());
                                                                proc_notes.set(p.clinical_notes.clone().unwrap_or_default());
                                                                proc_status.set(p.status.clone());
                                                                editing_proc.set(Some(p));
                                                                is_proc_modal_open.set(true);
                                                            },
                                                            IconEdit { size: 15, color: "#64748b".to_string() }
                                                        }
                                                    }

                                                    if can_delete {
                                                        button {
                                                            class: "btn-action-icon text-danger",
                                                            style: "width: 32px; height: 32px;",
                                                            title: "Excluir Procedimento",
                                                            onclick: move |_| {
                                                                delete_proc_target.set(Some(treat_clone_del.clone()));
                                                                is_delete_proc_modal_open.set(true);
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

            // Modal Completo de Registro / Edição de Procedimento Clínico (Estrutura idêntica ao Novo Procedimento Padrão)
            if is_proc_modal_open() {
                div { class: "modal-overlay", onclick: move |_| is_proc_modal_open.set(false),
                    div {
                        class: "modal-card treatment-template-modal",
                        style: "max-width: 860px; width: 95vw; max-height: 90vh; display: flex; flex-direction: column; border-radius: 16px; background: #ffffff; box-shadow: 0 25px 60px -15px rgba(15, 23, 42, 0.3);",
                        onclick: move |e| e.stop_propagation(),

                        // 1. Header do Modal
                        div { class: "modal-header", style: "padding: 20px 26px; border-bottom: 1px solid #e2e8f0;",
                            div { class: "modal-header-left flex items-center gap-3",
                                div { class: "stock-header-icon-box", style: "width: 44px; height: 44px; border-radius: 10px; background: #eff6ff; border: 1px solid #bfdbfe; display: flex; align-items: center; justify-content: center; color: #0284c7;",
                                    IconTooth { size: 22, color: "#0284c7".to_string() }
                                }
                                div { class: "header-text-col",
                                    h2 { class: "modal-title", style: "font-size: 1.2rem; font-weight: 700; color: #0f172a; margin: 0;",
                                        if editing_proc().is_some() { "Editar Procedimento Clínico" } else { "Adicionar Procedimento no Prontuário" }
                                    }
                                    p { class: "modal-subtitle", style: "font-size: 0.85rem; color: #64748b; margin-top: 4px; margin-bottom: 0;",
                                        "Carregue um modelo do catálogo ou especifique dentes, insumos e valor clínico."
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "modal-close-btn",
                                onclick: move |_| is_proc_modal_open.set(false),
                                "✕"
                            }
                        }

                        // 2. Conteúdo com Rolagem Fluida
                        div { class: "modal-body treatment-modal-scroll", style: "padding: 24px 28px; overflow-y: auto; flex: 1; display: flex; flex-direction: column; gap: 20px;",
                            // Seletor Rápido de Modelo do Catálogo
                            div { class: "input-group-wrapper full-width", style: "background: #f0f9ff; border: 1px solid #bae6fd; border-radius: 10px; padding: 14px 18px;",
                                label { style: "font-size: 0.85rem; font-weight: 700; color: #0369a1; margin-bottom: 6px; display: block;",
                                    "✨ Carregar Dados a partir de um Modelo do Catálogo (Opcional)"
                                }
                                select {
                                    class: "modern-input-field modern-select",
                                    style: "height: 42px; width: 100%; border: 1px solid #7dd3fc; border-radius: 8px; background: #ffffff; font-size: 0.92rem;",
                                    value: "{selected_template_id}",
                                    onchange: move |e| {
                                        let val = e.value();
                                        selected_template_id.set(val.clone());
                                        if let Some(tmpl) = templates_list.iter().find(|t| t.id == val) {
                                            proc_name.set(tmpl.name.clone());
                                            proc_category.set(tmpl.category.clone().unwrap_or_else(|| "Dentística".into()));
                                            proc_price_str.set(format!("{:.2}", (tmpl.default_price_cents as f64) / 100.0).replace('.', ","));
                                            selected_teeth.set(tmpl.target_teeth.clone());
                                            selected_regions.set(tmpl.dental_regions.clone());
                                            let mats = tmpl.required_materials.iter().map(|s| parse_stored_material(s)).collect();
                                            selected_materials.set(mats);
                                            selected_equipment.set(tmpl.required_equipment.clone());
                                            proc_post_care.set(tmpl.post_care_instructions.clone().unwrap_or_default());
                                            proc_notes.set(tmpl.clinical_notes.clone().unwrap_or_default());
                                        }
                                    },
                                    option { value: "", "+ Selecionar Procedimento Padrão do Catálogo..." }
                                    for tmpl in templates_list.iter() {
                                        {
                                            let cat = tmpl.category.as_deref().unwrap_or("Geral");
                                            rsx! {
                                                option { value: "{tmpl.id}",
                                                    "{tmpl.name} ({format_currency(tmpl.default_price_cents)}) - {cat}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // 1. Especialidade Odontológica
                            div { class: "input-group-wrapper full-width",
                                label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Especialidade Odontológica *" }
                                div { class: "treatment-category-selector-grid",
                                    for cat in categories.iter() {
                                        {
                                            let c = cat.to_string();
                                            let is_sel = proc_category() == c;
                                            rsx! {
                                                button {
                                                    key: "{cat}",
                                                    r#type: "button",
                                                    class: if is_sel { "treatment-category-card active" } else { "treatment-category-card" },
                                                    onclick: move |_| proc_category.set(c.clone()),
                                                    span { "{cat}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // 2. Nome do Procedimento
                            div { class: "input-group-wrapper full-width",
                                label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Nome do Procedimento *" }
                                input {
                                    class: "modern-input-field",
                                    style: "width: 100%; height: 42px; border: 1px solid #cbd5e1; border-radius: 8px; padding: 0 14px; font-weight: 600;",
                                    placeholder: "Ex: Restauração Resina Fotopolimerizável 2 Faces",
                                    value: "{proc_name}",
                                    oninput: move |e| proc_name.set(e.value()),
                                }
                            }

                            // 3. Valor (R$) e Status Clínico Inicial
                            div { style: "display: grid; grid-template-columns: 1fr 1.2fr; gap: 16px;",
                                div { class: "input-group-wrapper",
                                    label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Valor do Procedimento (R$)" }
                                    input {
                                        r#type: "text",
                                        class: "modern-input-field font-mono font-semibold",
                                        style: "width: 100%; height: 42px; border: 1px solid #cbd5e1; border-radius: 8px; padding: 0 14px;",
                                        placeholder: "0,00",
                                        value: "{proc_price_str}",
                                        oninput: move |e| proc_price_str.set(e.value()),
                                    }
                                }

                                div { class: "input-group-wrapper",
                                    label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Status Clínico Inicial *" }
                                    select {
                                        class: "modern-input-field modern-select",
                                        style: "width: 100%; height: 42px; border: 1px solid #cbd5e1; border-radius: 8px; padding: 0 12px;",
                                        value: "{proc_status}",
                                        onchange: move |e| proc_status.set(e.value()),
                                        option { value: "pending", "Pendente (Planejado)" }
                                        option { value: "scheduled", "Agendado na Agenda" }
                                        option { value: "in_consultation", "Em Consulta (Em andamento)" }
                                        option { value: "completed", "Concluído (Baixa automática no estoque)" }
                                    }
                                }
                            }

                            // 4. Regiões Odontológicas e Dentes Alvo
                            div { class: "input-group-wrapper full-width",
                                label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Regiões Odontológicas ou Dentes" }
                                div { class: "region-chips-grid", style: "display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 10px;",
                                    for reg in ["Arcada Superior", "Arcada Inferior", "Dentes Anteriores", "Molares", "Todos os Dentes", "Geral"].iter() {
                                        {
                                            let r = reg.to_string();
                                            let is_sel = selected_regions().contains(&r);
                                            rsx! {
                                                button {
                                                    key: "{reg}",
                                                    r#type: "button",
                                                    class: if is_sel { "region-chip active" } else { "region-chip" },
                                                    style: if is_sel {
                                                        "height: 34px; padding: 0 16px; border-radius: 9999px; font-size: 12.5px; font-weight: 600; cursor: pointer; transition: all 0.15s ease; border: 1.5px solid #0284c7; background: #e0f2fe; color: #0369a1; display: inline-flex; align-items: center; gap: 6px; box-shadow: 0 2px 4px rgba(2, 132, 199, 0.15);"
                                                    } else {
                                                        "height: 34px; padding: 0 16px; border-radius: 9999px; font-size: 12.5px; font-weight: 500; cursor: pointer; transition: all 0.15s ease; border: 1px solid #cbd5e1; background: #ffffff; color: #475569; display: inline-flex; align-items: center; gap: 6px; box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);"
                                                    },
                                                    onclick: move |_| {
                                                        let mut list = selected_regions();
                                                        if list.contains(&r) {
                                                            list.retain(|x| x != &r);
                                                        } else {
                                                            list.push(r.clone());
                                                        }
                                                        selected_regions.set(list);
                                                    },
                                                    if is_sel {
                                                        span { style: "color: #0284c7; font-weight: 800; font-size: 13px;", "✓" }
                                                    }
                                                    span { "{reg}" }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Campo de Dentes e Faces
                                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 14px;",
                                    div {
                                        label { style: "font-size: 0.75rem; color: #64748b; font-weight: 600; display: block; margin-bottom: 4px;", "Dentes Específicos (ex: 16, 21, 38)" }
                                        input {
                                            class: "modern-input-field font-mono",
                                            style: "width: 100%; height: 40px; border: 1px solid #cbd5e1; border-radius: 8px; padding: 0 12px;",
                                            placeholder: "Ex: 11, 12, 21, 22...",
                                            value: "{custom_tooth_input}",
                                            oninput: move |e| custom_tooth_input.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { style: "font-size: 0.75rem; color: #64748b; font-weight: 600; display: block; margin-bottom: 4px;", "Faces Envolvidas (ex: V, M, D, O, L)" }
                                        input {
                                            class: "modern-input-field",
                                            style: "width: 100%; height: 40px; border: 1px solid #cbd5e1; border-radius: 8px; padding: 0 12px;",
                                            placeholder: "Ex: Oclusal, Mesial, Distal",
                                            value: "{proc_surfaces}",
                                            oninput: move |e| proc_surfaces.set(e.value()),
                                        }
                                    }
                                }
                            }

                            // 5. Materiais e Insumos do Estoque
                            div { class: "input-group-wrapper full-width stock-sync-section",
                                div { class: "sync-section-header",
                                    div { class: "sync-title-wrap",
                                        IconBox { size: 18, color: "#0284c7".to_string() }
                                        strong { "Materiais & Insumos Utilizados (Estoque da Clínica)" }
                                    }
                                    span { class: "sync-badge-count", "{selected_materials().len()} insumos definidos" }
                                }

                                // Seletor de Estoque com Quantidade
                                div { class: "stock-picker-grid-with-qty",
                                    div { class: "picker-col-select",
                                        select {
                                            class: "modern-input-field modern-select stock-picker-select",
                                            value: "{chosen_stock_mat_idx}",
                                            onchange: move |e: FormEvent| chosen_stock_mat_idx.set(e.value()),
                                            option { value: "", "🔍 Selecione um item do estoque..." }
                                            for (idx, mat) in stock_materials.iter().enumerate() {
                                                {
                                                    let m_manuf = mat.manufacturer.clone().unwrap_or_default();
                                                    let display_opt = format!("{} ({} {} em estoque) - {}", mat.name, mat.current_stock, mat.unit_type, m_manuf);
                                                    rsx! {
                                                        option {
                                                            value: "{idx}",
                                                            "{display_opt}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "picker-col-qty",
                                        span { class: "picker-qty-label", "Qtd:" }
                                        input {
                                            r#type: "number",
                                            class: "modern-input-field font-mono qty-number-input",
                                            min: "1",
                                            value: "{chosen_mat_qty}",
                                            oninput: move |e| {
                                                if let Ok(v) = e.value().parse::<i32>() {
                                                    chosen_mat_qty.set(v.max(1));
                                                }
                                            },
                                        }
                                    }

                                    button {
                                        r#type: "button",
                                        class: "btn-primary btn-add-mat",
                                        onclick: move |_| {
                                            let idx_str = chosen_stock_mat_idx();
                                            if let Ok(idx) = idx_str.parse::<usize>() {
                                                if let Some(item) = stock_materials.get(idx) {
                                                    let mut list = selected_materials();
                                                    let qty = chosen_mat_qty();
                                                    if let Some(existing) = list.iter_mut().find(|m| m.name == item.name) {
                                                        existing.quantity += qty;
                                                    } else {
                                                        list.push(SelectedMaterialItem {
                                                            name: item.name.clone(),
                                                            quantity: qty,
                                                            unit: item.unit_type.clone(),
                                                        });
                                                    }
                                                    selected_materials.set(list);
                                                    chosen_stock_mat_idx.set(String::new());
                                                    chosen_mat_qty.set(1);
                                                }
                                            }
                                        },
                                        IconPlus { size: 15, color: "#ffffff".to_string() }
                                        span { "Adicionar" }
                                    }
                                }

                                // Linha para adicionar material manual personalizado
                                div { class: "custom-material-manual-row",
                                    input {
                                        class: "modern-input-field flex-2",
                                        placeholder: "Ou digite outro material manual...",
                                        value: "{custom_mat_name}",
                                        oninput: move |e| custom_mat_name.set(e.value()),
                                    }
                                    select {
                                        class: "modern-input-field modern-select flex-1",
                                        value: "{custom_mat_unit}",
                                        onchange: move |e: FormEvent| custom_mat_unit.set(e.value()),
                                        option { value: "unidade", "unidade" }
                                        option { value: "par", "par" }
                                        option { value: "caixa", "caixa" }
                                        option { value: "frasco", "frasco" }
                                        option { value: "tubete", "tubete" }
                                        option { value: "grama", "grama" }
                                        option { value: "ml", "ml" }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn-secondary btn-sm",
                                        onclick: move |_| {
                                            let name = custom_mat_name().trim().to_string();
                                            if !name.is_empty() {
                                                let mut list = selected_materials();
                                                list.push(SelectedMaterialItem {
                                                    name,
                                                    quantity: chosen_mat_qty(),
                                                    unit: custom_mat_unit(),
                                                });
                                                selected_materials.set(list);
                                                custom_mat_name.set(String::new());
                                            }
                                        },
                                        IconPlus { size: 14, color: "currentColor".to_string() }
                                        span { "Incluir Manual" }
                                    }
                                }

                                // Lista dos Materiais Selecionados com Quantidades
                                if !selected_materials().is_empty() {
                                    div { class: "selected-materials-list-container",
                                        for (idx, mat) in selected_materials().iter().enumerate() {
                                            {
                                                let m_clone = mat.clone();
                                                rsx! {
                                                    div { key: "{idx}_{mat.name}", class: "selected-material-card-row",
                                                        div { class: "mat-row-left",
                                                            span { class: "mat-qty-badge", "{m_clone.quantity}x" }
                                                            IconBox { size: 15, color: "#0284c7".to_string() }
                                                            span { class: "mat-name-text", "{m_clone.name}" }
                                                            span { class: "mat-unit-pill", "({m_clone.unit})" }
                                                        }

                                                        div { class: "mat-row-actions",
                                                            button {
                                                                r#type: "button",
                                                                class: "mat-remove-btn",
                                                                title: "Remover este insumo",
                                                                onclick: move |_| {
                                                                    let mut list = selected_materials();
                                                                    if idx < list.len() {
                                                                        list.remove(idx);
                                                                        selected_materials.set(list);
                                                                    }
                                                                },
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

                            // 6. Equipamentos Odontológicos Vinculados
                            div { class: "input-group-wrapper full-width stock-sync-section",
                                div { class: "sync-section-header",
                                    div { class: "sync-title-wrap",
                                        IconTool { size: 18, color: "#0284c7".to_string() }
                                        strong { "Equipamentos Odontológicos Necessários" }
                                    }
                                    span { class: "sync-badge-count", "{selected_equipment().len()} vinculados" }
                                }

                                div { class: "stock-picker-grid",
                                    select {
                                        class: "modern-input-field modern-select",
                                        onchange: move |e: FormEvent| {
                                            let val = e.value();
                                            if !val.is_empty() {
                                                let mut list = selected_equipment();
                                                if !list.contains(&val) {
                                                    list.push(val);
                                                    selected_equipment.set(list);
                                                }
                                            }
                                        },
                                        option { value: "", "🔍 Vincular equipamento do estoque..." }
                                        for eq in stock_equipments.iter() {
                                            {
                                                let eq_manuf = eq.manufacturer.clone().unwrap_or_default();
                                                let eq_display = if eq_manuf.is_empty() { eq.name.clone() } else { format!("{} - {}", eq.name, eq_manuf) };
                                                rsx! {
                                                    option { value: "{eq.name}", "{eq_display}" }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "custom-input-with-btn",
                                        input {
                                            class: "modern-input-field",
                                            placeholder: "Ou digite outro equipamento...",
                                            value: "{custom_equipment_input}",
                                            oninput: move |e| custom_equipment_input.set(e.value()),
                                        }
                                        button {
                                            r#type: "button",
                                            class: "btn-secondary btn-sm",
                                            onclick: move |_| {
                                                let eq = custom_equipment_input().trim().to_string();
                                                if !eq.is_empty() {
                                                    let mut list = selected_equipment();
                                                    if !list.contains(&eq) {
                                                        list.push(eq);
                                                        selected_equipment.set(list);
                                                    }
                                                    custom_equipment_input.set(String::new());
                                                }
                                            },
                                            IconPlus { size: 14, color: "currentColor".to_string() }
                                            span { "Adicionar" }
                                        }
                                    }
                                }

                                if !selected_equipment().is_empty() {
                                    div { class: "selected-tags-flex-wrap",
                                        for (idx, eq) in selected_equipment().iter().enumerate() {
                                            {
                                                let eq_str = eq.clone();
                                                rsx! {
                                                    div { key: "{idx}_{eq}", class: "selected-equipment-pill",
                                                        IconTool { size: 13, color: "#0284c7".to_string() }
                                                        span { "{eq_str}" }
                                                        button {
                                                            r#type: "button",
                                                            class: "tag-remove-x",
                                                            onclick: move |_| {
                                                                let mut list = selected_equipment();
                                                                list.retain(|x| x != &eq_str);
                                                                selected_equipment.set(list);
                                                            },
                                                            "×"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // 7. Instruções Pós-Atendimento e Recomendações
                            div { class: "input-group-wrapper full-width",
                                label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Instruções Pós-Atendimento e Recomendações" }
                                input {
                                    class: "modern-input-field",
                                    style: "width: 100%; height: 42px; border: 1px solid #cbd5e1; border-radius: 8px; padding: 0 14px;",
                                    placeholder: "Ex: Evitar mastigação rígida por 24h; aplicar compressa de gelo...",
                                    value: "{proc_post_care}",
                                    oninput: move |e| proc_post_care.set(e.value()),
                                }
                            }

                            // 8. Evolução Clínica e Anotações do Prontuário
                            div { class: "input-group-wrapper full-width",
                                label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Evolução Clínica / Anotações do Prontuário" }
                                textarea {
                                    class: "modern-input-field",
                                    style: "width: 100%; padding: 12px 14px; border: 1px solid #cbd5e1; border-radius: 8px; min-height: 75px;",
                                    rows: "3",
                                    placeholder: "Descreva procedimentos realizados, anestésico, intercorrências ou evolução clínica...",
                                    value: "{proc_notes}",
                                    oninput: move |e| proc_notes.set(e.value()),
                                }
                            }
                        }

                        // 3. Rodapé do Modal
                        div { class: "modal-footer-actions", style: "padding: 18px 28px; background: #f8fafc; border-top: 1px solid #e2e8f0; display: flex; justify-content: flex-end; gap: 12px; border-bottom-left-radius: 16px; border-bottom-right-radius: 16px;",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                style: "height: 42px; padding: 0 20px; border-radius: 8px; font-weight: 600;",
                                onclick: move |_| is_proc_modal_open.set(false),
                                "Cancelar"
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary",
                                style: "height: 42px; padding: 0 24px; border-radius: 8px; font-weight: 700; background: #0284c7;",
                                disabled: is_proc_saving(),
                                onclick: handle_save_proc,
                                if is_proc_saving() { "Salvando..." } else { "Salvar no Prontuário" }
                            }
                        }
                    }
                }
            }

            // Modal Completo de Agendamento do Procedimento
            if is_sched_modal_open() {
                if let Some(ref proc) = *sched_target_proc.read() {
                    div { class: "modal-overlay", onclick: move |_| is_sched_modal_open.set(false),
                        div { class: "action-modal", style: "max-width: 580px; border-radius: 12px; background: #ffffff;", onclick: move |e| e.stop_propagation(),
                            div { class: "modal-header", style: "padding: 20px 24px; border-bottom: 1px solid #e2e8f0; display: flex; align-items: center; justify-content: space-between;",
                                div { class: "flex items-center gap-3",
                                    div { style: "width: 40px; height: 40px; border-radius: 8px; background: #eff6ff; border: 1px solid #bfdbfe; display: flex; align-items: center; justify-content: center; color: #1d4ed8;",
                                        IconCalendar { size: 20, color: "#1d4ed8".to_string() }
                                    }
                                    div {
                                        h3 { class: "modal-title font-bold text-slate-800", style: "font-size: 1.15rem; margin: 0;", "Agendar Procedimento na Agenda" }
                                        p { class: "modal-subtitle font-xs text-muted mt-1", "{proc.procedure_name}" }
                                    }
                                }
                                button { class: "modal-close", onclick: move |_| is_sched_modal_open.set(false), "×" }
                            }

                            div { class: "modal-body", style: "padding: 22px 24px; display: flex; flex-direction: column; gap: 16px;",
                                div { class: "form-group",
                                    label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Título do Agendamento *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field font-semibold",
                                        style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 14px;",
                                        value: "{sched_title}",
                                        oninput: move |e| sched_title.set(e.value()),
                                    }
                                }

                                div { style: "display: grid; grid-template-columns: 1.2fr 1fr; gap: 14px;",
                                    div { class: "form-group",
                                        label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Data da Consulta *" }
                                        input {
                                            r#type: "date",
                                            class: "input-field font-mono",
                                            style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 14px;",
                                            value: "{sched_date}",
                                            oninput: move |e| sched_date.set(e.value()),
                                        }
                                    }

                                    div { class: "form-group",
                                        label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Horário *" }
                                        input {
                                            r#type: "time",
                                            class: "input-field font-mono",
                                            style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 14px;",
                                            value: "{sched_time}",
                                            oninput: move |e| sched_time.set(e.value()),
                                        }
                                    }
                                }

                                div { style: "display: grid; grid-template-columns: 1.5fr 1fr; gap: 14px;",
                                    div { class: "form-group",
                                        label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Dentista Responsável *" }
                                        select {
                                            class: "select-field",
                                            style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 12px;",
                                            value: "{sched_dentist_id}",
                                            onchange: move |e| sched_dentist_id.set(e.value()),
                                            option { value: "", "Selecionar Dentista..." }
                                            for member in &agenda_resources.team_members {
                                                {
                                                    let info_str = member.extra_info.clone().unwrap_or_default();
                                                    let display_text = if info_str.is_empty() {
                                                        member.name.clone()
                                                    } else {
                                                        format!("{} ({})", member.name, info_str)
                                                    };
                                                    rsx! {
                                                        option { value: "{member.id}", "{display_text}" }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "form-group",
                                        label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Duração Estimada" }
                                        select {
                                            class: "select-field",
                                            style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 12px;",
                                            value: "{sched_duration}",
                                            onchange: move |e| {
                                                if let Ok(v) = e.value().parse::<i32>() {
                                                    sched_duration.set(v);
                                                }
                                            },
                                            option { value: "15", "15 minutos" }
                                            option { value: "30", "30 minutos" }
                                            option { value: "45", "45 minutos" }
                                            option { value: "60", "1 hora (60m)" }
                                            option { value: "90", "1h 30m (90m)" }
                                            option { value: "120", "2 horas (120m)" }
                                        }
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Observações e Instruções Pré-Consulta" }
                                    textarea {
                                        class: "input-field",
                                        style: "width: 100%; padding: 12px; border: 1px solid #cbd5e1; border-radius: 6px;",
                                        rows: "2",
                                        placeholder: "Ex: Paciente com sensibilidade; preparar moldeira e kit cirúrgico...",
                                        value: "{sched_notes}",
                                        oninput: move |e| sched_notes.set(e.value()),
                                    }
                                }
                            }

                            div { class: "modal-footer", style: "padding: 16px 24px; background: #f8fafc; border-top: 1px solid #e2e8f0; display: flex; justify-content: flex-end; gap: 10px;",
                                button { class: "btn-secondary", style: "height: 40px; padding: 0 18px;", onclick: move |_| is_sched_modal_open.set(false), "Cancelar" }
                                button {
                                    class: "btn-primary",
                                    style: "height: 40px; padding: 0 20px; display: flex; align-items: center; gap: 6px;",
                                    disabled: is_scheduling(),
                                    onclick: handle_confirm_schedule,
                                    IconCalendar { size: 16, color: "#ffffff".to_string() }
                                    span { if is_scheduling() { "Agendando..." } else { "Confirmar Agendamento" } }
                                }
                            }
                        }
                    }
                }
            }

            // Modal de Lançamento Financeiro para Procedimento Avulso
            if is_charge_modal_open() {
                if let Some(ref proc) = *charge_target_proc.read() {
                    div { class: "modal-overlay", onclick: move |_| is_charge_modal_open.set(false),
                        div { class: "action-modal", style: "max-width: 500px; border-radius: 12px; background: #ffffff;", onclick: move |e| e.stop_propagation(),
                            div { class: "modal-header", style: "padding: 20px 24px; border-bottom: 1px solid #e2e8f0; display: flex; align-items: center; justify-content: space-between;",
                                div { class: "flex items-center gap-3",
                                    div { style: "width: 40px; height: 40px; border-radius: 8px; background: #ecfdf5; border: 1px solid #a7f3d0; display: flex; align-items: center; justify-content: center; color: #059669;",
                                        IconFinance { size: 20, color: "#059669".to_string() }
                                    }
                                    div {
                                        h3 { class: "modal-title font-bold text-slate-800", style: "font-size: 1.15rem; margin: 0;", "Lançar Cobrança no Financeiro" }
                                        p { class: "modal-subtitle font-xs text-muted mt-1", "{proc.procedure_name}" }
                                    }
                                }
                                button { class: "modal-close", onclick: move |_| is_charge_modal_open.set(false), "×" }
                            }

                            div { class: "modal-body", style: "padding: 22px 24px; display: flex; flex-direction: column; gap: 16px;",
                                div { class: "form-group",
                                    label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Valor da Cobrança (R$) *" }
                                    input {
                                        r#type: "text",
                                        class: "input-field font-mono font-semibold",
                                        style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 14px;",
                                        value: "{charge_amount_str}",
                                        oninput: move |e| charge_amount_str.set(e.value()),
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Status do Pagamento" }
                                    select {
                                        class: "select-field",
                                        style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 12px;",
                                        value: if charge_is_paid() { "paid" } else { "pending" },
                                        onchange: move |e| charge_is_paid.set(e.value() == "paid"),
                                        option { value: "paid", "Pago Imediatamente" }
                                        option { value: "pending", "Pendente (Aguardando Pagamento)" }
                                    }
                                }

                                if charge_is_paid() {
                                    div { class: "form-group",
                                        label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Método de Pagamento *" }
                                        select {
                                            class: "select-field",
                                            style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 12px;",
                                            value: "{charge_method}",
                                            onchange: move |e| charge_method.set(e.value()),
                                            option { value: "Pix", "Pix" }
                                            option { value: "Cartão de Crédito", "Cartão de Crédito" }
                                            option { value: "Cartão de Débito", "Cartão de Débito" }
                                            option { value: "Dinheiro", "Dinheiro" }
                                            option { value: "Boleto Bancário", "Boleto Bancário" }
                                            option { value: "Transferência TED/DOC", "Transferência TED/DOC" }
                                        }
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Observações do Lançamento" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        style: "height: 42px; width: 100%; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 14px;",
                                        placeholder: "Ex: Procedimento avulso pago via Pix no balcão",
                                        value: "{charge_notes}",
                                        oninput: move |e| charge_notes.set(e.value()),
                                    }
                                }
                            }

                            div { class: "modal-footer", style: "padding: 16px 24px; background: #f8fafc; border-top: 1px solid #e2e8f0; display: flex; justify-content: flex-end; gap: 10px;",
                                button { class: "btn-secondary", style: "height: 40px; padding: 0 18px;", onclick: move |_| is_charge_modal_open.set(false), "Cancelar" }
                                button {
                                    class: "btn-primary",
                                    style: "height: 40px; padding: 0 20px; display: flex; align-items: center; gap: 6px;",
                                    disabled: is_charging(),
                                    onclick: handle_confirm_charge,
                                    IconCheck { size: 16, color: "#ffffff".to_string() }
                                    span { if is_charging() { "Lançando..." } else { "Confirmar Lançamento" } }
                                }
                            }
                        }
                    }
                }
            }

            // Modal de Exclusão de Procedimento Clínico
            if is_delete_proc_modal_open() {
                div { class: "modal-overlay", onclick: move |_| is_delete_proc_modal_open.set(false),
                    div { class: "action-modal delete-modal-card", onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            h2 { class: "modal-title text-danger font-bold", "Excluir Registro de Procedimento" }
                            button { class: "modal-close", onclick: move |_| is_delete_proc_modal_open.set(false), "×" }
                        }
                        div { class: "modal-body",
                            if let Some(ref proc) = *delete_proc_target.read() {
                                p { "Tem certeza que deseja excluir o registro de ", strong { "{proc.procedure_name}" }, "?" }
                                p { class: "text-muted font-xs mt-2", "Esta ação removerá o procedimento do histórico de evolução do paciente." }
                            }
                        }
                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_delete_proc_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-danger",
                                disabled: is_deleting(),
                                onclick: handle_confirm_delete_proc,
                                if is_deleting() { "Excluindo..." } else { "Confirmar Exclusão" }
                            }
                        }
                    }
                }
            }
        }
    }
}
