use dolores_core::{PausedTask, Role, SessionStore};

pub(super) const INPUT: &str = "Continue working on the previous task.";
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
