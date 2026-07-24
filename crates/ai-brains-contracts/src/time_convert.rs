use chrono::{DateTime, TimeZone, Utc};
use time::OffsetDateTime;

/// Single shared domain→contract timestamp conversion (T148 R2).
///
/// Converts via unix timestamp nanos. Falls back to epoch if the instant is
/// outside chrono's representable range (should not occur for vault timestamps).
pub fn offset_to_utc(t: OffsetDateTime) -> DateTime<Utc> {
    let nanos = t.unix_timestamp_nanos();
    let secs = (nanos / 1_000_000_000) as i64;
    let nsecs = (nanos.rem_euclid(1_000_000_000)) as u32;
    match Utc.timestamp_opt(secs, nsecs) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(a, _) => a,
        chrono::LocalResult::None => DateTime::<Utc>::UNIX_EPOCH,
    }
}
