use crate::storage_error;
use dolores_core::scheduling::*;
use rusqlite::{Connection,OptionalExtension,params};
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
fn terminal(state:&str)->bool{matches!(state,"succeeded"|"failed"|"interrupted"|"cancelled"|"missed"|"skipped"|"paused")}
pub(super) fn claim(c:&mut Connection,id:&str,revision:u32,now:i64,manual:bool)->Result<Option<ScheduledOccurrence>,String>{
    let tx=c.transaction().map_err(storage_error)?;
    let mut task=tasks(&tx)?.into_iter().find(|t|t.id==id).ok_or("Task unavailable. Refresh Scheduled.")?;
    if task.revision!=revision || task.deleted{return Err("Task changed. Refresh Scheduled.").map_err(str::to_string);}
    if history(&tx,id)?.iter().any(|o|!terminal(&o.state)){return Ok(None);}
    if !manual && task.paused{return Ok(None);}
    let due=if manual{now}else{match task.next_due{Some(t) if t<=now=>t,_=>return Ok(None)}};
    let occurrence_id=if manual{format!("{id}:manual:{}",uuid::Uuid::new_v4())}else{format!("{id}:{}:{due}",task.revision)};
    let duplicate:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM scheduled_occurrences WHERE id=?1)",[&occurrence_id],|r|r.get(0)).map_err(storage_error)?;
    if duplicate{return Ok(None);}
    let missed=!manual && now.saturating_sub(due)>LATE_SECONDS;
    let occurrence=ScheduledOccurrence{id:occurrence_id,task:id.into(),due,state:if missed{"missed"}else{"claimed"}.into(),session:None,run:None,lease_until:now.saturating_add(LATE_SECONDS),snapshot:task.clone(),error:missed.then(||"Missed while Dolores was closed or unavailable. Use Run now if still needed.".into())};
    tx.execute("INSERT INTO scheduled_occurrences(id,task,due,data) VALUES(?1,?2,?3,?4)",params![occurrence.id,id,due,serde_json::to_string(&occurrence).map_err(storage_error)?]).map_err(storage_error)?;
    if !manual {task.next_due=task.rule.next_after(now.max(due))?;}
    task.revision=task.revision.checked_add(1).ok_or("Task revision exhausted.")?;
    tx.execute("UPDATE scheduled_tasks SET data=?2 WHERE id=?1",params![id,serde_json::to_string(&task).map_err(storage_error)?]).map_err(storage_error)?;
    tx.execute("DELETE FROM scheduled_occurrences WHERE task=?1 AND id NOT IN (SELECT id FROM scheduled_occurrences WHERE task=?1 ORDER BY due DESC,id DESC LIMIT 50) AND json_extract(data,'$.state') IN ('succeeded','failed','interrupted','cancelled','missed','skipped','paused')",[id]).map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;Ok(Some(occurrence))
}
pub(super) fn update(c:&mut Connection,o:&ScheduledOccurrence,expected:&str)->Result<(),String>{
    let tx=c.transaction().map_err(storage_error)?;
    let old:Option<String>=tx.query_row("SELECT data FROM scheduled_occurrences WHERE id=?1",[&o.id],|r|r.get(0)).optional().map_err(storage_error)?;
    let old:ScheduledOccurrence=serde_json::from_str(&old.ok_or("Occurrence unavailable.")?).map_err(storage_error)?;
    if old.state!=expected || terminal(&old.state) || old.snapshot!=o.snapshot || old.task!=o.task || old.due!=o.due{return Err("Occurrence changed. Refresh Scheduled.".into());}
    tx.execute("UPDATE scheduled_occurrences SET data=?2 WHERE id=?1",params![o.id,serde_json::to_string(o).map_err(storage_error)?]).map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;Ok(())
}
pub(super) fn recover(c:&mut Connection)->Result<(),String>{
    let tx=c.transaction().map_err(storage_error)?;
    tx.execute("UPDATE scheduled_occurrences SET data=json_set(data,'$.state','interrupted','$.error','Dolores closed before this run finished. Inspect its result before using Run now.') WHERE json_extract(data,'$.state') NOT IN ('succeeded','failed','interrupted','cancelled','missed','skipped','paused')",[]).map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;Ok(())
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
    #[test] fn duplicate_ticks_restart_and_backward_clock_never_replay(){
        let store=SqliteStore::open(std::path::Path::new(":memory:")).unwrap();let t=store.save_scheduled_task(&task(),None).unwrap();let now=t.next_due.unwrap();
        let o=store.claim_scheduled_occurrence(&t.id,t.revision,now,false).unwrap().unwrap();
        assert!(store.claim_scheduled_occurrence(&t.id,t.revision,now,false).is_err());
        store.recover_scheduled_occurrences().unwrap();assert_eq!(store.scheduled_occurrences(&t.id).unwrap()[0].state,"interrupted");
        let revision=store.scheduled_tasks().unwrap()[0].revision;
        assert!(store.claim_scheduled_occurrence(&t.id,revision,now-300,false).unwrap().is_none());
        assert!(store.update_scheduled_occurrence(&o,"claimed").is_err());
    }
    #[test] fn missed_suspend_skips_once_and_claim_write_failure_is_atomic(){
        let store=SqliteStore::open(std::path::Path::new(":memory:")).unwrap();let t=store.save_scheduled_task(&task(),None).unwrap();let now=t.next_due.unwrap()+86400*20;
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_claim BEFORE INSERT ON scheduled_occurrences BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store.claim_scheduled_occurrence(&t.id,t.revision,now,false).is_err());assert_eq!(store.scheduled_tasks().unwrap()[0].next_due,t.next_due);
        store.lock().unwrap().execute_batch("DROP TRIGGER reject_claim;").unwrap();
        assert_eq!(store.claim_scheduled_occurrence(&t.id,t.revision,now,false).unwrap().unwrap().state,"missed");
        let current=store.scheduled_tasks().unwrap().remove(0);
        assert!(current.next_due.unwrap()>now);assert!(store.claim_scheduled_occurrence(&t.id,current.revision,now,false).unwrap().is_none());assert_eq!(store.scheduled_occurrences(&t.id).unwrap().len(),1);
    }
}
