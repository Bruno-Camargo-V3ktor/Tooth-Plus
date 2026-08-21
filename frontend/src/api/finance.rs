use super::API_BASE;
use reqwest::Client;
use shared::finance::{CreateTransactionRequest, FinanceResponse, UpdateTransactionStatusRequest};

fn get_client() -> Client {
    Client::new()
}

pub async fn fetch_finance_data(
    token: &str,
    clinic_id: &str,
    month: Option<u32>,
    year: Option<i32>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<FinanceResponse, String> {
    let mut url = format!("{}/finance?clinic_id={}", API_BASE, clinic_id);
    if let Some(m) = month {
        url.push_str(&format!("&month={}", m));
    }
    if let Some(y) = year {
        url.push_str(&format!("&year={}", y));
    }
    if let Some(ref s) = start_date {
        url.push_str(&format!("&start_date={}", s));
    }
    if let Some(ref e) = end_date {
        url.push_str(&format!("&end_date={}", e));
    }

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão com o servidor financeiro.".to_string())?;

    if res.status().is_success() {
        res.json::<FinanceResponse>()
            .await
            .map_err(|_| "Erro ao processar dados financeiros.".into())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(if err_text.is_empty() {
            "Falha ao carregar finanças.".into()
        } else {
            err_text
        })
    }
}

pub async fn create_transaction(
    token: &str,
    req: CreateTransactionRequest,
) -> Result<String, String> {
    let url = format!("{}/finance", API_BASE);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha ao enviar transação financeira.".to_string())?;

    if res.status().is_success() {
        Ok("Lançamento cadastrado com sucesso.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao registrar movimentação.".into()
        } else {
            err
        })
    }
}

pub async fn update_transaction_status(
    token: &str,
    clinic_id: &str,
    id: &str,
    req: UpdateTransactionStatusRequest,
) -> Result<(), String> {
    let url = format!("{}/finance/{}/status?clinic_id={}", API_BASE, id, clinic_id);

    let res = get_client()
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha ao atualizar status da transação.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao modificar status do lançamento.".into()
        } else {
            err
        })
    }
}

pub async fn delete_transaction(token: &str, clinic_id: &str, id: &str) -> Result<(), String> {
    let url = format!("{}/finance/{}?clinic_id={}", API_BASE, id, clinic_id);

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha ao comunicar exclusão da transação.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao excluir transação financeira.".into()
        } else {
            err
        })
    }
}

pub async fn register_transaction_payment(
    token: &str,
    tx_id: &str,
    req: shared::finance::RegisterPaymentRequest,
) -> Result<shared::finance::Transaction, String> {
    let url = format!("{}/finance/{}/pay", API_BASE, tx_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha ao registrar pagamento financeiro.".to_string())?;

    if res.status().is_success() {
        res.json::<shared::finance::Transaction>()
            .await
            .map_err(|_| "Erro ao processar resposta do pagamento.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao registrar pagamento.".into()
        } else {
            err
        })
    }
}
