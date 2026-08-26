use dioxus::prelude::*;

#[component]
pub fn FinanceEquationSummary(
    received_cents: i64,
    pending_income_cents: i64,
    paid_expense_cents: i64,
    pending_expense_cents: i64,
) -> Element {
    let balance_cents = received_cents - paid_expense_cents;
    let expected_income_cents = received_cents + pending_income_cents;
    let expected_expense_cents = paid_expense_cents + pending_expense_cents;
    let expected_balance_cents = expected_income_cents - expected_expense_cents;

    rsx! {
        div { class: "finance-equation-bar",
            // RECEITAS
            div { class: "equation-col",
                span { class: "equation-label", "RECEITAS" }
                span { class: "equation-val-received", "R$ {received_cents as f64 / 100.0:.2}" }
                span { class: "equation-sub",
                    "A receber: "
                    strong { "R$ {pending_income_cents as f64 / 100.0:.2}" }
                }
                span { class: "equation-sub",
                    "Total previsto: "
                    strong { "R$ {expected_income_cents as f64 / 100.0:.2}" }
                }
            }

            // SINAL MENOS
            div { class: "equation-symbol", "—" }

            // DESPESAS
            div { class: "equation-col",
                span { class: "equation-label", "DESPESAS" }
                span { class: "equation-val-expense", "R$ {paid_expense_cents as f64 / 100.0:.2}" }
                span { class: "equation-sub",
                    "A pagar: "
                    strong { "R$ {pending_expense_cents as f64 / 100.0:.2}" }
                }
                span { class: "equation-sub",
                    "Total previsto: "
                    strong { "R$ {expected_expense_cents as f64 / 100.0:.2}" }
                }
            }

            // SINAL IGUAL
            div { class: "equation-symbol", "=" }

            // SALDO
            div { class: "equation-col",
                span { class: "equation-label", "SALDO" }
                span { class: "equation-val-balance", "R$ {balance_cents as f64 / 100.0:.2}" }
                span { class: "equation-sub",
                    "Total previsto: "
                    strong { "R$ {expected_balance_cents as f64 / 100.0:.2}" }
                }
            }
        }
    }
}
