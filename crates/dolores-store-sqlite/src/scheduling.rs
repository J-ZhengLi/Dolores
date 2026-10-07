use crate::storage_error;
use dolores_core::scheduling::*;
use rusqlite::{Connection,params};
fn tasks(c:&Connection)->Result<Vec<ScheduledTask>,String>{
    let mut q=c.prepare("SELECT data FROM scheduled_tasks ORDER BY id").map_err(storage_error)?;
    let rows=q.query_map([],|r|r.get::<_,String>(0)).map_err(storage_error)?;
    rows.map(|r|serde_json::from_str(&r.map_err(storage_error)?).map_err(storage_error)).collect()
}
pub(super) fn list(c:&Connection)->Result<Vec<ScheduledTask>,String>{tasks(c)}
pub(super) fn save(c:&mut Connection,value:&ScheduledTask,expected:Option<u32>)->Result<ScheduledTask,String>{
    value.validate()?;
    let tx=c.transaction().map_err(storage_error)?;
    let existing=tasks(&tx)?;
    if expected.is_none(){
        if let Some(t)=existing.iter().find(|t|t.source_key==value.source_key){return Ok(t.clone());}
        if existing.len()>=MAX_SCHEDULES{return Err("Scheduled task limit reached. Remove an unused task first.".into());}
    }
    let old=existing.iter().find(|t|t.id==value.id);
    if old.map(|t|t.revision)!=expected{return Err("Task changed. Refresh before trying again.".into());}
    let mut next=value.clone();
    next.revision=expected.map_or(Some(1),|r|r.checked_add(1)).ok_or("Task revision exhausted.")?;
    tx.execute("INSERT INTO scheduled_tasks(id,source_key,data) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![next.id,next.source_key,serde_json::to_string(&next).map_err(storage_error)?]).map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;Ok(next)
}
pub(super) fn history(c:&Connection,task:&str)->Result<Vec<ScheduledOccurrence>,String>{
    let mut q=c.prepare("SELECT data FROM scheduled_occurrences WHERE task=?1 ORDER BY due DESC,id DESC LIMIT 50").map_err(storage_error)?;
    let rows=q.query_map([task],|r|r.get::<_,String>(0)).map_err(storage_error)?;
    rows.map(|r|serde_json::from_str(&r.map_err(storage_error)?).map_err(storage_error)).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStore;
    use dolores_core::SessionStore;
    fn task()->ScheduledTask{ScheduledTask{id:"task".into(),revision:1,source_session:"chat".into(),source_key:"exchange".into(),title:"Report".into(),prompt:"Write a report".into(),workspace:Default::default(),rule:ScheduleRule{kind:"daily".into(),time:"21:00".into(),date:None,weekdays:vec![],zone:"Asia/Shanghai".into()},next_due:Some(1791380000),paused:false,deleted:false,preferences:Default::default(),skill:None,effective:dolores_core::inspect_settings(Default::default(),"test",None,&[]).unwrap()}}
    #[test] fn creation_is_idempotent_and_stale_or_failed_updates_preserve_receipt(){
        let d=tempfile::tempdir().unwrap();let p=d.path().join("test.db");let store=SqliteStore::open(&p).unwrap();
        let t=store.save_scheduled_task(&task(),None).unwrap();let mut repeat=t.clone();repeat.id="duplicate".into();assert_eq!(store.save_scheduled_task(&repeat,None).unwrap(),t);
        let mut paused=t.clone();paused.paused=true;let next=store.save_scheduled_task(&paused,Some(t.revision)).unwrap();
        assert!(store.save_scheduled_task(&paused,Some(t.revision)).is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_schedule BEFORE UPDATE ON scheduled_tasks BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store.save_scheduled_task(&t,Some(next.revision)).is_err());drop(store);
        assert_eq!(SqliteStore::open(&p).unwrap().scheduled_tasks().unwrap(),vec![next]);
    }
}
