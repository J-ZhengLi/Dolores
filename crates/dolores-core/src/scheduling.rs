//! Named-zone, source-bound scheduling. No tool authority is conferred here.
use chrono::{Datelike, Duration, LocalResult, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackgroundPolicy {
    pub revision: u32,
    pub enabled: bool,
}
impl Default for BackgroundPolicy {
    fn default() -> Self {
        Self {
            revision: 1,
            enabled: false,
        }
    }
}

pub const MAX_SCHEDULES: usize = 32;
pub const MAX_OCCURRENCES: usize = 50;
pub const LATE_SECONDS: i64 = 120;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
        self.zone
            .parse::<Tz>()
            .map_err(|_| "Choose a named timezone, such as Asia/Shanghai.")?;
        NaiveTime::parse_from_str(&self.time, "%H:%M")
            .map_err(|_| "Specify a time with AM/PM or use HH:MM.")?;
        if self.time.len() != 5
            || self.weekdays.len() > 7
            || self.weekdays.iter().any(|d| *d > 6)
            || self
                .weekdays
                .iter()
                .enumerate()
                .any(|(i, d)| self.weekdays[..i].contains(d))
        {
            return Err("Invalid schedule time or weekdays.".into());
        }
        match self.kind.as_str() {
            "once" if self.weekdays.is_empty() => {
                NaiveDate::parse_from_str(
                    self.date
                        .as_deref()
                        .ok_or("Specify the date for this task.")?,
                    "%Y-%m-%d",
                )
                .map_err(|_| "Specify the date as YYYY-MM-DD.")?;
            }
            "daily" if self.date.is_none() && self.weekdays.is_empty() => {}
            "weekdays" if self.date.is_none() && !self.weekdays.is_empty() => {}
            _ => return Err("Choose one date, daily, or selected weekdays.".into()),
        }
        Ok(())
    }
    fn instant(&self, date: NaiveDate) -> Result<i64, String> {
        let zone = self
            .zone
            .parse::<Tz>()
            .map_err(|_| "Timezone unavailable.")?;
        let local = date.and_time(
            NaiveTime::parse_from_str(&self.time, "%H:%M").map_err(|_| "Time unavailable.")?,
        );
        for minute in 0..=1560 {
            let candidate = local
                .checked_add_signed(Duration::minutes(minute))
                .ok_or("Date is out of range.")?;
            match zone.from_local_datetime(&candidate) {
                LocalResult::Single(t) => return Ok(t.timestamp()),
                LocalResult::Ambiguous(a, b) => return Ok(a.timestamp().min(b.timestamp())),
                LocalResult::None => {}
            }
        }
        Err("This local time is unavailable. Choose another time.".into())
    }
    pub fn next_after(&self, after: i64) -> Result<Option<i64>, String> {
        self.validate()?;
        if self.kind == "once" {
            let t = self.instant(
                NaiveDate::parse_from_str(self.date.as_ref().unwrap(), "%Y-%m-%d")
                    .map_err(|_| "Date unavailable.")?,
            )?;
            return Ok((t > after).then_some(t));
        }
        let zone = self
            .zone
            .parse::<Tz>()
            .map_err(|_| "Timezone unavailable.")?;
        let date = chrono::DateTime::from_timestamp(after, 0)
            .ok_or("Clock is out of range.")?
            .with_timezone(&zone)
            .date_naive();
        for day in 0..=370 {
            let candidate = date
                .checked_add_signed(Duration::days(day))
                .ok_or("Date is out of range.")?;
            if self.kind == "daily"
                || self
                    .weekdays
                    .contains(&candidate.weekday().num_days_from_monday())
            {
                let t = self.instant(candidate)?;
                if t > after {
                    return Ok(Some(t));
                }
            }
        }
        Err("No next occurrence found. Review the schedule.".into())
    }
    pub fn describe(&self) -> String {
        let day = match self.kind.as_str() {
            "once" => self.date.clone().unwrap_or_default(),
            "daily" => "Every day".into(),
            _ => self
                .weekdays
                .iter()
                .map(|d| ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][*d as usize])
                .collect::<Vec<_>>()
                .join(", "),
        };
        format!("{day} at {} ({})", self.time, self.zone)
    }
}
pub fn now_seconds() -> i64 {
    Utc::now().timestamp()
}

/// Source provenance and size are host checks. Meaning is interpreted by the
/// configured model in any language; lexical allowlists cannot establish intent.
pub fn validate_source_rule(
    input: &str,
    rule: &ScheduleRule,
    _system_zone: &str,
) -> Result<(), String> {
    validate_source_input(input)?;
    rule.validate()
}
pub fn validate_source_input(input: &str) -> Result<(), String> {
    if input.trim().is_empty() || input.len() > 4096 {
        return Err("Task requests need a nonempty message of at most 4 KiB.".into());
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduledTask {
    pub id: String,
    pub revision: u32,
    pub source_session: String,
    pub source_key: String,
    pub title: String,
    pub prompt: String,
    pub workspace: crate::SessionWorkspace,
    pub rule: ScheduleRule,
    pub next_due: Option<i64>,
    pub paused: bool,
    pub deleted: bool,
    pub preferences: crate::ConnectionPreferences,
    pub skill: Option<crate::ProjectSkill>,
    pub effective: crate::EffectiveSettings,
}
impl ScheduledTask {
    pub fn validate(&self) -> Result<(), String> {
        self.rule.validate()?;
        if self.id.is_empty()
            || self.revision == 0
            || self.source_key.is_empty()
            || self.prompt.is_empty()
            || self.prompt.len() > 4096
            || self.title.len() > 160
        {
            return Err("Invalid scheduled task.".into());
        }
        if let Some(s) = &self.skill {
            s.validate()?;
            if !s.enabled || s.versions.len() != 1 {
                return Err("Scheduled tasks pin one enabled skill version.".into());
            }
        }
        self.effective.validate()
    }
}
pub fn pin_skill(skill: &crate::ProjectSkill) -> crate::ProjectSkill {
    let mut snapshot = skill.clone();
    snapshot.versions = vec![skill.current().clone()];
    snapshot
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledOccurrence {
    pub id: String,
    pub task: String,
    pub due: i64,
    pub state: String,
    pub session: Option<String>,
    pub run: Option<u64>,
    pub lease_until: i64,
    pub snapshot: ScheduledTask,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rule(zone: &str, time: &str) -> ScheduleRule {
        ScheduleRule {
            kind: "daily".into(),
            time: time.into(),
            date: None,
            weekdays: vec![],
            zone: zone.into(),
        }
    }
    fn utc(value: &str) -> i64 {
        chrono::DateTime::parse_from_rfc3339(value)
            .unwrap()
            .timestamp()
    }
    #[test]
    fn daylight_gap_overlap_and_weekday_clock() {
        let gap = rule("America/New_York", "02:30");
        assert_eq!(
            gap.next_after(utc("2026-03-08T00:00:00Z")).unwrap(),
            Some(utc("2026-03-08T07:00:00Z"))
        );
        let fold = rule("America/New_York", "01:30");
        assert_eq!(
            fold.next_after(utc("2026-11-01T00:00:00Z")).unwrap(),
            Some(utc("2026-11-01T05:30:00Z"))
        );
        assert_eq!(
            fold.next_after(utc("2026-11-01T05:30:00Z")).unwrap(),
            Some(utc("2026-11-02T06:30:00Z"))
        );
        let mut r = rule("Asia/Shanghai", "21:00");
        r.kind = "weekdays".into();
        r.weekdays = vec![0, 1, 2, 3, 4];
        assert_eq!(
            r.next_after(utc("2026-10-09T14:00:00Z")).unwrap(),
            Some(utc("2026-10-12T13:00:00Z"))
        );
    }
    #[test]
    fn invalid_source_and_structured_rules_do_not_create_tasks() {
        let mut r = rule("Asia/Shanghai", "21:00");
        for input in ["".to_owned(), " ".to_owned(), "x".repeat(4097)] {
            assert!(validate_source_rule(&input, &r, "Asia/Shanghai").is_err());
        }
        r.time = "29:00".into();
        assert!(validate_source_rule("Redacta un informe cada día", &r, "Asia/Shanghai").is_err());
        r.time = "09:00".into();
        r.kind = "weekdays".into();
        r.weekdays = vec![0, 0];
        assert!(validate_source_rule("平日にレポートを書いて", &r, "Asia/Shanghai").is_err());
        r.weekdays = vec![0, 1, 2, 3, 4];
        r.zone = "unknown".into();
        assert!(validate_source_rule("Write a report each workday", &r, "Asia/Shanghai").is_err());
    }
    #[test]
    fn source_validation_does_not_require_a_language_or_a_stated_time() {
        let mut r = rule("Asia/Shanghai", "09:00");
        r.kind = "weekdays".into();
        r.weekdays = vec![0, 1, 2, 3, 4];
        for input in [
            "Write a report each workday",
            "Redacta un informe cada día laborable a las nueve de la mañana",
            "平日の朝九時にレポートを書いてください",
            "Rédige un rapport tous les jours ouvrables à neuf heures",
            "اكتب تقريرًا كل يوم عمل في التاسعة صباحًا",
        ] {
            assert!(
                validate_source_rule(input, &r, "Asia/Shanghai").is_ok(),
                "{input}"
            );
        }
    }
}
