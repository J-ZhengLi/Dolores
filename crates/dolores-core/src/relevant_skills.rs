use crate::{Message, ProjectSkill, SkillSource};

fn keyword_matches(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    // Keep selection literal and bounded, while accepting a simple English
    // singular/plural pair such as "review" / "reviews" in either direction.
    left.is_ascii()
        && right.is_ascii()
        && (left
            .strip_suffix('s')
            .is_some_and(|word| word.len() >= 4 && word == right)
            || right
                .strip_suffix('s')
                .is_some_and(|word| word.len() >= 4 && word == left))
}

pub fn prepare_relevant_skill_context(
    messages: Vec<Message>,
    skills: &[ProjectSkill],
) -> Result<(Vec<Message>, Vec<SkillSource>), String> {
    prepare_selected_skill_context(messages, skills, &[])
}

pub fn prepare_selected_skill_context(
    mut messages: Vec<Message>,
    skills: &[ProjectSkill],
    selections: &[crate::SkillSelection],
) -> Result<(Vec<Message>, Vec<SkillSource>), String> {
    let explicit = crate::resolve_skill_selections(skills, selections)?;
    let input = messages
        .last()
        .map_or("", |m| m.content.as_str())
        .to_lowercase();
    let active = crate::effective_skills(skills);
    let selected: Vec<_> = active
        .iter()
        .filter(|s| {
            explicit
                .iter()
                .any(|p| p.scope == s.scope && p.name == s.name)
                || input
                    .split(|c: char| !c.is_alphanumeric() && c != '-')
                    .any(|word| word == s.name)
                || s.current()
                    .document
                    .description
                    .split(|c: char| !c.is_alphanumeric())
                    .any(|word| {
                        let keyword = word.to_lowercase();
                        word.len() >= 4
                            && input
                                .split(|c: char| !c.is_alphanumeric())
                                .any(|w| keyword_matches(w, &keyword))
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
    fn generated_review_description_selects_the_singular_request() {
        let skill = ProjectSkill {
            scope: SkillScope::Global,
            name: "code-review-structure".into(),
            revision: 1,
            enabled: true,
            versions: vec![SkillVersion {
                package: None,
                version: 1,
                reviewed_at: 0,
                rollback_from: None,
                evaluation: None,
                document: SkillDocument {
                    name: "code-review-structure".into(),
                    description: "Use for code reviews requiring a Verdict, Risk, and Check in under 80 words.".into(),
                    text: "EXACT_REVIEW_INSTRUCTIONS".into(),
                },
            }],
        };
        let request = "Review this function change: first(values) was return values[0] if values else None, now return values[0].";
        let (prepared, sources) = prepare_relevant_skill_context(
            crate::preview_context(vec![], request).unwrap(),
            std::slice::from_ref(&skill),
        )
        .unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, skill.name);
        assert!(prepared[0].content.contains("EXACT_REVIEW_INSTRUCTIONS"));
        let (unrelated, sources) = prepare_relevant_skill_context(
            crate::preview_context(vec![], "Show a preview of this function.").unwrap(),
            std::slice::from_ref(&skill),
        )
        .unwrap();
        assert!(sources.is_empty());
        assert!(!unrelated[0].content.contains("EXACT_REVIEW_INSTRUCTIONS"));
        let mut singular = skill;
        singular.versions[0].document.description = "Review functions".into();
        let (prepared, sources) = prepare_relevant_skill_context(
            crate::preview_context(vec![], "Give two REVIEWS.").unwrap(),
            &[singular],
        )
        .unwrap();
        assert_eq!(sources.len(), 1);
        assert!(prepared[0].content.contains("EXACT_REVIEW_INSTRUCTIONS"));
    }
    #[test]
    fn unrelated_skills_keep_only_catalogue_and_explicit_names_select_exact_snapshots() {
        let skill = ProjectSkill {
            scope: SkillScope::Global,
            name: "review-code".into(),
            revision: 1,
            enabled: true,
            versions: vec![SkillVersion {
                package: None,
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
