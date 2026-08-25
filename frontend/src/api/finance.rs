//! # Módulo de Integração e Serviço Financeiro (FinanceApi)

use super::mock_db::DB;
use shared::finance::{
    CreateTransactionRequest, FinanceQuery, FinanceResponse, FinanceSummary, RegisterPaymentRequest,
    Transaction, TransactionDirection, TransactionPaymentEntry, TransactionStatus,
    UpdateTransactionStatusRequest,
};

pub struct FinanceApi;

impl FinanceApi {
    /// Lista transações da clínica e calcula o sumário financeiro consolidado.
    pub async fn list_transactions(query: FinanceQuery) -> Result<FinanceResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        let filtered: Vec<Transaction> = db
            .transactions
            .iter()
            .filter(|t| t.clinic_id == query.clinic_id)
            .cloned()
            .collect();

        let mut total_income_cents: i64 = 0;
        let mut total_expense_cents: i64 = 0;
        let mut pending_income_cents: i64 = 0;
        let mut pending_expense_cents: i64 = 0;

        for t in &filtered {
            match t.direction {
                TransactionDirection::Income => {
                    if t.status == TransactionStatus::Paid {
                        total_income_cents += t.paid_amount_cents;
                    } else if t.status == TransactionStatus::Pending || t.status == TransactionStatus::Partial {
                        pending_income_cents += t.remaining_amount_cents.max(t.amount_cents);
                    }
                }
                TransactionDirection::Expense => {
                    if t.status == TransactionStatus::Paid {
                        total_expense_cents += t.paid_amount_cents;
                    } else if t.status == TransactionStatus::Pending || t.status == TransactionStatus::Partial {
                        pending_expense_cents += t.remaining_amount_cents.max(t.amount_cents);
                    }
                }
            }
        }

        let summary = FinanceSummary {
            total_income_cents,
            total_expense_cents,
            net_balance_cents: total_income_cents - total_expense_cents,
            pending_income_cents,
            pending_expense_cents,
            total_transactions_count: filtered.len(),
        };

        Ok(FinanceResponse {
            summary,
            transactions: filtered,
        })
    }

    /// Cria um novo lançamento financeiro (Receita ou Despesa).
    pub async fn create_transaction(req: CreateTransactionRequest) -> Result<Transaction, String> {
        gloo_timers::future::TimeoutFuture::new(200).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let is_paid = req.status == TransactionStatus::Paid;
        let paid_amount = if is_paid { req.amount_cents } else { 0 };
        let remaining_amount = if is_paid { 0 } else { req.amount_cents };

        let new_tx = Transaction {
            id: format!("tx:{}", db.transactions.len() + 1),
            clinic_id: req.clinic_id,
            appointment_id: req.appointment_id,
            patient_id: req.patient_id,
            patient_name: req.patient_name,
            user_id: req.user_id,
            user_name: None,
            treatment_plan_id: req.treatment_plan_id,
            direction: req.direction,
            amount_cents: req.amount_cents,
            paid_amount_cents: paid_amount,
            remaining_amount_cents: remaining_amount,
            description: req.description,
            category: req.category,
            status: req.status,
            due_date: req.due_date,
            paid_date: req.paid_date,
            payment_method: req.payment_method,
            payments: vec![],
            installment_current: req.installment_current.unwrap_or(1),
            installment_total: req.installment_total.unwrap_or(1),
            is_calculated_pending: false,
        };

        db.transactions.insert(0, new_tx.clone());
        Ok(new_tx)
    }

    /// Registra quitação ou pagamento parcial de uma transação.
    pub async fn register_payment(
        transaction_id: &str,
        req: RegisterPaymentRequest,
    ) -> Result<Transaction, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let tx = db
            .transactions
            .iter_mut()
            .find(|t| t.id == transaction_id)
            .ok_or_else(|| format!("Transação {} não encontrada.", transaction_id))?;

        let entry = TransactionPaymentEntry {
            id: format!("pay:{}", tx.payments.len() + 1),
            paid_at: req.paid_date.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            amount_cents: req.amount_cents,
            payment_method: req.payment_method.clone(),
            notes: req.notes,
            registered_by_user_id: Some("user:admin_principal".to_string()),
            registered_by_user_name: Some("Recepção".to_string()),
        };

        tx.payments.push(entry);
        tx.paid_amount_cents += req.amount_cents;
        tx.remaining_amount_cents = (tx.amount_cents - tx.paid_amount_cents).max(0);

        if tx.remaining_amount_cents == 0 {
            tx.status = TransactionStatus::Paid;
            tx.paid_date = Some(chrono::Utc::now().to_rfc3339());
        } else {
            tx.status = TransactionStatus::Partial;
        }

        tx.payment_method = Some(req.payment_method);
        Ok(tx.clone())
    }

    /// Atualiza status de um lançamento financeiro.
    pub async fn update_transaction_status(
        transaction_id: &str,
        req: UpdateTransactionStatusRequest,
    ) -> Result<Transaction, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let tx = db
            .transactions
            .iter_mut()
            .find(|t| t.id == transaction_id)
            .ok_or_else(|| format!("Transação {} não encontrada.", transaction_id))?;

        tx.status = req.status;
        if let Some(pd) = req.paid_date { tx.paid_date = Some(pd); }
        if let Some(pm) = req.payment_method { tx.payment_method = Some(pm); }

        if req.status == TransactionStatus::Paid {
            tx.paid_amount_cents = tx.amount_cents;
            tx.remaining_amount_cents = 0;
        }

        Ok(tx.clone())
    }
}
