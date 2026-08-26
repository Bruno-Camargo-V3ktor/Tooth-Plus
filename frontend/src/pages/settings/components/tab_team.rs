use dioxus::prelude::*;

#[component]
pub fn TabTeam() -> Element {
    let team_members = vec![
        ("RA", "Dr. Roberto Alencar", "admin • CRO-SP 84920", "Administrador Geral", "badge badge-blue"),
        ("LM", "Dr. Lucas Mendes", "dr.lucas • CRO-SP 99120", "Cirurgião-Dentista", "badge badge-green"),
        ("FO", "Fernanda Oliveira", "recepcao • Atendimento", "Recepção / Secretária", "badge badge-gray"),
    ];

    rsx! {
        div { class: "settings-card",
            div { class: "settings-card-header",
                h3 { class: "settings-card-title", "Membros da Equipe e Permissões" }
            }
            div { class: "settings-card-body",
                for (avatar, name, meta, role, badge_cls) in team_members {
                    div { key: "{name}", class: "team-member-row",
                        div { class: "team-member-info",
                            div { class: "team-avatar", "{avatar}" }
                            div {
                                div { style: "font-weight: 700; color: #f8fafc;", "{name}" }
                                div { style: "font-size: 12px; color: #94a3b8;", "{meta}" }
                            }
                        }
                        span { class: "{badge_cls}", "{role}" }
                    }
                }
            }
        }
    }
}
