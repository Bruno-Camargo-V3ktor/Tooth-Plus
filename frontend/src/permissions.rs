use crate::{ActiveClinicState, SessionState};

pub struct PermGroup {
    pub label: &'static str,
    pub items: &'static [(&'static str, &'static str)],
}

pub const ALL_PERMISSION_GROUPS: &[PermGroup] = &[
    PermGroup {
        label: "Módulo: Agenda",
        items: &[
            ("appointments:read", "Visualizar Agendamentos"),
            ("appointments:write", "Criar e Editar Agendamentos"),
            ("appointments:delete", "Cancelar e Excluir Agendamentos"),
            ("appointments:finance", "Comissão e Rateio Financeiro"),
        ],
    },
    PermGroup {
        label: "Módulo: Pacientes e Cadastro",
        items: &[
            ("patients:read", "Visualizar Lista de Pacientes"),
            ("patients:write", "Cadastrar e Editar Pacientes"),
            ("patients:delete", "Excluir Pacientes"),
        ],
    },
    PermGroup {
        label: "Módulo: Orçamentos & Planos Clínicos",
        items: &[
            ("treatment_plans:read", "Visualizar Orçamentos e Propostas"),
            ("treatment_plans:write", "Criar, Editar e Aprovar Orçamentos"),
            ("treatment_plans:delete", "Excluir Orçamentos"),
            ("treatment_plans:pay", "Registrar Pagamentos / Amortização de Orçamentos"),
        ],
    },
    PermGroup {
        label: "Módulo: Procedimentos e Evolução",
        items: &[
            ("treatments:read", "Visualizar Histórico de Procedimentos"),
            ("treatments:write", "Registrar e Evoluir Procedimentos"),
            ("treatments:delete", "Excluir Registros de Procedimentos"),
        ],
    },
    PermGroup {
        label: "Módulo: Catálogo de Tratamentos (Templates)",
        items: &[
            ("treatment_templates:read", "Visualizar Catálogo de Tratamentos"),
            ("treatment_templates:write", "Criar e Editar Templates de Tratamentos"),
            ("treatment_templates:delete", "Excluir Templates do Catálogo"),
        ],
    },
    PermGroup {
        label: "Módulo: Anamnese Clínica",
        items: &[
            ("anamnese:read", "Visualizar Ficha de Anamnese"),
            ("anamnese:write", "Editar Respostas de Anamnese"),
            ("anamnese:manage_templates", "Gerenciar Modelos de Anamnese (Adulto/Infantil)"),
        ],
    },
    PermGroup {
        label: "Módulo: Exames e Laudos",
        items: &[
            ("exams:read", "Visualizar Exames e Fotos"),
            ("exams:upload", "Enviar e Anexar Novos Exames"),
            ("exams:edit", "Editar Laudos e Diagnósticos"),
            ("exams:delete", "Excluir Exames"),
        ],
    },
    PermGroup {
        label: "Módulo: Financeiro",
        items: &[
            ("finance:read_all", "Visualizar Todas as Finanças"),
            ("finance:read_income", "Visualizar Entradas (Receitas)"),
            ("finance:read_expense", "Visualizar Saídas (Despesas)"),
            ("finance:read_pending", "Visualizar Lançamentos Pendentes"),
            ("finance:write_income", "Lançar Novas Entradas"),
            ("finance:write_expense", "Lançar Novas Saídas"),
            ("finance:update_status", "Alterar Status de Pagamentos"),
            ("finance:delete", "Excluir e Estornar Lançamentos"),
        ],
    },
    PermGroup {
        label: "Módulo: Equipe e Acessos",
        items: &[
            ("users:read", "Visualizar Equipe"),
            ("users:write", "Criar e Editar Membros"),
            ("users:manage_status", "Ativar e Desativar Membros"),
        ],
    },
    PermGroup {
        label: "Módulo: Estoque e Patrimônio",
        items: &[
            ("stock:read", "Visualizar Itens e Alertas"),
            ("stock:write", "Cadastrar e Editar Itens"),
            ("stock:movement", "Registrar Entradas e Saídas"),
            ("stock:delete", "Excluir Itens do Estoque"),
        ],
    },
    PermGroup {
        label: "Módulo: Documentos",
        items: &[
            ("documents:read", "Visualizar Documentos"),
            ("documents:write", "Criar e Editar Documentos"),
            ("documents:delete", "Excluir Documentos"),
        ],
    },
    PermGroup {
        label: "Configurações da Clínica",
        items: &[
            ("clinics:read", "Visualizar Dados Cadastrais"),
            ("clinics:write", "Editar Configurações"),
            ("clinics:delete", "Encerrar Unidade"),
        ],
    },
    PermGroup {
        label: "Integração: WhatsApp",
        items: &[
            ("whatsapp:read", "Visualizar Sessão WhatsApp"),
            ("whatsapp:write", "Conectar e Gerenciar Sessão"),
        ],
    },
    PermGroup {
        label: "Configurações: Avançadas",
        items: &[
            ("advanced:read", "Visualizar Config. Avançadas"),
            ("advanced:write", "Editar Comportamento Global"),
        ],
    },
];

pub fn has_permission(session: &SessionState, active: &ActiveClinicState, perm: &str) -> bool {
    let (Some(sess), Some(clinic)) = (session, active) else {
        return false;
    };
    let access = sess
        .clinics
        .iter()
        .find(|c| c.clinic_id == clinic.clinic_id);
    let Some(a) = access else { return false };
    if a.role == "admin" || a.permissions.iter().any(|p| p == "admin:all") {
        return true;
    }

    a.permissions.iter().any(|p| {
        if p == perm || p == "admin:all" {
            return true;
        }

        // Agenda / Appointments
        if (perm.starts_with("appointments:") && p.replace("agenda:", "appointments:") == perm)
            || (perm.starts_with("agenda:") && p.replace("appointments:", "agenda:") == perm)
        {
            return true;
        }

        // Tratamentos e Sub-módulos (Orçamentos, Catálogo, Prontuário)
        if perm == "treatments:read"
            && (p == "treatment_plans:read" || p == "treatment_templates:read" || p == "patients:read" || p == "patients:write")
        {
            return true;
        }
        if perm == "treatments:write"
            && (p == "treatment_plans:write" || p == "treatment_templates:write" || p == "patients:write")
        {
            return true;
        }
        if perm == "treatments:delete"
            && (p == "treatment_plans:delete" || p == "treatment_templates:delete" || p == "treatments:write" || p == "patients:delete")
        {
            return true;
        }
        if (perm.starts_with("treatment_plans:") || perm.starts_with("treatment_templates:"))
            && (p == "treatments:write" || p == "patients:write")
        {
            return true;
        }

        // Anamnese & Exames
        if perm.starts_with("anamnese:") && (p == "patients:write" || (p == "patients:read" && perm.ends_with(":read"))) {
            return true;
        }
        if perm.starts_with("exams:") && (p == "patients:write" || (p == "patients:read" && perm.ends_with(":read"))) {
            return true;
        }

        // Documentos
        if perm.starts_with("documents:") && (p == "patients:write" || (p == "patients:read" && perm.ends_with(":read"))) {
            return true;
        }

        // Financeiro
        if perm == "finance:read"
            && (p == "finance:read_all" || p == "finance:read_income" || p == "finance:read_expense" || p == "finance:read_pending")
        {
            return true;
        }
        if perm == "finance:read_all" && p == "finance:read" {
            return true;
        }
        if (perm == "finance:read_income" || perm == "finance:read_expense" || perm == "finance:read_pending")
            && (p == "finance:read_all" || p == "finance:read")
        {
            return true;
        }
        if perm == "finance:write" && (p == "finance:write_income" || p == "finance:write_expense") {
            return true;
        }
        if (perm == "finance:write_income" || perm == "finance:write_expense" || perm == "finance:update_status")
            && p == "finance:write"
        {
            return true;
        }

        // Estoque
        if (perm == "stock:movement" || perm == "stock:delete") && p == "stock:write" {
            return true;
        }

        false
    })
}

pub fn has_any_permission(
    session: &SessionState,
    active: &ActiveClinicState,
    perms: &[&str],
) -> bool {
    perms.iter().any(|p| has_permission(session, active, p))
}
