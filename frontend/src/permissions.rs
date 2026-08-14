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
        label: "Módulo: Pacientes",
        items: &[
            ("patients:read", "Visualizar Prontuários"),
            ("patients:write", "Editar Fichas Clínicas"),
            ("patients:delete", "Excluir Pacientes"),
        ],
    },
    PermGroup {
        label: "Módulo: Financeiro",
        items: &[
            ("finance:read", "Visualizar Fluxo de Caixa"),
            ("finance:write", "Lançar Receitas e Despesas"),
            ("finance:delete", "Estornar Pagamentos"),
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
        label: "Módulo: Estoque",
        items: &[
            ("stock:read", "Visualizar Inventário"),
            ("stock:write", "Movimentar Itens"),
            ("stock:delete", "Ajustes de Estoque"),
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

pub fn has_permission(
    session: &SessionState,
    active: &ActiveClinicState,
    perm: &str,
) -> bool {
    let (Some(sess), Some(clinic)) = (session, active) else {
        return false;
    };
    let access = sess.clinics.iter().find(|c| c.clinic_id == clinic.clinic_id);
    let Some(a) = access else { return false };
    if a.role == "admin" || a.permissions.iter().any(|p| p == "admin:all") {
        return true;
    }
    let alt_perm = if perm.starts_with("appointments:") {
        Some(perm.replace("appointments:", "agenda:"))
    } else if perm.starts_with("agenda:") {
        Some(perm.replace("agenda:", "appointments:"))
    } else {
        None
    };

    a.permissions.iter().any(|p| {
        p == perm || alt_perm.as_deref() == Some(p.as_str())
    })
}

pub fn has_any_permission(
    session: &SessionState,
    active: &ActiveClinicState,
    perms: &[&str],
) -> bool {
    perms.iter().any(|p| has_permission(session, active, p))
}
