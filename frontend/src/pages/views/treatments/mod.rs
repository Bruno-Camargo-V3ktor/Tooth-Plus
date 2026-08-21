//! # Módulo de Tratamentos Padrão (Catálogo da Clínica)
//!
//! Permite cadastrar e gerenciar procedimentos odontológicos padronizados,
//! com valor sugerido, dentes/regiões alvo, materiais e equipamentos integrados ao estoque,
//! para reutilização ágil nos orçamentos dos pacientes.

pub mod template_modal;

pub use template_modal::*;

use crate::api::{delete_treatment_template, fetch_treatment_templates};
use crate::components::icons::{
    IconBox, IconEdit, IconFilter, IconPlus, IconRefresh, IconSearch, IconTool, IconTooth,
    IconTrash,
};
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::treatments::TreatmentTemplate;

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

fn category_badge_class(cat: &str) -> &'static str {
    match cat {
        "Cirurgia" => "stock-tag tag-cirurgia",
        "Endodontia" => "stock-tag tag-endodontia",
        "Ortodontia" => "stock-tag tag-ortodontia",
        "Periodontia" => "stock-tag tag-periodontia",
        "Prótese" => "stock-tag tag-protese",
        "Estética" => "stock-tag tag-estetica",
        "Implantodontia" => "stock-tag tag-implante",
        "Odontopediatria" => "stock-tag tag-pediatria",
        "Dentística" => "stock-tag tag-dentistica",
        _ => "stock-tag tag-material",
    }
}

#[component]
pub fn TreatmentsView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = permissions::has_permission(&sess, &clinic, "treatments:read");
    let can_write = permissions::has_permission(&sess, &clinic, "treatments:write");
    let can_delete = permissions::has_permission(&sess, &clinic, "treatments:delete") || can_write;

    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();
    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar o catálogo de procedimentos." }
            }
        };
    }

    let mut selected_category = use_signal(|| "all".to_string());
    let mut search_query = use_signal(String::new);
    let mut reload_trigger = use_signal(|| 0usize);
    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    // Modal state
    let mut is_modal_open = use_signal(|| false);
    let mut editing_template = use_signal(|| None::<TreatmentTemplate>);
    let mut delete_target_item = use_signal(|| None::<TreatmentTemplate>);
    let mut is_delete_modal_open = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    let tok_res = token.clone();
    let cid_res = clinic_id.clone();
    let templates_resource = use_resource(move || {
        let t = tok_res.clone();
        let cid = cid_res.clone();
        let _ = reload_trigger();
        async move {
            if t.is_empty() || cid.is_empty() {
                return Ok(vec![]);
            }
            fetch_treatment_templates(&t, &cid).await
        }
    });

    let (templates_list, is_loading) = match &*templates_resource.read() {
        Some(Ok(items)) => (items.clone(), false),
        _ => (vec![], true),
    };

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

    let total_templates = templates_list.len();
    let total_value_cents: i64 = templates_list.iter().map(|t| t.default_price_cents).sum();
    let avg_price_cents = if total_templates > 0 {
        total_value_cents / total_templates as i64
    } else {
        0
    };

    let total_stock_linked_items: usize = templates_list
        .iter()
        .map(|t| t.required_materials.len() + t.required_equipment.len())
        .sum();

    let active_categories_count = categories
        .iter()
        .filter(|cat| templates_list.iter().any(|t| t.category.as_deref() == Some(*cat)))
        .count();

    let q = search_query().trim().to_lowercase();
    let filtered_templates: Vec<TreatmentTemplate> = templates_list
        .iter()
        .filter(|t| {
            let cat = selected_category();
            let matches_cat = cat == "all" || t.category.as_deref() == Some(&cat);

            let matches_search = if q.is_empty() {
                true
            } else {
                t.name.to_lowercase().contains(&q)
                    || t.category.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || t.description.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || t.target_teeth.iter().any(|tooth| tooth.to_lowercase().contains(&q))
                    || t.dental_regions.iter().any(|reg| reg.to_lowercase().contains(&q))
                    || t.required_materials.iter().any(|mat| mat.to_lowercase().contains(&q))
                    || t.required_equipment.iter().any(|eq| eq.to_lowercase().contains(&q))
            };

            matches_cat && matches_search
        })
        .cloned()
        .collect();

    let tok_del = token.clone();
    let cid_del = clinic_id.clone();
    let mut handle_confirm_delete = move |_| {
        let Some(ref target) = *delete_target_item.read() else {
            return;
        };
        let target_id = target.id.clone();
        let t = tok_del.clone();
        let c = cid_del.clone();
        let mut is_del = is_deleting;
        let mut open_del = is_delete_modal_open;
        let mut target_sig = delete_target_item;
        let mut rel = reload_trigger;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        is_del.set(true);
        spawn(async move {
            let res = delete_treatment_template(&t, &c, &target_id).await;
            is_del.set(false);
            open_del.set(false);
            target_sig.set(None);
            match res {
                Ok(_) => {
                    toast.set(Some("Procedimento removido do catálogo com sucesso!".into()));
                    rel.set(rel() + 1);
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir procedimento: {}", e)));
                }
            }
        });
    };

    rsx! {
        div { class: "documents-view-container",
            // 1. Notificações Toast
            if let Some(ref msg) = *toast_msg.read() {
                div { class: "toast toast-success",
                    span { "{msg}" }
                    button { class: "toast-close", onclick: move |_| toast_msg.set(None), "✕" }
                }
            }
            if let Some(ref err) = *error_toast.read() {
                div { class: "toast toast-error",
                    span { "{err}" }
                    button { class: "toast-close", onclick: move |_| error_toast.set(None), "✕" }
                }
            }

            // 2. Compact Horizontal KPIs (Igual ao Estoque)
            div { class: "agenda-kpi-row",
                // 1. TOTAL DE PROCEDIMENTOS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                        IconTooth { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Total de Procedimentos" }
                        span { class: "agenda-kpi-sublbl", "{total_templates} cadastrados no catálogo" }
                    }
                    div { class: "agenda-kpi-val", "{total_templates}" }
                }

                // 2. VALOR MÉDIO TABELADO
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                        IconBox { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Valor Médio Tabelado" }
                        span { class: "agenda-kpi-sublbl", "Preço base sugerido" }
                    }
                    div { class: "agenda-kpi-val kpi-pending", "{format_currency(avg_price_cents)}" }
                }

                // 3. ESPECIALIDADES ATIVAS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                        IconTool { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Especialidades Ativas" }
                        span { class: "agenda-kpi-sublbl", "{active_categories_count} áreas com procedimentos" }
                    }
                    div { class: "agenda-kpi-val kpi-completed", "{active_categories_count}" }
                }

                // 4. INSUMOS & EQUIPAMENTOS SINCRONIZADOS
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-progress",
                        IconBox { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Insumos & Equipamentos" }
                        span { class: "agenda-kpi-sublbl", "Vínculos ativos com o estoque" }
                    }
                    div { class: "agenda-kpi-val kpi-progress", "{total_stock_linked_items}" }
                }
            }

            // 3. View Toolbar (Busca + Filtro de Especialidade com ícone SVG limpo + Ações)
            div { class: "view-toolbar",
                div { class: "search-input-wrap",
                    IconSearch { size: 18, color: "#94a3b8".to_string() }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Buscar por procedimento, dente, material ou especialidade...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                // Botão/Dropdown de Filtro de Especialidades Limpo e Elegante
                div { class: "toolbar-filter-select-wrap",
                    IconFilter { size: 16, color: "#64748b".to_string() }
                    select {
                        class: "toolbar-specialty-select",
                        value: "{selected_category}",
                        onchange: move |e: FormEvent| selected_category.set(e.value()),
                        option { value: "all", "Todas as Especialidades ({total_templates})" }
                        for cat in categories.iter() {
                            {
                                let cat_str = cat.to_string();
                                let count = templates_list.iter().filter(|t| t.category.as_deref() == Some(&cat_str)).count();
                                rsx! {
                                    option { value: "{cat}", "{cat} ({count})" }
                                }
                            }
                        }
                    }
                }

                div { class: "toolbar-actions",
                    button {
                        class: "btn-refresh",
                        onclick: move |_| reload_trigger.set(reload_trigger() + 1),
                        title: "Recarregar catálogo de procedimentos",
                        IconRefresh { size: 16, color: "#475569".to_string() }
                    }

                    if can_write {
                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                editing_template.set(None);
                                is_modal_open.set(true);
                            },
                            IconPlus { size: 16, color: "currentColor".to_string() }
                            span { " Novo Tratamento Padrão" }
                        }
                    }
                }
            }

            // 4. Grid de Cards de Procedimentos
            if is_loading {
                div { class: "loading-card",
                    div { class: "loading-spinner" }
                    p { "Carregando catálogo de procedimentos..." }
                }
            } else if filtered_templates.is_empty() {
                div { class: "empty-state-card",
                    IconTooth { size: 48, color: "#94a3b8".to_string() }
                    h3 { "Nenhum procedimento encontrado" }
                    p { "Cadastre novos procedimentos padrão ou ajuste os termos do filtro." }
                    if can_write {
                        button {
                            class: "btn-primary",
                            style: "margin-top: 14px;",
                            onclick: move |_| {
                                editing_template.set(None);
                                is_modal_open.set(true);
                            },
                            IconPlus { size: 16, color: "currentColor".to_string() }
                            span { " Cadastrar Primeiro Procedimento" }
                        }
                    }
                }
            } else {
                div { class: "stock-cards-grid",
                    for tmpl in filtered_templates {
                        {
                            let tmpl_edit = tmpl.clone();
                            let tmpl_del = tmpl.clone();
                            let cat_label = tmpl.category.clone().unwrap_or_else(|| "Geral".to_string());
                            let badge_cls = category_badge_class(&cat_label);

                            rsx! {
                                div { key: "{tmpl.id}", class: "stock-item-card treatment-card-clean",
                                    // Header do Card
                                    div { class: "stock-card-header",
                                        div { class: "stock-badges-group",
                                            span { class: "{badge_cls}", "{cat_label}" }
                                            if let Some(dur) = tmpl.estimated_duration_minutes {
                                                span { class: "stock-tag tag-active", "{dur} MIN" }
                                            }
                                        }

                                        div { class: "stock-card-actions",
                                            if can_write {
                                                button {
                                                    class: "stock-action-icon-btn",
                                                    title: "Editar Procedimento",
                                                    onclick: move |_| {
                                                        editing_template.set(Some(tmpl_edit.clone()));
                                                        is_modal_open.set(true);
                                                    },
                                                    IconEdit { size: 14, color: "currentColor".to_string() }
                                                }
                                            }
                                            if can_delete {
                                                button {
                                                    class: "stock-action-icon-btn btn-danger-icon",
                                                    title: "Excluir Procedimento",
                                                    onclick: move |_| {
                                                        delete_target_item.set(Some(tmpl_del.clone()));
                                                        is_delete_modal_open.set(true);
                                                    },
                                                    IconTrash { size: 14, color: "currentColor".to_string() }
                                                }
                                            }
                                        }
                                    }

                                    // Corpo do Card
                                    div { class: "stock-card-body",
                                        h3 { class: "stock-item-title", "{tmpl.name}" }
                                        if let Some(ref desc) = tmpl.description {
                                            div { class: "stock-item-manufacturer", "{desc}" }
                                        }

                                        // Dentes e Regiões
                                        if !tmpl.target_teeth.is_empty() || !tmpl.dental_regions.is_empty() {
                                            div { class: "treatment-details-pill-row",
                                                if !tmpl.target_teeth.is_empty() {
                                                    span { class: "treatment-mini-pill pill-tooth",
                                                        "🦷 Dentes: {tmpl.target_teeth.join(\", \")}"
                                                    }
                                                }
                                                if !tmpl.dental_regions.is_empty() {
                                                    span { class: "treatment-mini-pill pill-region",
                                                        "📍 {tmpl.dental_regions.join(\", \")}"
                                                    }
                                                }
                                            }
                                        }

                                        // Insumos e Equipamentos do Estoque
                                        if !tmpl.required_materials.is_empty() || !tmpl.required_equipment.is_empty() {
                                            div { class: "treatment-stock-sync-preview",
                                                if !tmpl.required_materials.is_empty() {
                                                    div { class: "stock-preview-line",
                                                        span { class: "preview-label", "📦 Insumos:" }
                                                        span { class: "preview-items", "{tmpl.required_materials.join(\", \")}" }
                                                    }
                                                }
                                                if !tmpl.required_equipment.is_empty() {
                                                    div { class: "stock-preview-line",
                                                        span { class: "preview-label", "🛠️ Equip.:" }
                                                        span { class: "preview-items", "{tmpl.required_equipment.join(\", \")}" }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Rodapé do Card com Preço Base
                                    div { class: "stock-card-footer",
                                        div { class: "stock-footer-row",
                                            span { class: "stock-footer-label", "Preço Base Sugerido:" }
                                            span { class: "stock-footer-val font-mono", "{format_currency(tmpl.default_price_cents)}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal de Cadastro / Edição
            if is_modal_open() {
                TreatmentTemplateModal {
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    editing_template: editing_template(),
                    is_open: is_modal_open,
                    reload_counter: reload_trigger,
                    toast_msg,
                    error_toast,
                }
            }

            // Modal de Confirmação de Exclusão (Estilo Estoque)
            if is_delete_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal modal-delete-confirm",
                        div { class: "settings-header",
                            div { class: "modal-header-left",
                                div { class: "stock-header-icon-box header-icon-danger",
                                    IconTrash { size: 20, color: "#dc2626".to_string() }
                                }
                                div {
                                    h2 { class: "settings-title", "Excluir Procedimento" }
                                    p { class: "settings-subtitle", "Esta ação removerá o procedimento padrão do catálogo da clínica." }
                                }
                            }
                            button { class: "close-btn", onclick: move |_| is_delete_modal_open.set(false), "×" }
                        }

                        div { class: "settings-content",
                            div { class: "delete-confirm-box",
                                if let Some(ref target) = *delete_target_item.read() {
                                    p {
                                        "Você está prestes a excluir "
                                        strong { "\"{target.name}\"" }
                                        ". Os orçamentos clínicos já criados que utilizaram este procedimento não serão afetados."
                                    }
                                }
                            }
                        }

                        div { class: "modal-actions",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                onclick: move |_| is_delete_modal_open.set(false),
                                "Cancelar"
                            }
                            button {
                                r#type: "button",
                                class: "btn-danger",
                                disabled: is_deleting(),
                                onclick: handle_confirm_delete,
                                if is_deleting() { "Excluindo..." } else { "Sim, Excluir" }
                            }
                        }
                    }
                }
            }
        }
    }
}
