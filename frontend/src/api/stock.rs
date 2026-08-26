//! # Módulo de Integração e Serviço de Estoque (StockApi)

use super::mock_db::DB;
use shared::stock::{
    CreateInventoryItemRequest, CreateStockMovementRequest, InventoryItem, ItemType,
    StockAlertItem, StockAlertSeverity, StockAlertType, StockKPIs, StockMovement, StockQuery,
    StockResponse,
};

pub struct StockApi;

impl StockApi {
    /// Consulta itens do estoque, alertas de validade/mínimo e extrato recente.
    pub async fn list_stock(query: StockQuery) -> Result<StockResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        let items: Vec<InventoryItem> = db
            .inventory_items
            .iter()
            .filter(|i| i.clinic_id == query.clinic_id)
            .cloned()
            .collect();

        let mut materials_count = 0;
        let mut chemicals_count = 0;
        let mut equipments_count = 0;
        let mut total_val_cents = 0;
        let mut alerts = vec![];

        for item in &items {
            total_val_cents += (item.current_stock as i64) * item.cost_price_cents;
            match item.item_type {
                ItemType::Material => materials_count += 1,
                ItemType::Chemical => chemicals_count += 1,
                ItemType::Equipment => equipments_count += 1,
            }

            if item.current_stock <= item.min_stock {
                alerts.push(StockAlertItem {
                    id: format!("alert:low:{}", item.id),
                    item_id: item.id.clone(),
                    item_name: item.name.clone(),
                    item_type: item.item_type,
                    alert_type: StockAlertType::LowStock,
                    severity: StockAlertSeverity::Critical,
                    title: "Estoque Mínimo Atingido".to_string(),
                    message: format!("Apenas {} {} restantes (Mínimo: {})", item.current_stock, item.unit_type, item.min_stock),
                    current_value: format!("{} {}", item.current_stock, item.unit_type),
                    target_value: format!("{} {}", item.min_stock, item.unit_type),
                });
            }
        }

        let kpis = StockKPIs {
            total_items_count: items.len(),
            materials_count,
            chemicals_count,
            equipments_count,
            total_inventory_value_cents: total_val_cents,
            low_stock_alerts_count: alerts.len(),
            expiring_alerts_count: 0,
            maintenance_alerts_count: 0,
        };

        Ok(StockResponse {
            kpis,
            items,
            alerts,
            recent_movements: db.stock_movements.clone(),
        })
    }

    /// Cadastra um novo item de estoque.
    pub async fn create_item(req: CreateInventoryItemRequest) -> Result<InventoryItem, String> {
        gloo_timers::future::TimeoutFuture::new(200).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let new_item = InventoryItem {
            id: format!("item:{}", db.inventory_items.len() + 1),
            clinic_id: req.clinic_id,
            item_type: req.item_type,
            name: req.name,
            unit_type: req.unit_type,
            current_stock: req.current_stock,
            min_stock: req.min_stock,
            cost_price_cents: req.cost_price_cents,
            manufacturer: req.manufacturer,
            attachments: req.attachments,
            expiration_date: req.expiration_date,
            batch_number: req.batch_number,
            serial_number: req.serial_number,
            warranty_until: req.warranty_until,
            next_maintenance_date: req.next_maintenance_date,
            equipment_status: req.equipment_status,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        };

        db.inventory_items.push(new_item.clone());
        Ok(new_item)
    }

    /// Registra movimentação de entrada/saída de estoque.
    pub async fn create_movement(req: CreateStockMovementRequest) -> Result<StockMovement, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let item_name = {
            let item = db
                .inventory_items
                .iter_mut()
                .find(|i| i.id == req.item_id)
                .ok_or_else(|| format!("Item {} não encontrado.", req.item_id))?;

            item.current_stock = (item.current_stock + req.quantity_change).max(0);
            item.updated_at = Some(chrono::Utc::now().to_rfc3339());
            item.name.clone()
        };

        let mov_id = format!("mov:{}", db.stock_movements.len() + 1);

        let movement = StockMovement {
            id: mov_id,
            item_id: req.item_id,
            item_name: Some(item_name),
            clinic_id: req.clinic_id,
            user_id: Some("user:admin_principal".to_string()),
            user_name: Some("Dr. Roberto Alencar".to_string()),
            quantity_change: req.quantity_change,
            movement_type: req.movement_type,
            unit_cost_cents: req.unit_cost_cents,
            invoice_number: req.invoice_number,
            notes: req.notes,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        db.stock_movements.insert(0, movement.clone());
        Ok(movement)
    }
}

impl StockApi {
    pub async fn update_item(id: &str, req: shared::stock::UpdateInventoryItemRequest) -> Result<InventoryItem, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;
        let item = db.inventory_items.iter_mut().find(|i| i.id == id).ok_or_else(|| "Item não encontrado.".to_string())?;

        item.item_type = req.item_type;
        item.name = req.name;
        item.unit_type = req.unit_type;
        item.current_stock = req.current_stock;
        item.min_stock = req.min_stock;
        item.cost_price_cents = req.cost_price_cents;
        item.manufacturer = req.manufacturer;
        item.expiration_date = req.expiration_date;
        item.batch_number = req.batch_number;
        item.updated_at = Some(chrono::Utc::now().to_rfc3339());

        Ok(item.clone())
    }

    pub async fn delete_item(id: &str) -> Result<(), String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;
        db.inventory_items.retain(|i| i.id != id);
        Ok(())
    }
}
