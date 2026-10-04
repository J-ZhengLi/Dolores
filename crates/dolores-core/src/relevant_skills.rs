use crate::{Message, ProjectSkill, SkillSource};
pub fn prepare_relevant_skill_context(
    mut messages: Vec<Message>,
    skills: &[ProjectSkill],
) -> Result<(Vec<Message>, Vec<SkillSource>), String> {
    let input = messages
        .last()
        .map_or("", |m| m.content.as_str())
        .to_lowercase();
    let active = crate::effective_skills(skills);
    let selected: Vec<_> = active
        .iter()
        .filter(|s| {
            input
                .split(|c: char| !c.is_alphanumeric() && c != '-')
                .any(|word| word == s.name)
                || s.current()
                    .document
                    .description
                    .split(|c: char| !c.is_alphanumeric())
                    .any(|word| {
                        word.len() >= 4
                            && input
                                .split(|c: char| !c.is_alphanumeric())
                                .any(|w| w == word.to_lowercase())
                    })
        })
        .map(|s| (*s).clone())
        .collect();
    if !active.is_empty() {
        messages[0].content.push_str("\nActivated skill catalogue (name/description keyword relevance, not semantic search). Full instructions are included only for matching skills. Mention a skill name to select it explicitly:\n");
        for skill in active {
            skill.validate()?;
            messages[0].content.push_str(&format!(
                "{}: {}\n",
                skill.name,
                skill.current().document.description
            ));
        }
    }
    crate::prepare_skill_context(messages, &selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SkillDocument, SkillScope, SkillVersion};
    #[test]
    fn unrelated_skills_keep_only_catalogue_and_explicit_names_select_exact_snapshots() {
        let skill = ProjectSkill {
            scope: SkillScope::Global,
            name: "review-code".into(),
            revision: 1,
            enabled: true,
            versions: vec![SkillVersion {
                version: 1,
                reviewed_at: 0,
                rollback_from: None,
                evaluation: None,
                document: SkillDocument {
                    name: "review-code".into(),
                    description: "Review Rust changes".into(),
                    text: "EXACT_REVIEW_INSTRUCTIONS".into(),
                },
            }],
        };
        let (unrelated, sources) = prepare_relevant_skill_context(
            crate::preview_context(vec![], "Write a poem").unwrap(),
            std::slice::from_ref(&skill),
        )
        .unwrap();
        assert!(sources.is_empty());
        assert!(unrelated[0].content.contains("review-code"));
        assert!(!unrelated[0].content.contains("EXACT_REVIEW_INSTRUCTIONS"));
        let (selected, sources) = prepare_relevant_skill_context(
            crate::preview_context(vec![], "Use review-code").unwrap(),
            &[skill],
        )
        .unwrap();
        assert_eq!(sources.len(), 1);
        assert!(selected[0].content.contains("EXACT_REVIEW_INSTRUCTIONS"));
    }
}
