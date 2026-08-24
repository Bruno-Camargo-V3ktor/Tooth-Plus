//! # Modelos de Domínio - Gestão de Estoque e Suprimentos
//!
//! Este módulo define itens de consumo odontológico, controle de lotes,
//! datas de validade, fornecedores e movimentações de entrada e saída.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Material,
    Chemical,
    Equipment,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentStatus {
    Active,
    InMaintenance,
    Broken,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MovementType {
    PurchaseIn,
    ManualOut,
    AppointmentOut,
    Adjustment,
    Loss,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StockAlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StockAlertType {
    LowStock,
    ExpiringSoon,
    Expired,
    MaintenanceDue,
    MaintenanceOverdue,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InventoryItem {
    pub id: String,
    pub clinic_id: String,
    pub item_type: ItemType,
    pub name: String,
    pub unit_type: String,
    pub current_stock: i32,
    pub min_stock: i32,
    pub cost_price_cents: i64,
    pub manufacturer: Option<String>,
    pub attachments: Vec<String>,
    pub expiration_date: Option<String>,
    pub batch_number: Option<String>,
    pub serial_number: Option<String>,
    pub warranty_until: Option<String>,
    pub next_maintenance_date: Option<String>,
    pub equipment_status: Option<EquipmentStatus>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateInventoryItemRequest {
    pub clinic_id: String,
    pub item_type: ItemType,
    pub name: String,
    pub unit_type: String,
    pub current_stock: i32,
    pub min_stock: i32,
    pub cost_price_cents: i64,
    pub manufacturer: Option<String>,
    pub attachments: Vec<String>,
    pub expiration_date: Option<String>,
    pub batch_number: Option<String>,
    pub serial_number: Option<String>,
    pub warranty_until: Option<String>,
    pub next_maintenance_date: Option<String>,
    pub equipment_status: Option<EquipmentStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdateInventoryItemRequest {
    pub clinic_id: String,
    pub item_type: ItemType,
    pub name: String,
    pub unit_type: String,
    pub current_stock: i32,
    pub min_stock: i32,
    pub cost_price_cents: i64,
    pub manufacturer: Option<String>,
    pub attachments: Vec<String>,
    pub expiration_date: Option<String>,
    pub batch_number: Option<String>,
    pub serial_number: Option<String>,
    pub warranty_until: Option<String>,
    pub next_maintenance_date: Option<String>,
    pub equipment_status: Option<EquipmentStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StockMovement {
    pub id: String,
    pub item_id: String,
    pub item_name: Option<String>,
    pub clinic_id: String,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub quantity_change: i32,
    pub movement_type: MovementType,
    pub unit_cost_cents: Option<i64>,
    pub invoice_number: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateStockMovementRequest {
    pub clinic_id: String,
    pub item_id: String,
    pub quantity_change: i32,
    pub movement_type: MovementType,
    pub unit_cost_cents: Option<i64>,
    pub invoice_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StockAlertItem {
    pub id: String,
    pub item_id: String,
    pub item_name: String,
    pub item_type: ItemType,
    pub alert_type: StockAlertType,
    pub severity: StockAlertSeverity,
    pub title: String,
    pub message: String,
    pub current_value: String,
    pub target_value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct StockKPIs {
    pub total_items_count: usize,
    pub materials_count: usize,
    pub chemicals_count: usize,
    pub equipments_count: usize,
    pub total_inventory_value_cents: i64,
    pub low_stock_alerts_count: usize,
    pub expiring_alerts_count: usize,
    pub maintenance_alerts_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StockResponse {
    pub kpis: StockKPIs,
    pub items: Vec<InventoryItem>,
    pub alerts: Vec<StockAlertItem>,
    pub recent_movements: Vec<StockMovement>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StockQuery {
    pub clinic_id: String,
    pub item_type: Option<String>,
    pub search: Option<String>,
}
