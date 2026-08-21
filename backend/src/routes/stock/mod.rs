//! # Módulo de Estoque e Suprimentos Odontológicos (Backend)
//!
//! Agrega sub-módulos para cadastro de materiais e equipamentos, controle de alertas de
//! estoque baixo e vencimento de químicos, movimentações de entrada/saída e upload de notas fiscais/laudos.

pub mod items;
pub mod movements;
pub mod uploads;

pub use items::*;
pub use movements::*;
pub use uploads::*;

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use shared::stock::{
    EquipmentStatus, InventoryItem, ItemType, MovementType, StockAlertItem,
    StockAlertSeverity, StockAlertType,
};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

/// Linha da tabela `inventory_item` no banco de dados SurrealDB.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbInventoryItemRow {
    pub id: RecordId,
    pub clinic_id: RecordId,
    pub item_type: String,
    pub name: String,
    pub unit_type: String,
    pub current_stock: i32,
    pub min_stock: i32,
    pub cost_price_cents: i64,
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub attachments: Vec<String>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub batch_number: Option<String>,
    pub serial_number: Option<String>,
    pub warranty_until: Option<DateTime<Utc>>,
    pub next_maintenance_date: Option<DateTime<Utc>>,
    pub equipment_status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Linha da tabela `stock_movement` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbStockMovementRow {
    pub id: RecordId,
    pub item_id: RecordId,
    pub clinic_id: RecordId,
    pub user_id: Option<RecordId>,
    pub quantity_change: i32,
    pub movement_type: String,
    pub unit_cost_cents: Option<i64>,
    pub invoice_number: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Converte string em `RecordId`.
pub(crate) fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else if let Some(stripped) = raw.strip_prefix(&format!("{}s:", table)) {
        stripped
    } else if let Some(pos) = raw.find(':') {
        &raw[pos + 1..]
    } else {
        raw
    };
    let clean_key = key.trim_matches(|c| c == '⟨' || c == '⟩');
    RecordId::new(table, clean_key)
}

/// Normaliza ID de clínica para `clinic:UUID`.
pub(crate) fn clinic_record_id(id: &str) -> String {
    if id.starts_with("clinic:") {
        id.to_string()
    } else {
        format!("clinic:{}", id)
    }
}

/// Converte string em enum `ItemType`.
pub(crate) fn parse_item_type(s: &str) -> ItemType {
    match s {
        "chemical" => ItemType::Chemical,
        "equipment" => ItemType::Equipment,
        _ => ItemType::Material,
    }
}

/// Converte string em enum `EquipmentStatus`.
pub(crate) fn parse_equipment_status(s: &str) -> EquipmentStatus {
    match s {
        "in_maintenance" => EquipmentStatus::InMaintenance,
        "broken" => EquipmentStatus::Broken,
        _ => EquipmentStatus::Active,
    }
}

/// Converte string em enum `MovementType`.
pub(crate) fn parse_movement_type(s: &str) -> MovementType {
    match s {
        "manual_out" => MovementType::ManualOut,
        "appointment_out" => MovementType::AppointmentOut,
        "adjustment" => MovementType::Adjustment,
        "loss" => MovementType::Loss,
        _ => MovementType::PurchaseIn,
    }
}

/// Converte a linha de banco de dados `DbInventoryItemRow` no modelo compartilhado `InventoryItem`.
pub(crate) fn map_db_item(row: DbInventoryItemRow) -> InventoryItem {
    InventoryItem {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        item_type: parse_item_type(&row.item_type),
        name: row.name,
        unit_type: row.unit_type,
        current_stock: row.current_stock,
        min_stock: row.min_stock,
        cost_price_cents: row.cost_price_cents,
        manufacturer: row.manufacturer,
        attachments: row.attachments,
        expiration_date: row.expiration_date.map(|d| d.to_rfc3339()),
        batch_number: row.batch_number,
        serial_number: row.serial_number,
        warranty_until: row.warranty_until.map(|d| d.to_rfc3339()),
        next_maintenance_date: row.next_maintenance_date.map(|d| d.to_rfc3339()),
        equipment_status: row.equipment_status.as_deref().map(parse_equipment_status),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.map(|d| d.to_rfc3339()),
    }
}

/// Calcula alertas automáticos de estoque baixo, vencimento e manutenções preventivas.
pub(crate) fn calculate_alerts(items: &[InventoryItem]) -> Vec<StockAlertItem> {
    let mut alerts = Vec::new();
    let now = Utc::now();
    let thirty_days_future = now + Duration::days(30);

    for item in items {
        if item.item_type != ItemType::Equipment && item.current_stock <= item.min_stock {
            let is_critical = item.current_stock <= (item.min_stock / 2);
            alerts.push(StockAlertItem {
                id: format!("alert_low_{}", item.id),
                item_id: item.id.clone(),
                item_name: item.name.clone(),
                item_type: item.item_type,
                alert_type: StockAlertType::LowStock,
                severity: if is_critical {
                    StockAlertSeverity::Critical
                } else {
                    StockAlertSeverity::Warning
                },
                title: "Estoque Abaixo do Mínimo".into(),
                message: format!(
                    "Item '{}' atingiu nível de alerta (Estoque: {} {}, Mínimo: {} {}).",
                    item.name, item.current_stock, item.unit_type, item.min_stock, item.unit_type
                ),
                current_value: format!("{} {}", item.current_stock, item.unit_type),
                target_value: format!("{} {}", item.min_stock, item.unit_type),
            });
        }

        if item.item_type == ItemType::Chemical {
            if let Some(ref exp_str) = item.expiration_date {
                if let Ok(exp_dt) = DateTime::parse_from_rfc3339(exp_str) {
                    let exp_utc = exp_dt.with_timezone(&Utc);
                    if exp_utc < now {
                        alerts.push(StockAlertItem {
                            id: format!("alert_exp_over_{}", item.id),
                            item_id: item.id.clone(),
                            item_name: item.name.clone(),
                            item_type: item.item_type,
                            alert_type: StockAlertType::Expired,
                            severity: StockAlertSeverity::Critical,
                            title: "Produto Vencido".into(),
                            message: format!(
                                "Produto químico '{}' venceu em {}. Descarte ou reposição necessária.",
                                item.name,
                                exp_utc.format("%d/%m/%Y")
                            ),
                            current_value: "Vencido".into(),
                            target_value: exp_utc.format("%d/%m/%Y").to_string(),
                        });
                    } else if exp_utc <= thirty_days_future {
                        let days_left = (exp_utc - now).num_days();
                        alerts.push(StockAlertItem {
                            id: format!("alert_exp_soon_{}", item.id),
                            item_id: item.id.clone(),
                            item_name: item.name.clone(),
                            item_type: item.item_type,
                            alert_type: StockAlertType::ExpiringSoon,
                            severity: StockAlertSeverity::Warning,
                            title: "Validade Próxima".into(),
                            message: format!(
                                "Produto químico '{}' vence em {} dias ({}).",
                                item.name,
                                days_left,
                                exp_utc.format("%d/%m/%Y")
                            ),
                            current_value: format!("{} dias restantes", days_left),
                            target_value: exp_utc.format("%d/%m/%Y").to_string(),
                        });
                    }
                }
            }
        }

        if item.item_type == ItemType::Equipment {
            if let Some(ref maint_str) = item.next_maintenance_date {
                if let Ok(maint_dt) = DateTime::parse_from_rfc3339(maint_str) {
                    let maint_utc = maint_dt.with_timezone(&Utc);
                    if maint_utc < now {
                        alerts.push(StockAlertItem {
                            id: format!("alert_maint_over_{}", item.id),
                            item_id: item.id.clone(),
                            item_name: item.name.clone(),
                            item_type: item.item_type,
                            alert_type: StockAlertType::MaintenanceOverdue,
                            severity: StockAlertSeverity::Critical,
                            title: "Manutenção Preventiva Atrasada".into(),
                            message: format!(
                                "Equipamento '{}' estava agendado para revisão em {}.",
                                item.name,
                                maint_utc.format("%d/%m/%Y")
                            ),
                            current_value: "Atrasada".into(),
                            target_value: maint_utc.format("%d/%m/%Y").to_string(),
                        });
                    } else if maint_utc <= thirty_days_future {
                        let days_left = (maint_utc - now).num_days();
                        alerts.push(StockAlertItem {
                            id: format!("alert_maint_soon_{}", item.id),
                            item_id: item.id.clone(),
                            item_name: item.name.clone(),
                            item_type: item.item_type,
                            alert_type: StockAlertType::MaintenanceDue,
                            severity: StockAlertSeverity::Warning,
                            title: "Revisão Preventiva Próxima".into(),
                            message: format!(
                                "Equipamento '{}' deve passar por revisão preventiva em {} dias ({}).",
                                item.name,
                                days_left,
                                maint_utc.format("%d/%m/%Y")
                            ),
                            current_value: format!("Em {} dias", days_left),
                            target_value: maint_utc.format("%d/%m/%Y").to_string(),
                        });
                    }
                }
            }

            if item.equipment_status == Some(EquipmentStatus::InMaintenance) {
                alerts.push(StockAlertItem {
                    id: format!("alert_eq_in_maint_{}", item.id),
                    item_id: item.id.clone(),
                    item_name: item.name.clone(),
                    item_type: item.item_type,
                    alert_type: StockAlertType::MaintenanceDue,
                    severity: StockAlertSeverity::Warning,
                    title: "Equipamento em Manutenção".into(),
                    message: format!(
                        "O equipamento '{}' está atualmente em assistência técnica.",
                        item.name
                    ),
                    current_value: "Em Manutenção".into(),
                    target_value: "Aguardando Retorno".into(),
                });
            } else if item.equipment_status == Some(EquipmentStatus::Broken) {
                alerts.push(StockAlertItem {
                    id: format!("alert_eq_broken_{}", item.id),
                    item_id: item.id.clone(),
                    item_name: item.name.clone(),
                    item_type: item.item_type,
                    alert_type: StockAlertType::MaintenanceOverdue,
                    severity: StockAlertSeverity::Critical,
                    title: "Equipamento Inoperante".into(),
                    message: format!(
                        "O equipamento '{}' está registrado como danificado/inoperante.",
                        item.name
                    ),
                    current_value: "Inoperante".into(),
                    target_value: "Reparo Urgente".into(),
                });
            }
        }
    }

    alerts
}
