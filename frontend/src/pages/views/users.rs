use dioxus::prelude::*;

#[component]
pub fn UsersView() -> Element {
    rsx! { div { h1 { class: "page-title", "Gerenciamento de Usuários" } div { class: "content-card", "Lista de funcionários e permissões." } } }
}
