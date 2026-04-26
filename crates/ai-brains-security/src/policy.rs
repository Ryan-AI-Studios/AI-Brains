use ai_brains_core::privacy::Privacy;

pub fn is_embeddable(privacy: Privacy) -> bool {
    matches!(privacy, Privacy::CloudOk | Privacy::LocalOnly)
}
