//! Named-zone, source-bound scheduling. No tool authority is conferred here.
use chrono::{Datelike, Duration, LocalResult, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

pub const MAX_SCHEDULES: usize = 32;
pub const MAX_OCCURRENCES: usize = 50;
pub const LATE_SECONDS: i64 = 120;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct ScheduleRule {
    pub kind: String,
    pub time: String,
    pub date: Option<String>,
    #[serde(default)]
    pub weekdays: Vec<u32>, // Monday = 0
    pub zone: String,
}
impl ScheduleRule {
    pub fn validate(&self) -> Result<(), String> {
        self.zone.parse::<Tz>().map_err(|_| "Choose a named timezone, such as Asia/Shanghai.")?;
        NaiveTime::parse_from_str(&self.time,"%H:%M").map_err(|_| "Specify a time with AM/PM or use HH:MM.")?;
        if self.time.len()!=5 || self.weekdays.len()>7 || self.weekdays.iter().any(|d|*d>6) || self.weekdays.iter().enumerate().any(|(i,d)|self.weekdays[..i].contains(d)) {
            return Err("Invalid schedule time or weekdays.".into());
        }
        match self.kind.as_str() {
            "once" if self.weekdays.is_empty() => { NaiveDate::parse_from_str(self.date.as_deref().ok_or("Specify the date for this task.")?,"%Y-%m-%d").map_err(|_|"Specify the date as YYYY-MM-DD.")?; }
            "daily" if self.date.is_none() && self.weekdays.is_empty() => {},
            "weekdays" if self.date.is_none() && !self.weekdays.is_empty() => {},
            _ => return Err("Choose one date, daily, or selected weekdays.".into()),
        }
        Ok(())
    }
    fn instant(&self, date: NaiveDate) -> Result<i64,String> {
        let zone=self.zone.parse::<Tz>().map_err(|_|"Timezone unavailable.")?;
        let local=date.and_time(NaiveTime::parse_from_str(&self.time,"%H:%M").map_err(|_|"Time unavailable.")?);
        for minute in 0..=1560 {
            let candidate=local.checked_add_signed(Duration::minutes(minute)).ok_or("Date is out of range.")?;
            match zone.from_local_datetime(&candidate) {
                LocalResult::Single(t)=>return Ok(t.timestamp()),
                LocalResult::Ambiguous(a,b)=>return Ok(a.timestamp().min(b.timestamp())),
                LocalResult::None=>{},
            }
        }
        Err("This local time is unavailable. Choose another time.".into())
    }
    pub fn next_after(&self, after:i64)->Result<Option<i64>,String> {
        self.validate()?;
        if self.kind=="once" {
            let t=self.instant(NaiveDate::parse_from_str(self.date.as_ref().unwrap(),"%Y-%m-%d").map_err(|_|"Date unavailable.")?)?;
            return Ok((t>after).then_some(t));
        }
        let zone=self.zone.parse::<Tz>().map_err(|_|"Timezone unavailable.")?;
        let date=chrono::DateTime::from_timestamp(after,0).ok_or("Clock is out of range.")?.with_timezone(&zone).date_naive();
        for day in 0..=370 {
            let candidate=date.checked_add_signed(Duration::days(day)).ok_or("Date is out of range.")?;
            if self.kind=="daily" || self.weekdays.contains(&candidate.weekday().num_days_from_monday()) {
                let t=self.instant(candidate)?;
                if t>after { return Ok(Some(t)); }
            }
        }
        Err("No next occurrence found. Review the schedule.".into())
    }
    pub fn describe(&self)->String {
        let day=match self.kind.as_str(){"once"=>self.date.clone().unwrap_or_default(),"daily"=>"Every day".into(),_=>self.weekdays.iter().map(|d|["Mon","Tue","Wed","Thu","Fri","Sat","Sun"][*d as usize]).collect::<Vec<_>>().join(", ")};
        format!("{day} at {} ({})",self.time,self.zone)
    }
}
pub fn now_seconds()->i64 { Utc::now().timestamp() }

/// Intentionally conservative: discussion is recoverable; unauthorized creation is not.
pub fn explicit_schedule_intent(input:&str)->bool {
    let s=input.to_lowercase();
    if s.len()>4096 || s.contains(['"','“','”','`','\n','?','？']) ||
        ["don't","do not","not create","not schedule","example","for instance","hypothetical","what if","could you explain","how do","discuss","quoted","someone said","不","举例","例如","假设","讨论","解释"].iter().any(|v|s.contains(v)) { return false; }
    let action=["schedule","remind me","write","generate","prepare","run","summarize","every","每天","每个","每周","定时","提醒","生成","写"].iter().any(|v|s.contains(v));
    let clock=regex::Regex::new(r"(?i)(\b\d{1,2}:\d{2}\b|\b\d{1,2}\s*[ap]m\b|\d{1,2}点)").unwrap().is_match(&s);
    action && clock
}
pub fn validate_source_rule(input:&str,rule:&ScheduleRule,system_zone:&str)->Result<(),String> {
    rule.validate()?;
    if !explicit_schedule_intent(input) {return Err("Ask directly to schedule a task, with a clear time. Quoted examples don't create tasks.".into());}
    let s=input.to_lowercase();
    let compact=s.replace(' ',"");
    let hour=rule.time[..2].parse::<u32>().map_err(|_|"Invalid hour.")?;
    let minute=rule.time[3..].parse::<u32>().map_err(|_|"Invalid minute.")?;
    let suffix=if hour<12{"am"}else{"pm"};
    let twelve=if hour.is_multiple_of(12){12}else{hour%12};
    let mut spellings=vec![rule.time.clone(),format!("{hour}:{minute:02}"),format!("{twelve}:{minute:02}{suffix}")];
    if minute==0 {spellings.extend([format!("{twelve}{suffix}"),format!("{hour}点")]);}
    let source_times=regex::Regex::new(r"(?i)(\b\d{1,2}:\d{2}(?:\s*[ap]m)?\b|\b\d{1,2}\s*[ap]m\b|\d{1,2}点)").unwrap().find_iter(&s).map(|v|v.as_str().replace(' ',"")).collect::<Vec<_>>();
    if source_times.len()!=1 || !spellings.contains(&source_times[0]) { return Err("Clarify one exact time with AM/PM or HH:MM.".into()); }
    if rule.zone!=system_zone && !s.contains(&rule.zone.to_lowercase()) {return Err("Name the timezone in your request before scheduling there.".into());}
    match rule.kind.as_str() {
        "once" if compact.contains(rule.date.as_ref().unwrap()) => {},
        "daily" if ["everyday","daily","每天"].iter().any(|v|compact.contains(v)) => {},
        "weekdays" => {
            let expected=if ["everyweekday","weekdays","工作日"].iter().any(|v|compact.contains(v)){vec![0,1,2,3,4]}else{
                ["monday","tuesday","wednesday","thursday","friday","saturday","sunday"].iter().enumerate().filter_map(|(i,v)|s.contains(v).then_some(i as u32)).collect()};
            let mut actual=rule.weekdays.clone();actual.sort_unstable();
            if expected.is_empty() || actual!=expected {return Err("Specify which weekdays this task should run.".into());}
        },
        _ => return Err("Specify a date, every day, or the weekdays.".into()),
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct ScheduledTask {
    pub id:String, pub revision:u32, pub source_session:String, pub source_key:String,
    pub title:String, pub prompt:String, pub workspace:crate::SessionWorkspace,
    pub rule:ScheduleRule, pub next_due:Option<i64>, pub paused:bool, pub deleted:bool,
    pub preferences:crate::ConnectionPreferences, pub skill:Option<crate::ProjectSkill>,
    pub effective:crate::EffectiveSettings,
}
impl ScheduledTask {
    pub fn validate(&self)->Result<(),String>{
        self.rule.validate()?;
        if self.id.is_empty() || self.revision==0 || self.source_key.is_empty() || self.prompt.is_empty() || self.prompt.len()>4096 || self.title.len()>160 {return Err("Invalid scheduled task.".into());}
        if let Some(s)=&self.skill{s.validate()?;}
        self.effective.validate()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="camelCase")]
pub struct ScheduledOccurrence {
    pub id:String, pub task:String, pub due:i64, pub state:String,
    pub session:Option<String>, pub run:Option<u64>, pub lease_until:i64,
    pub snapshot:ScheduledTask, pub error:Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rule(zone:&str,time:&str)->ScheduleRule{ScheduleRule{kind:"daily".into(),time:time.into(),date:None,weekdays:vec![],zone:zone.into()}}
    fn utc(value:&str)->i64{chrono::DateTime::parse_from_rfc3339(value).unwrap().timestamp()}
    #[test] fn daylight_gap_overlap_and_weekday_clock(){
        let gap=rule("America/New_York","02:30");
        assert_eq!(gap.next_after(utc("2026-03-08T00:00:00Z")).unwrap(),Some(utc("2026-03-08T07:00:00Z")));
        let fold=rule("America/New_York","01:30");
        assert_eq!(fold.next_after(utc("2026-11-01T00:00:00Z")).unwrap(),Some(utc("2026-11-01T05:30:00Z")));
        assert_eq!(fold.next_after(utc("2026-11-01T05:30:00Z")).unwrap(),Some(utc("2026-11-02T06:30:00Z")));
        let mut r=rule("Asia/Shanghai","21:00");r.kind="weekdays".into();r.weekdays=vec![0,1,2,3,4];
        assert_eq!(r.next_after(utc("2026-10-09T14:00:00Z")).unwrap(),Some(utc("2026-10-12T13:00:00Z")));
    }
    #[test] fn source_intent_refuses_quotes_and_invented_rule(){
        let r=rule("Asia/Shanghai","21:00");
        assert!(validate_source_rule("Write my report every day at 9pm",&r,"Asia/Shanghai").is_ok());
        for s in ["Example: write my report daily at 9pm","Do not schedule a report daily at 9pm","What if we schedule daily at 9pm?","\"write daily at 9pm\"","Write my report daily at 8pm","Write daily at 9am and 9pm"]{assert!(validate_source_rule(s,&r,"Asia/Shanghai").is_err(),"{s}");}
    }
}
