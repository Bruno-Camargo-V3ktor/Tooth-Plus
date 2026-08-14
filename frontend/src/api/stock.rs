use super::API_BASE;
use reqwest::Client;
use shared::stock::{
    CreateInventoryItemRequest, CreateStockMovementRequest, InventoryItem, StockMovement,
    StockResponse, UpdateInventoryItemRequest,
};

fn get_client() -> Client {
    Client::new()
}

pub async fn fetch_stock_data(
    token: &str,
    clinic_id: &str,
    item_type: Option<&str>,
    search: Option<&str>,
) -> Result<StockResponse, String> {
    let mut url = format!("{}/stock?clinic_id={}", API_BASE, clinic_id);
    if let Some(t) = item_type {
        if !t.is_empty() && t != "all" {
            url.push_str(&format!("&item_type={}", t));
        }
    }
    if let Some(s) = search {
        if !s.is_empty() {
            url.push_str(&format!("&search={}", s));
        }
    }

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão com o servidor de estoque.".to_string())?;

    if res.status().is_success() {
        res.json::<StockResponse>()
            .await
            .map_err(|_| "Erro ao processar dados do estoque.".into())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(if err_text.is_empty() {
            "Falha ao carregar itens do estoque.".into()
        } else {
            err_text
        })
    }
}

pub async fn create_stock_item(
    token: &str,
    req: CreateInventoryItemRequest,
) -> Result<InventoryItem, String> {
    let url = format!("{}/stock", API_BASE);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha ao cadastrar item no estoque.".to_string())?;

    if res.status().is_success() {
        res.json::<InventoryItem>()
            .await
            .map_err(|_| "Item cadastrado, mas falha ao ler resposta.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao registrar item no estoque.".into()
        } else {
            err
        })
    }
}

pub async fn update_stock_item(
    token: &str,
    id: &str,
    req: UpdateInventoryItemRequest,
) -> Result<InventoryItem, String> {
    let url = format!("{}/stock/{}", API_BASE, id);

    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha ao enviar atualização do item.".to_string())?;

    if res.status().is_success() {
        res.json::<InventoryItem>()
            .await
            .map_err(|_| "Item atualizado, mas falha ao ler resposta.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao atualizar item de estoque.".into()
        } else {
            err
        })
    }
}

pub async fn delete_stock_item(
    token: &str,
    clinic_id: &str,
    id: &str,
) -> Result<(), String> {
    let url = format!("{}/stock/{}?clinic_id={}", API_BASE, id, clinic_id);

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha ao comunicar exclusão do item.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao excluir item do estoque.".into()
        } else {
            err
        })
    }
}

pub async fn create_stock_movement(
    token: &str,
    id: &str,
    req: CreateStockMovementRequest,
) -> Result<StockMovement, String> {
    let url = format!("{}/stock/{}/movement", API_BASE, id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha ao registrar movimentação de estoque.".to_string())?;

    if res.status().is_success() {
        res.json::<StockMovement>()
            .await
            .map_err(|_| "Movimentação registrada com sucesso.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao registrar movimentação.".into()
        } else {
            err
        })
    }
}

pub async fn upload_stock_document(
    token: &str,
    clinic_id: &str,
    req: shared::files::FileUploadRequest,
) -> Result<String, String> {
    let url = format!("{}/stock/{}/upload", API_BASE, clinic_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão no envio do anexo.".to_string())?;

    if res.status().is_success() {
        #[derive(serde::Deserialize)]
        struct UploadRes {
            url: String,
        }
        let body: UploadRes = res
            .json()
            .await
            .map_err(|_| "Erro ao processar URL do arquivo.".to_string())?;
        Ok(body.url)
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Falha ao enviar arquivo.".to_string()
        } else {
            err
        })
    }
}
