pub struct PermGroup {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub items: &'static [(&'static str, &'static str)],
}

pub const ALL_PERMISSION_GROUPS: &[PermGroup] = &[
    PermGroup {
        key: "agenda",
        label: "Agenda & Agendamentos",
        description: "Controle de consultas, bloqueios e horários da equipe",
        items: &[
            ("agenda:read", "Visualizar grade de horários e agendamentos"),
            ("agenda:write", "Criar, reagendar e editar consultas"),
            ("agenda:delete", "Cancelar e excluir agendamentos"),
            ("agenda:finance", "Definir comissão e rateio de procedimentos"),
        ],
    },
    PermGroup {
        key: "patients",
        label: "Pacientes & Prontuário",
        description: "Fichas cadastrais, histórico clínico e odontograma",
        items: &[
            ("patients:read", "Visualizar lista de pacientes e cadastros"),
            ("patients:write", "Cadastrar e editar dados de pacientes"),
            ("patients:delete", "Excluir cadastro de pacientes"),
            ("patients:evolutions", "Registrar e evoluir odontograma clínico"),
        ],
    },
    PermGroup {
        key: "anamnese",
        label: "Anamnese Clínica",
        description: "Questionários médicos e histórico de saúde",
        items: &[
            ("anamnese:read", "Visualizar respostas de anamnese dos pacientes"),
            ("anamnese:write", "Preencher e atualizar fichas de anamnese"),
            ("anamnese:manage_templates", "Criar e editar modelos e perguntas de anamnese"),
        ],
    },
    PermGroup {
        key: "treatments",
        label: "Procedimentos & Orçamentos",
        description: "Planos de tratamento, orçamentos e tabela de procedimentos",
        items: &[
            ("treatment_plans:read", "Visualizar propostas e orçamentos"),
            ("treatment_plans:write", "Criar, editar e aprovar orçamentos"),
            ("treatment_plans:delete", "Excluir propostas de orçamentos"),
            ("treatment_templates:manage", "Gerenciar catálogo e tabela de preços"),
        ],
    },
    PermGroup {
        key: "finance",
        label: "Módulo Financeiro",
        description: "Fluxo de caixa, recebimentos, despesas e relatórios",
        items: &[
            ("finance:read_all", "Visualizar todas as movimentações financeiras"),
            ("finance:read_income", "Visualizar entradas (receitas recebidas)"),
            ("finance:read_expense", "Visualizar saídas (despesas e custos)"),
            ("finance:write_income", "Registrar recebimentos e baixar faturas"),
            ("finance:write_expense", "Lançar despesas e contas a pagar"),
            ("finance:delete", "Estornar ou excluir lançamentos financeiros"),
        ],
    },
    PermGroup {
        key: "stock",
        label: "Estoque & Insumos",
        description: "Controle de materiais, lotes e validade",
        items: &[
            ("stock:read", "Visualizar níveis de estoque e alertas de reposição"),
            ("stock:write", "Cadastrar e editar materiais e equipamentos"),
            ("stock:movement", "Lançar movimentações de entrada e saída"),
            ("stock:delete", "Excluir itens do estoque"),
        ],
    },
    PermGroup {
        key: "documents",
        label: "Documentos & Assinatura Digital",
        description: "Emissão de contratos, atestados, receitas e termos legais",
        items: &[
            ("documents:read", "Visualizar histórico de documentos e assinaturas"),
            ("documents:write", "Gerar contratos, receitas e atestados"),
            ("documents:sign", "Assinar digitalmente com carimbo ICP-Brasil"),
            ("documents:delete", "Excluir documentos emitidos"),
        ],
    },
    PermGroup {
        key: "settings",
        label: "Configurações da Clínica",
        description: "Dados institucionais, gestão de equipe e consultórios",
        items: &[
            ("clinics:read", "Visualizar configurações e dados da clínica"),
            ("clinics:write", "Alterar dados cadastrais, horários e papel timbrado"),
            ("users:manage", "Gerenciar membros da equipe e permissões de acesso"),
            ("chairs:manage", "Gerenciar cadeiras e consultórios clínicos"),
        ],
    },
];
