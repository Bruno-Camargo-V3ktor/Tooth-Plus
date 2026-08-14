use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionDirection {
    Income,
    Expense,
}

impl TransactionDirection {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Income => "Entrada",
            Self::Expense => "Saída",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Paid,
    Canceled,
    Refunded,
}

impl TransactionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pendente",
            Self::Paid => "Pago",
            Self::Canceled => "Cancelado",
            Self::Refunded => "Estornado",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            Self::Pending => "fin-status-pending",
            Self::Paid => "fin-status-paid",
            Self::Canceled => "fin-status-canceled",
            Self::Refunded => "fin-status-refunded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub clinic_id: String,
    pub appointment_id: Option<String>,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub direction: TransactionDirection,
    pub amount_cents: i64,
    pub description: String,
    pub category: String,
    pub status: TransactionStatus,
    pub due_date: String,
    pub paid_date: Option<String>,
    pub payment_method: Option<String>,
    pub installment_current: i32,
    pub installment_total: i32,
    #[serde(default)]
    pub is_calculated_pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateTransactionRequest {
    pub clinic_id: String,
    pub appointment_id: Option<String>,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub user_id: Option<String>,
    pub direction: TransactionDirection,
    pub amount_cents: i64,
    pub description: String,
    pub category: String,
    pub due_date: String,
    pub paid_date: Option<String>,
    pub payment_method: Option<String>,
    pub status: TransactionStatus,
    pub installment_current: Option<i32>,
    pub installment_total: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateTransactionStatusRequest {
    pub status: TransactionStatus,
    pub paid_date: Option<String>,
    pub payment_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinanceQuery {
    pub clinic_id: String,
    #[serde(default)]
    pub month: Option<u32>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FinanceSummary {
    pub total_income_cents: i64,
    pub total_expense_cents: i64,
    pub net_balance_cents: i64,
    pub pending_income_cents: i64,
    pub pending_expense_cents: i64,
    pub total_transactions_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinanceResponse {
    pub summary: FinanceSummary,
    pub transactions: Vec<Transaction>,
}
