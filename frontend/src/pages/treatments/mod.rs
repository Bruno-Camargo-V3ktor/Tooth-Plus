pub mod components;

use crate::api::treatments::TreatmentsApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use shared::treatments::{CreateTreatmentTemplateRequest, TreatmentTemplate};
use dioxus::prelude::*;

pub use components::{TemplateGrid, TemplateModal, TreatmentToolbar};

const STYLE: Asset = asset!("/src/pages/treatments/style.css");

#[component]
pub fn TreatmentsView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut templates = use_signal(Vec::<TreatmentTemplate>::new);
    let mut search_query = use_signal(String::new);
    let mut category_filter = use_signal(|| "ALL".to_string());
    let mut show_modal = use_signal(|| false);
    let mut reload_trigger = use_signal(|| 0);

    // Form fields
    let mut name = use_signal(String::new);
    let mut category = use_signal(|| "Dentística".to_string());
    let mut description = use_signal(String::new);
    let mut price_str = use_signal(|| "150.00".to_string());
    let mut duration_str = use_signal(|| "30".to_string());

    let cid_effect = clinic_id.clone();
    use_effect(move || {
        let _ = reload_trigger.read();
        let cid = cid_effect.clone();
        spawn(async move {
            if let Ok(list) = TreatmentsApi::list_templates(&cid).await {
                templates.set(list);
            }
        });
    });

    let handle_submit = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut modal_sig = show_modal;
        let n_s = name.clone();
        let c_s = category.clone();
        let d_s = description.clone();
        let p_s = price_str.clone();
        let dur_s = duration_str.clone();
        let mut reload_sig = reload_trigger;

        move |_| {
            let n = n_s.read().trim().to_string();
            if n.is_empty() {
                toast_c.show("Informe o nome do procedimento.", ToastVariant::Error);
                return;
            }

            let price_num: f64 = p_s.read().replace(',', ".").parse().unwrap_or(0.0);
            let dur_num: i32 = dur_s.read().parse().unwrap_or(30);

            let req = CreateTreatmentTemplateRequest {
                clinic_id: cid.clone(),
                name: n,
                category: Some(c_s.read().clone()),
                description: if d_s.read().is_empty() { None } else { Some(d_s.read().clone()) },
                default_price_cents: (price_num * 100.0) as i64,
                estimated_duration_minutes: Some(dur_num),
                dental_regions: vec![],
                target_teeth: vec![],
                required_materials: vec![],
                required_equipment: vec![],
                post_care_instructions: None,
                clinical_notes: None,
            };

            let mut toast_resp = toast_c.clone();
            let mut modal_c = modal_sig;
            let mut reload_c = reload_sig;

            spawn(async move {
                match TreatmentsApi::create_template(req).await {
                    Ok(_) => {
                        toast_resp.show("Procedimento salvo no catálogo!", ToastVariant::Success);
                        modal_c.set(false);
                        reload_c.set(reload_c() + 1);
                    }
                    Err(err) => toast_resp.show(err, ToastVariant::Error),
                }
            });
        }
    };

    let filtered_templates: Vec<TreatmentTemplate> = templates.read().iter().filter(|t| {
        let cat = category_filter.read().clone();
        if cat != "ALL" && t.category.as_deref() != Some(&cat) { return false; }
        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() { return true; }
        t.name.to_lowercase().contains(&q)
    }).cloned().collect();

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "treatments-page",
            div { class: "treatments-header-row",
                div {
                    h1 { class: "treatments-title", "Catálogo de Procedimentos & Tratamentos" }
                    p { style: "font-size: 13.5px; color: #94a3b8; margin: 4px 0 0 0;",
                        "Gerencie a tabela de procedimentos padrão, valores sugeridos e tempos de cadeira da clínica."
                    }
                }
            }

            TreatmentToolbar {
                search_query,
                category_filter,
                on_new_template: move |_| {
                    name.set(String::new());
                    description.set(String::new());
                    price_str.set("150.00".to_string());
                    show_modal.set(true);
                },
            }

            TemplateGrid {
                templates: filtered_templates,
                on_edit: move |_tid: String| {
                    show_modal.set(true);
                },
                on_delete: move |tid: String| {
                    let mut toast_d = toast.clone();
                    let mut reload_c = reload_trigger;
                    spawn(async move {
                        if let Ok(_) = TreatmentsApi::delete_template(&tid).await {
                            toast_d.show("Procedimento removido do catálogo.", ToastVariant::Success);
                            reload_c.set(reload_c() + 1);
                        }
                    });
                },
            }

            TemplateModal {
                is_open: show_modal(),
                name,
                category,
                description,
                price_str,
                duration_str,
                on_close: move |_| show_modal.set(false),
                on_submit: handle_submit,
            }
        }
    }
}
