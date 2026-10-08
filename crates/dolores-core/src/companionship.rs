//! Quiet, persisted eligibility. No tools or background task authority.
use chrono::{Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

pub const MIN_GAP: i64 = 3 * 3600;
pub const MAX_DAILY_CAP: u8 = 100;
pub const EXPIRY: i64 = 15 * 60;
pub const MAX_ACTIVITY: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompanionPolicy {
    pub revision: u32,
    pub enabled: bool,
    pub model: String,
    pub zone: String,
    pub start: u16,
    pub end: u16,
    pub daily_cap: u8,
}
impl Default for CompanionPolicy {
    fn default() -> Self {
        Self {
            revision: 1,
            enabled: false,
            model: String::new(),
            zone: "UTC".into(),
            start: 540,
            end: 1260,
            daily_cap: 2,
        }
    }
}
impl CompanionPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.revision == 0
            || self.model.len() > 200
            || self.model.chars().any(char::is_control)
            || self.start >= 1440
            || self.end >= 1440
            || self.start == self.end
            || self.daily_cap > MAX_DAILY_CAP
        {
            return Err("Choose valid hours and a daily limit from 0 to 100.".into());
        }
        self.zone
            .parse::<Tz>()
            .map_err(|_| "Choose a named timezone.")?;
        Ok(())
    }
    /// Spread higher frequencies across the allowed window, retaining the
    /// original three-hour spacing at low frequencies. Never queue catch-up.
    pub fn gap(&self) -> i64 {
        let minutes = (i64::from(self.end) - i64::from(self.start)).rem_euclid(1440);
        (minutes * 60 / i64::from(self.daily_cap.max(1))).clamp(60, MIN_GAP)
    }

    pub fn local(&self, now: i64) -> Result<(String, bool), String> {
        let zone = self
            .zone
            .parse::<Tz>()
            .map_err(|_| "Timezone unavailable.")?;
        let local = chrono::DateTime::from_timestamp(now, 0)
            .ok_or("Clock unavailable.")?
            .with_timezone(&zone);
        let minute = (local.hour() * 60 + local.minute()) as u16;
        let allowed = if self.start < self.end {
            minute >= self.start && minute < self.end
        } else {
            minute >= self.start || minute < self.end
        };
        Ok((local.date_naive().to_string(), allowed))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompanionSource {
    pub root: Option<String>,
    pub memory: String,
    pub signature: String,
    pub session: String,
    pub message_id: i64,
    pub quote: String,
    pub kind: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompanionCandidate {
    pub id: String,
    pub policy_revision: u32,
    pub created: i64,
    pub expires: i64,
    pub workspace: crate::SessionWorkspace,
    pub kind: String,
    pub body: String,
    pub citation: Option<String>,
    pub source: Option<CompanionSource>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompanionActivity {
    pub id: String,
    pub at: i64,
    pub status: String,
    pub kind: String,
    pub session: Option<String>,
    pub seen: bool,
    pub note: Option<String>,
    pub source: Option<CompanionSource>,
    pub citation: Option<String>,
    pub usage: Option<crate::TokenUsage>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompanionState {
    pub revision: u64,
    pub policy: CompanionPolicy,
    pub day: String,
    pub attempts: u8,
    pub last_attempt: Option<i64>,
    pub next_opportunity: Option<i64>,
    pub snooze_until: i64,
    pub pending: Option<CompanionCandidate>,
    pub activity: Vec<CompanionActivity>,
}
impl CompanionState {
    pub fn validate(&self) -> Result<(), String> {
        self.policy.validate()?;
        if self.activity.len() > MAX_ACTIVITY || self.attempts > MAX_DAILY_CAP {
            return Err("Companion state is invalid.".into());
        }
        if let Some(c) = &self.pending {
            if c.id.is_empty() || c.body.len() > 2048 || c.expires <= c.created {
                return Err("Companion candidate is invalid.".into());
            }
        }
        Ok(())
    }
    pub fn unread(&self) -> Option<&CompanionActivity> {
        self.activity
            .iter()
            .rev()
            .find(|a| a.status == "delivered" && !a.seen)
    }
    pub fn reserve(
        &mut self,
        now: i64,
        present: bool,
        busy: bool,
        jitter: u32,
        candidate: CompanionCandidate,
    ) -> Result<bool, String> {
        self.validate()?;
        if !self.policy.enabled
            || self.policy.model.is_empty()
            || self.policy.daily_cap == 0
            || !present
            || busy
            || self.snooze_until > now
            || self.unread().is_some()
        {
            return Ok(false);
        }
        if self.pending.as_ref().is_some_and(|c| c.expires > now) {
            return Ok(false);
        }
        self.pending = None;
        let (day, allowed) = self.policy.local(now)?;
        if !allowed
            || (!self.day.is_empty() && day < self.day)
            || self
                .last_attempt
                .is_some_and(|last| now.saturating_sub(last) < self.policy.gap())
        {
            return Ok(false);
        }
        if self.day != day {
            self.day = day;
            self.attempts = 0;
        }
        if self.attempts >= self.policy.daily_cap {
            return Ok(false);
        }
        if self.next_opportunity.is_none()
            || self
                .next_opportunity
                .is_some_and(|due| now.saturating_sub(due) > EXPIRY)
        {
            let gap = self.policy.gap();
            let earliest = (gap / 4).clamp(60, 300);
            let latest = gap.clamp(60, 1800);
            self.next_opportunity =
                Some(now.saturating_add(earliest + i64::from(jitter) % (latest - earliest + 1)));
            return Ok(false);
        }
        if self.next_opportunity.is_some_and(|due| due > now) {
            return Ok(false);
        }
        if candidate.policy_revision != self.policy.revision
            || candidate.created != now
            || candidate.expires != now.saturating_add(EXPIRY)
        {
            return Err("Companion candidate changed.".into());
        }
        self.attempts += 1;
        self.last_attempt = Some(now);
        let gap = self.policy.gap();
        let spread = (gap / 3).min(3600);
        self.next_opportunity = Some(now.saturating_add(gap + i64::from(jitter) % (spread + 1)));
        self.activity.push(CompanionActivity {
            id: candidate.id.clone(),
            at: now,
            status: "generating".into(),
            kind: candidate.kind.clone(),
            session: None,
            seen: true,
            note: None,
            source: candidate.source.clone(),
            citation: candidate.citation.clone(),
            usage: None,
        });
        if self.activity.len() > MAX_ACTIVITY {
            self.activity.remove(0);
        }
        self.pending = Some(candidate);
        Ok(true)
    }
}
pub fn now() -> i64 {
    Utc::now().timestamp()
}

pub fn invitation(answer: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Answer {
        invitation: String,
    }
    let text = serde_json::from_str::<Answer>(answer)
        .map_err(|_| "The model returned an unusable message.")?
        .invitation;
    let lower = text.to_lowercase();
    if text.trim() != text
        || text.chars().count() > 240
        || text.chars().count() < 8
        || !text.ends_with('?')
        || text.contains(['\n', '\r'])
        || [
            "remember when",
            "remember that",
            "i feel",
            "i miss",
            "i need you",
            "only me",
            "only i",
            "you owe",
            "don't leave",
            "you forgot",
            "as we discussed",
            "our last",
            "yesterday",
            "http:",
            "https:",
        ]
        .iter()
        .any(|s| lower.contains(s))
    {
        return Err("The model returned an unsuitable message.".into());
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> CompanionState {
        CompanionState {
            policy: CompanionPolicy {
                enabled: true,
                model: "weak".into(),
                zone: "Asia/Shanghai".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    fn candidate(now: i64) -> CompanionCandidate {
        CompanionCandidate {
            id: "c".into(),
            policy_revision: 1,
            created: now,
            expires: now + EXPIRY,
            workspace: Default::default(),
            kind: "chat".into(),
            body: String::new(),
            citation: None,
            source: None,
        }
    }
    #[test]
    fn frequency_range_and_quiet_do_not_reset_consumed_attempts() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-07T10:00:00+08:00")
            .unwrap()
            .timestamp();
        let mut s = state();
        s.attempts = 100;
        s.policy.daily_cap = 100;
        assert!(s.validate().is_ok());
        s.policy.daily_cap = 101;
        assert!(s.validate().is_err());
        s.policy.daily_cap = 0;
        let quiet = s.clone();
        assert!(!s.reserve(now, true, false, 0, candidate(now)).unwrap());
        assert_eq!(s, quiet);
        s.policy.daily_cap = 2;
        s.day = s.policy.local(now).unwrap().0;
        s.next_opportunity = Some(now);
        assert!(!s.reserve(now, true, false, 0, candidate(now)).unwrap());
        assert_eq!(s.attempts, 100);
        let restored: CompanionState =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(restored, s);
        assert!(restored.validate().is_ok());
    }

    #[test]
    fn chatty_frequency_admits_100_spaced_attempts_and_stops_at_daily_cap() {
        let start = chrono::DateTime::parse_from_rfc3339("2026-10-07T09:00:00+08:00")
            .unwrap()
            .timestamp();
        let mut s = state();
        s.policy.daily_cap = 100;
        let gap = s.policy.gap();
        assert_eq!(gap, 432);
        s.next_opportunity = Some(start);
        for n in 0..100 {
            let now = start + n * gap;
            assert!(s.reserve(now, true, false, 0, candidate(now)).unwrap());
            // Failed generation consumes an attempt just like delivery.
            s.pending = None;
            s.activity.last_mut().unwrap().status = "failed".into();
            assert!(!s
                .reserve(now + 1, true, false, 0, candidate(now + 1))
                .unwrap());
        }
        assert_eq!(s.attempts, 100);
        assert_eq!(s.activity.len(), MAX_ACTIVITY);
        s.last_attempt = None;
        let now = start + 99 * gap + 1;
        s.next_opportunity = Some(now);
        assert!(!s.reserve(now, true, false, 0, candidate(now)).unwrap());
        let tomorrow = start + 86400;
        s.next_opportunity = Some(tomorrow);
        assert!(s
            .reserve(tomorrow, true, false, 0, candidate(tomorrow))
            .unwrap());
        assert_eq!(s.attempts, 1);
        s.policy.start = 1260;
        s.policy.end = 540;
        assert_eq!(s.policy.gap(), gap);
    }

    #[test]
    fn absence_busy_and_cooldown_do_not_accumulate_messages() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-07T10:00:00+08:00")
            .unwrap()
            .timestamp();
        let mut s = state();
        assert!(!s.reserve(now, false, false, 0, candidate(now)).unwrap());
        assert!(s.next_opportunity.is_none());
        assert!(!s.reserve(now, true, true, 0, candidate(now)).unwrap());
        assert!(!s.reserve(now, true, false, 0, candidate(now)).unwrap());
        let due = now + 300;
        assert!(s.reserve(due, true, false, 0, candidate(due)).unwrap());
        s.pending = None;
        assert!(!s
            .reserve(due + 30, true, false, 0, candidate(due + 30))
            .unwrap());
        assert!(!s
            .reserve(now - 86400, true, false, 0, candidate(now - 86400))
            .unwrap());
        assert_eq!(s.attempts, 1);
    }
    #[test]
    fn local_hours_dst_and_unread_messages_keep_quiet() {
        let mut s = state();
        s.policy.zone = "America/New_York".into();
        s.policy.start = 60;
        s.policy.end = 180;
        for time in ["2026-11-01T05:30:00Z", "2026-11-01T06:30:00Z"] {
            assert!(
                s.policy
                    .local(
                        chrono::DateTime::parse_from_rfc3339(time)
                            .unwrap()
                            .timestamp()
                    )
                    .unwrap()
                    .1
            );
        }
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-07T10:00:00-04:00")
            .unwrap()
            .timestamp();
        assert!(!s.reserve(now, true, false, 0, candidate(now)).unwrap());
        s.policy.start = 540;
        s.policy.end = 1260;
        s.next_opportunity = Some(now);
        assert!(s.reserve(now, true, false, 0, candidate(now)).unwrap());
        s.pending = None;
        s.activity[0].status = "delivered".into();
        s.activity[0].seen = false;
        assert!(!s
            .reserve(now + MIN_GAP, true, false, 0, candidate(now + MIN_GAP))
            .unwrap());
    }
    #[test]
    fn overdue_opportunity_reschedules_instead_of_catching_up() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-07T10:00:00+08:00")
            .unwrap()
            .timestamp();
        let mut s = state();
        s.next_opportunity = Some(now - 86400);
        assert!(!s.reserve(now, true, false, 0, candidate(now)).unwrap());
        assert_eq!(s.next_opportunity, Some(now + 300));
        assert_eq!(s.attempts, 0);
    }
}
