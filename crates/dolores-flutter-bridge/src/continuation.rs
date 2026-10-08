use dolores_core::{PausedTask, Role, SessionStore};

pub(super) const INPUT: &str = "Continue working on the previous task.";
pub(super) const REPAIR_INPUT: &str = "Repair the failed check and verify it by rerunning the same command. Propose any needed changes for fresh review.";
pub(super) fn validate_input(input: &str, source: &PausedTask) -> Result<(), String> {
    if input == INPUT
        || (input == REPAIR_INPUT && source.reason == dolores_core::PauseReason::CommandReview)
    {
        Ok(())
    } else {
        Err("Use Continue or Repair and verify with its unchanged recovery request.".into())
    }
}
pub(super) fn source(
    store: &dyn SessionStore,
    session: &str,
    id: i64,
) -> Result<(PausedTask, String), String> {
    let page = store.messages_page(session, None, false, 2)?;
    let last = page
        .items
        .last()
        .ok_or("There is no saved paused response to continue.")?;
    if last.id != id || last.role != Role::Assistant {
        return Err("This paused response is no longer the latest message. Review the current chat before continuing.".into());
    }
    let paused = last
        .metadata
        .as_ref()
        .and_then(|m| m.paused.clone())
        .ok_or("This response is not paused.")?;
    Ok((paused, last.content.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_request_is_bound_to_a_failed_check_and_legacy_continue_stays_valid() {
        let mut source = PausedTask {
            segments: 1,
            reason: dolores_core::PauseReason::CommandReview,
            task: "Run the check without editing".into(),
            receipts: vec![],
        };
        assert!(validate_input(REPAIR_INPUT, &source).is_ok());
        assert!(validate_input(INPUT, &source).is_ok());
        assert!(validate_input("Repair everything without review", &source).is_err());
        source.reason = dolores_core::PauseReason::StepLimit;
        assert!(validate_input(REPAIR_INPUT, &source).is_err());
        assert!(validate_input(INPUT, &source).is_ok());
    }
}
