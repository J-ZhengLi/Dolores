//! Explicit composer references bind reviewed scope, revision and version.
use crate::{effective_skills, valid_skill_name, ProjectSkill, SkillScope, MAX_ACTIVE_SKILLS};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillSelection {
    /// Live file identity is checked by the host against its configured collection.
    /// Storage retains the reference without requiring a saved activation record.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub file: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
    pub scope: SkillScope,
    pub name: String,
    pub revision: u32,
    pub version: u32,
}
impl SkillSelection {
    pub fn from_skill(skill: &ProjectSkill) -> Self {
        Self {
            file: skill.current().reviewed_at == 0
                && skill.current().package.is_none()
                && skill.current().evaluation.is_none(),
            package_id: skill.current().package.as_ref().map(|p| p.id.clone()),
            scope: skill.scope,
            name: skill.name.clone(),
            revision: skill.revision,
            version: skill.current().version,
        }
    }
}
pub fn validate_skill_selections(selections: &[SkillSelection]) -> Result<(), String> {
    if selections.len() > MAX_ACTIVE_SKILLS
        || selections.iter().enumerate().any(|(i, s)| {
            !valid_skill_name(&s.name)
                || s.revision == 0
                || s.version == 0
                || (s.file && s.package_id.is_some())
                || s.package_id
                    .as_ref()
                    .is_some_and(|id| !crate::skill_packages::valid_digest(id))
                || selections[..i]
                    .iter()
                    .any(|p| p.scope == s.scope && p.name == s.name)
        })
    {
        return Err("Choose up to three distinct available skills with valid snapshots.".into());
    }
    Ok(())
}
pub fn resolve_skill_selections(
    skills: &[ProjectSkill],
    selections: &[SkillSelection],
) -> Result<Vec<ProjectSkill>, String> {
    validate_skill_selections(selections)?;
    let effective = effective_skills(skills);
    selections.iter().map(|selection|{
        let skill=effective.iter().find(|s|s.scope==selection.scope&&s.name==selection.name&&s.revision==selection.revision&&s.current().version==selection.version&&s.current().package.as_ref().map(|p|&p.id)==selection.package_id.as_ref())
            .ok_or("Selected skill changed, was disabled or is overridden. Remove the stale selection, refresh commands and reselect it; your draft remains. For saved instructions, cancel and submit again.")?;
        skill.validate()?;
        Ok((*skill).clone())
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn skill() -> ProjectSkill {
        ProjectSkill {
            scope: SkillScope::Global,
            name: "precise-format".into(),
            revision: 1,
            enabled: true,
            versions: vec![crate::SkillVersion {
                package: None,
                version: 1,
                reviewed_at: 123,
                rollback_from: None,
                evaluation: None,
                document: crate::SkillDocument {
                    name: "precise-format".into(),
                    description: "Structure exports".into(),
                    text: "EXACT_PUBLIC_FORMAT_v1".into(),
                },
            }],
        }
    }
    #[test]
    fn explicit_reference_includes_exact_snapshot_without_keyword_match() {
        let skill = skill();
        let selected = SkillSelection::from_skill(&skill);
        let input = crate::preview_context(vec![], "Hello").unwrap();
        let (context, sources) = crate::prepare_selected_skill_context(
            input.clone(),
            std::slice::from_ref(&skill),
            std::slice::from_ref(&selected),
        )
        .unwrap();
        assert!(context[0].content.contains("EXACT_PUBLIC_FORMAT_v1"));
        assert_eq!(sources[0].version, 1);
        assert_eq!(sources[0].scope, SkillScope::Global);
        assert!(!crate::prepare_relevant_skill_context(input, &[skill])
            .unwrap()
            .0[0]
            .content
            .contains("EXACT_PUBLIC_FORMAT_v1"));
    }
    #[test]
    fn stale_disabled_and_overridden_refs_refuse_instead_of_substitution() {
        let mut skill = skill();
        let selected = SkillSelection::from_skill(&skill);
        skill.revision = 2;
        assert!(resolve_skill_selections(
            std::slice::from_ref(&skill),
            std::slice::from_ref(&selected)
        )
        .is_err());
        skill.revision = 1;
        skill.enabled = false;
        assert!(resolve_skill_selections(
            std::slice::from_ref(&skill),
            std::slice::from_ref(&selected)
        )
        .is_err());
        skill.enabled = true;
        let mut project = skill.clone();
        project.scope = SkillScope::Project;
        assert!(
            resolve_skill_selections(&[skill, project], std::slice::from_ref(&selected)).is_err()
        );
        assert!(validate_skill_selections(&[selected.clone(), selected]).is_err());
    }
    #[test]
    fn package_identity_survives_a_forgotten_and_readded_name_with_identical_text_and_version() {
        let mut skill = skill();
        skill.versions[0].package = Some(crate::skill_packages::Snapshot {
            id: "a".repeat(64),
            content_digest: "b".repeat(64),
            manifest: Default::default(),
            resources: vec![crate::skill_packages::Resource {
                path: "SKILL.md".into(),
                bytes: skill.current().document.text.len(),
                digest: "c".repeat(64),
            }],
            directories: vec![],
            root: std::env::temp_dir()
                .join("public-skill-package")
                .to_string_lossy()
                .into_owned(),
        });
        let selected = SkillSelection::from_skill(&skill);
        assert_eq!(
            selected.package_id.as_deref(),
            Some("a".repeat(64).as_str())
        );
        assert!(resolve_skill_selections(
            std::slice::from_ref(&skill),
            std::slice::from_ref(&selected)
        )
        .is_ok());
        skill.versions[0].package.as_mut().unwrap().id = "d".repeat(64);
        assert!(resolve_skill_selections(
            std::slice::from_ref(&skill),
            std::slice::from_ref(&selected)
        )
        .is_err());
        let legacy: SkillSelection = serde_json::from_str(
            r#"{"scope":"global","name":"precise-format","revision":1,"version":1}"#,
        )
        .unwrap();
        assert!(resolve_skill_selections(std::slice::from_ref(&skill), &[legacy]).is_err());
        assert!(resolve_skill_selections(
            std::slice::from_ref(&skill),
            &[SkillSelection::from_skill(&skill)]
        )
        .is_ok());
    }
}
