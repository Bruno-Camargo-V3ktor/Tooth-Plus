use dioxus::prelude::*;

#[derive(Clone, PartialEq, Copy)]
pub enum DentitionType {
    Permanent,
    Deciduous,
}

#[component]
pub fn ToothSvg(number: &'static str, is_selected: bool, on_select: EventHandler<&'static str>) -> Element {
    let selected_cls = if is_selected { "tooth-item-box tooth-selected" } else { "tooth-item-box" };

    rsx! {
        div {
            class: "{selected_cls}",
            title: "Dente {number}",
            onclick: move |_| on_select.call(number),

            span { class: "tooth-num-label", "{number}" }

            svg {
                class: "tooth-anatomical-svg",
                view_box: "0 0 24 36",
                rect { x: "2", y: "2", width: "20", height: "14", rx: "3", fill: if is_selected { "#00a0e4" } else { "#f1f5f9" } }
                path { d: "M5 16 C5 26 8 34 10 34 C12 34 12 24 12 18 C12 24 12 34 14 34 C16 34 19 26 19 16 Z", fill: if is_selected { "#38bdf8" } else { "#cbd5e1" } }
            }
        }
    }
}

#[component]
pub fn TabOdontogram(
    selected_teeth: Signal<Vec<String>>,
    on_toggle_tooth: EventHandler<String>,
) -> Element {
    let mut dentition = use_signal(|| DentitionType::Permanent);
    let mut is_collapsed = use_signal(|| false);

    let upper_right_perm = vec!["18", "17", "16", "15", "14", "13", "12", "11"];
    let upper_left_perm = vec!["21", "22", "23", "24", "25", "26", "27", "28"];
    let lower_right_perm = vec!["48", "47", "46", "45", "44", "43", "42", "41"];
    let lower_left_perm = vec!["31", "32", "33", "34", "35", "36", "37", "38"];

    let upper_right_dec = vec!["55", "54", "53", "52", "51"];
    let upper_left_dec = vec!["61", "62", "63", "64", "65"];
    let lower_right_dec = vec!["85", "84", "83", "82", "81"];
    let lower_left_dec = vec!["71", "72", "73", "74", "75"];

    let (u_r, u_l, l_r, l_l) = if *dentition.read() == DentitionType::Permanent {
        (upper_right_perm, upper_left_perm, lower_right_perm, lower_left_perm)
    } else {
        (upper_right_dec, upper_left_dec, lower_right_dec, lower_left_dec)
    };

    rsx! {
        div { class: "odontogram-wrapper",
            div { class: "odontogram-nav-row",
                div { style: "display: flex; align-items: center; gap: 8px;",
                    h3 { class: "patient-card-title", "Odontograma" }
                }

                div { style: "display: flex; align-items: center; gap: 12px;",
                    div { class: "tab-underline-bar", style: "margin: 0; padding: 0;",
                        button {
                            class: if *dentition.read() == DentitionType::Permanent { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                            onclick: move |_| dentition.set(DentitionType::Permanent),
                            "Permanentes"
                        }
                        button {
                            class: if *dentition.read() == DentitionType::Deciduous { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                            onclick: move |_| dentition.set(DentitionType::Deciduous),
                            "Decíduos"
                        }
                    }

                    button {
                        r#type: "button",
                        class: "btn-secondary",
                        style: "padding: 4px 10px; font-size: 11.5px;",
                        onclick: move |_| is_collapsed.set(!is_collapsed()),
                        if is_collapsed() { "Abrir ⌄" } else { "Fechar ⌃" }
                    }
                }
            }

            if !is_collapsed() {
                div { class: "odontogram-teeth-grid",
                    // Arcada Superior
                    div { class: "teeth-row",
                        for tooth in u_r {
                            ToothSvg {
                                key: "{tooth}",
                                number: tooth,
                                is_selected: selected_teeth.read().contains(&tooth.to_string()),
                                on_select: move |t: &'static str| on_toggle_tooth.call(t.to_string()),
                            }
                        }
                        div { class: "teeth-divider-vertical" }
                        for tooth in u_l {
                            ToothSvg {
                                key: "{tooth}",
                                number: tooth,
                                is_selected: selected_teeth.read().contains(&tooth.to_string()),
                                on_select: move |t: &'static str| on_toggle_tooth.call(t.to_string()),
                            }
                        }
                    }

                    div { style: "width: 100%; height: 1px; background: rgba(255,255,255,0.08); margin: 4px 0;" }

                    // Arcada Inferior
                    div { class: "teeth-row",
                        for tooth in l_r {
                            ToothSvg {
                                key: "{tooth}",
                                number: tooth,
                                is_selected: selected_teeth.read().contains(&tooth.to_string()),
                                on_select: move |t: &'static str| on_toggle_tooth.call(t.to_string()),
                            }
                        }
                        div { class: "teeth-divider-vertical" }
                        for tooth in l_l {
                            ToothSvg {
                                key: "{tooth}",
                                number: tooth,
                                is_selected: selected_teeth.read().contains(&tooth.to_string()),
                                on_select: move |t: &'static str| on_toggle_tooth.call(t.to_string()),
                            }
                        }
                    }
                }
            }
        }
    }
}
