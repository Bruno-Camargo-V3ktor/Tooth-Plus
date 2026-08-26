//! # Sistema de Toast / Notificações Pop-up (Tooth Plus V2)
//!
//! Fornece um contexto global para exibir notificações em tela:
//! - `ToastVariant::Error` → fundo vermelho (erros de sistema)
//! - `ToastVariant::Info` → fundo branco (informações leves)
//! - `ToastVariant::Success` → fundo tema azul (ações concluídas)

use dioxus::prelude::*;

/// Variante visual do toast.
#[derive(Clone, PartialEq, Debug)]
pub enum ToastVariant {
    Error,
    Info,
    Success,
}

/// Um toast que está sendo exibido.
#[derive(Clone, PartialEq, Debug)]
pub struct ToastEntry {
    pub id: u64,
    pub variant: ToastVariant,
    pub message: String,
}

/// Estado global do toast (lista de toasts ativos).
#[derive(Clone)]
pub struct ToastState {
    pub toasts: Signal<Vec<ToastEntry>>,
    next_id: Signal<u64>,
}

impl ToastState {
    pub fn new() -> Self {
        Self {
            toasts: Signal::new(vec![]),
            next_id: Signal::new(0),
        }
    }

    /// Exibe um toast. Ele será auto-descartado em 4 segundos.
    pub fn show(&mut self, message: impl Into<String>, variant: ToastVariant) {
        let id = *self.next_id.read();
        *self.next_id.write() = id + 1;

        let entry = ToastEntry {
            id,
            variant,
            message: message.into(),
        };

        self.toasts.write().push(entry);

        // Auto-dismiss após 4 segundos
        let mut toasts = self.toasts.clone();
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(4000).await;
            toasts.write().retain(|t| t.id != id);
        });
    }

    pub fn dismiss(&mut self, id: u64) {
        self.toasts.write().retain(|t| t.id != id);
    }
}

/// Renderiza o contêiner de toasts. Deve ser colocado no topo do componente raiz.
#[component]
pub fn ToastContainer() -> Element {
    let mut state = consume_context::<ToastState>();
    let toasts = state.toasts.read().clone();

    rsx! {
        div { class: "toast-container",
            for toast in toasts {
                {
                    let (css_class, icon) = match &toast.variant {
                        ToastVariant::Error   => ("toast-error", "✕"),
                        ToastVariant::Info    => ("toast-info",  "ℹ"),
                        ToastVariant::Success => ("toast-success", "✓"),
                    };
                    let tid = toast.id;
                    let mut state_clone = state.clone();
                    rsx! {
                        div {
                            key: "{tid}",
                            class: "toast-item {css_class}",
                            onclick: move |_| state_clone.dismiss(tid),
                            span { style: "font-size: 14px; flex-shrink: 0;", "{icon}" }
                            span { class: "toast-message", "{toast.message}" }
                        }
                    }
                }
            }
        }
    }
}
