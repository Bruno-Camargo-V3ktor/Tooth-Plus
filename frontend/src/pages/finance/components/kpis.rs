use shared::finance::FinanceSummary;
use dioxus::prelude::*;

#[component]
pub fn FinanceKpis(summary: FinanceSummary) -> Element {
    let income_fmt = format!("R$ {:.2}", summary.total_income_cents as f64 / 100.0);
    let expense_fmt = format!("R$ {:.2}", summary.total_expense_cents as f64 / 100.0);
    let pending_fmt = format!("R$ {:.2}", summary.pending_income_cents as f64 / 100.0);
    let balance_fmt = format!("R$ {:.2}", summary.net_balance_cents as f64 / 100.0);

    rsx! {
        div { class: "finance-kpi-grid",
            div { class: "finance-kpi-card kpi-income",
                span { class: "finance-kpi-label", "Receitas do Mês" }
                span { class: "finance-kpi-value val-income", "{income_fmt}" }
            }
            div { class: "finance-kpi-card kpi-expense",
                span { class: "finance-kpi-label", "Despesas Pagas" }
                span { class: "finance-kpi-value val-expense", "{expense_fmt}" }
            }
            div { class: "finance-kpi-card kpi-pending",
                span { class: "finance-kpi-label", "A Receber / Boletos" }
                span { class: "finance-kpi-value val-pending", "{pending_fmt}" }
            }
            div { class: "finance-kpi-card kpi-balance",
                span { class: "finance-kpi-label", "Saldo em Caixa" }
                span { class: "finance-kpi-value val-balance", "{balance_fmt}" }
            }
        }
    }
}
