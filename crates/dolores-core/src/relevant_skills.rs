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
    let mut candidates: Vec<_> = active
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
    // A named skill must win over incidental description keywords when only a
    // bounded subset can fit. Picker selections retain the first priority below.
    candidates.sort_by_key(|s| {
        !input
            .split(|c: char| !c.is_alphanumeric() && c != '-')
            .any(|word| word == s.name)
    });
    let mut selected = explicit;
    for skill in candidates {
        if selected
            .iter()
            .any(|s| s.scope == skill.scope && s.name == skill.name)
        {
            continue;
        }
        let scoped: Vec<_> = selected.iter().filter(|s| s.scope == skill.scope).collect();
        if scoped.len() < crate::MAX_ACTIVE_SKILLS
            && scoped
                .iter()
                .map(|s| s.current().document.text.len())
                .sum::<usize>()
                + skill.current().document.text.len()
                <= crate::MAX_SKILL_BYTES
        {
            selected.push(skill);
        }
    }
    if !active.is_empty() {
        messages[0].content.push_str("\nAvailable skill catalogue (name/description keyword relevance, not semantic search). Full instructions are included for matching skills within the per-request limits. Mention a skill name or select it with / to use it explicitly; no prior review is needed:\n");
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
    fn many_available_matches_keep_requests_bounded_and_explicit_selection_first() {
        let skills: Vec<_> = (0..8)
            .map(|i| ProjectSkill {
                scope: SkillScope::Global,
                name: format!("format-{i}"),
                revision: 1,
                enabled: true,
                versions: vec![SkillVersion {
                    package: None,
                    version: 1,
                    reviewed_at: 0,
                    rollback_from: None,
                    evaluation: None,
                    document: SkillDocument {
                        name: format!("format-{i}"),
                        description: "Format output".into(),
                        text: format!("EXACT_INSTRUCTIONS_{i}"),
                    },
                }],
            })
            .collect();
        let context = crate::preview_context(vec![], "Format output").unwrap();
        let explicit = crate::SkillSelection::from_skill(&skills[7]);
        let (messages, sources) =
            prepare_selected_skill_context(context, &skills, &[explicit]).unwrap();
        assert_eq!(sources.len(), 3);
        assert_eq!(sources[0].name, "format-7");
        assert!(messages[0].content.contains("format-6: Format output"));
        assert!(!messages[0].content.contains("EXACT_INSTRUCTIONS_6"));
        let (_, named) = prepare_relevant_skill_context(
            crate::preview_context(vec![], "Use format-7").unwrap(),
            &skills,
        )
        .unwrap();
        assert_eq!(named[0].name, "format-7");
    }
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
