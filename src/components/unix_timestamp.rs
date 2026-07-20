use dioxus::prelude::*;

use crate::components::common::output_with_copy::OutputWithCopy;
use crate::components::common::pill_select::PillSelect;

/// Unix-timestamp ↔ date converter.
#[component]
pub fn UnixTimestamp() -> Element {
    let mut input = use_signal(String::new);
    let mut tz = use_signal(|| {
        let local = get_local_offset_minutes() / 60;
        Timezone::nearest(local).unwrap_or(Timezone::Shanghai)
    });

    let output = use_memo(move || do_convert(&*input.read(), *tz.read()));

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 p-6 select-none",
            div { class: "flex flex-col gap-2",
                label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400",
                    "Input"
                }
                textarea {
                    id: "timestamp-input",
                    class: "min-h-[80px] w-full resize-y rounded-lg border border-gray-200 bg-white p-4 font-mono text-sm text-gray-900 focus:outline-none focus:border-gray-400 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 select-text",
                    spellcheck: false,
                    placeholder: "Unix timestamp (e.g. 1700000000) or ISO date (e.g. 2023-11-15T07:00:00Z) ...",
                    oninput: move |e: FormEvent| input.set(e.value()),
                }
            }

            PillSelect {
                label: "Timezone",
                options: Timezone::all_options(),
                selected: tz.read().name().to_string(),
                onchange: move |v: String| {
                    if let Some(t) = Timezone::all().iter().find(|t| t.name() == v) {
                        tz.set(*t);
                    }
                },
            }

            if !output.read().is_empty() {
                OutputWithCopy { output }
            }
        }
    }
}

// ── Timezones (IANA names, no hardcoded offsets) ──

#[derive(Clone, Copy)]
enum Timezone {
    Honolulu,
    Anchorage,
    LosAngeles,
    Denver,
    Chicago,
    NewYork,
    Halifax,
    SaoPaulo,
    Azores,
    London,
    Paris,
    Athens,
    Moscow,
    Dubai,
    Karachi,
    Dhaka,
    Bangkok,
    Kolkata,
    Shanghai,
    Tokyo,
    Sydney,
    Noumea,
    Auckland,
}

impl Timezone {
    fn name(&self) -> &'static str {
        match self {
            Timezone::Honolulu => "Pacific/Honolulu",
            Timezone::Anchorage => "America/Anchorage",
            Timezone::LosAngeles => "America/Los_Angeles",
            Timezone::Denver => "America/Denver",
            Timezone::Chicago => "America/Chicago",
            Timezone::NewYork => "America/New_York",
            Timezone::Halifax => "America/Halifax",
            Timezone::SaoPaulo => "America/Sao_Paulo",
            Timezone::Azores => "Atlantic/Azores",
            Timezone::London => "Europe/London",
            Timezone::Paris => "Europe/Paris",
            Timezone::Athens => "Europe/Athens",
            Timezone::Moscow => "Europe/Moscow",
            Timezone::Dubai => "Asia/Dubai",
            Timezone::Karachi => "Asia/Karachi",
            Timezone::Dhaka => "Asia/Dhaka",
            Timezone::Bangkok => "Asia/Bangkok",
            Timezone::Kolkata => "Asia/Kolkata",
            Timezone::Shanghai => "Asia/Shanghai",
            Timezone::Tokyo => "Asia/Tokyo",
            Timezone::Sydney => "Australia/Sydney",
            Timezone::Noumea => "Pacific/Noumea",
            Timezone::Auckland => "Pacific/Auckland",
        }
    }

    /// Nearest timezone to a given UTC offset in hours (rough initial guess).
    fn nearest(offset_hours: i64) -> Option<Self> {
        Timezone::all().iter().copied().min_by_key(|t| {
            let o = t.guess_offset();
            (o - offset_hours).abs()
        })
    }

    /// Rough offset (used only for guessing). Real conversion uses JS.
    fn guess_offset(&self) -> i64 {
        match self {
            Timezone::Honolulu => -10,
            Timezone::Anchorage => -9,
            Timezone::LosAngeles => -8,
            Timezone::Denver => -7,
            Timezone::Chicago => -6,
            Timezone::NewYork => -5,
            Timezone::Halifax => -4,
            Timezone::SaoPaulo => -3,
            Timezone::Azores => -1,
            Timezone::London => 0,
            Timezone::Paris => 1,
            Timezone::Athens => 2,
            Timezone::Moscow => 3,
            Timezone::Dubai => 4,
            Timezone::Karachi => 5,
            Timezone::Dhaka => 6,
            Timezone::Bangkok => 7,
            Timezone::Kolkata => 5,
            Timezone::Shanghai => 8,
            Timezone::Tokyo => 9,
            Timezone::Sydney => 10,
            Timezone::Noumea => 11,
            Timezone::Auckland => 12,
        }
    }

    fn all() -> &'static [Self] {
        &[
            Timezone::Honolulu,
            Timezone::Anchorage,
            Timezone::LosAngeles,
            Timezone::Denver,
            Timezone::Chicago,
            Timezone::NewYork,
            Timezone::Halifax,
            Timezone::SaoPaulo,
            Timezone::Azores,
            Timezone::London,
            Timezone::Paris,
            Timezone::Athens,
            Timezone::Moscow,
            Timezone::Dubai,
            Timezone::Karachi,
            Timezone::Dhaka,
            Timezone::Bangkok,
            Timezone::Kolkata,
            Timezone::Shanghai,
            Timezone::Tokyo,
            Timezone::Sydney,
            Timezone::Noumea,
            Timezone::Auckland,
        ]
    }

    fn all_options() -> &'static [(&'static str, &'static str)] {
        use std::sync::LazyLock;
        static OPTS: LazyLock<Vec<(&str, &str)>> =
            LazyLock::new(|| Timezone::all().iter().map(|t| (t.name(), t.name())).collect());
        &OPTS
    }
}

// ── Format options ──

#[derive(Clone, Copy)]
enum Format {
    Iso8601,
    IsoSpace,
    DateOnly,
    Hour12,
    Slash,
    Us,
    Eu,
    Human,
    Chinese,
}

impl Format {
    fn all() -> &'static [Format] {
        &[
            Format::Iso8601,
            Format::IsoSpace,
            Format::DateOnly,
            Format::Hour12,
            Format::Slash,
            Format::Us,
            Format::Eu,
            Format::Human,
            Format::Chinese,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Format::Iso8601 => "ISO 8601",
            Format::IsoSpace => "ISO (space)",
            Format::DateOnly => "Date only",
            Format::Hour12 => "12h",
            Format::Slash => "Slash",
            Format::Us => "US",
            Format::Eu => "EU",
            Format::Human => "Human",
            Format::Chinese => "Chinese",
        }
    }
}

fn format_date(
    year: i64,
    month: u32,
    day: u32,
    hours: u32,
    minutes: u32,
    seconds: u32,
    utc: bool,
    f: Format,
) -> String {
    let suffix = if utc { "Z" } else { "" };
    match f {
        Format::Iso8601 => format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}",
            year, month, day, hours, minutes, seconds, suffix
        ),
        Format::IsoSpace => format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}{}",
            year, month, day, hours, minutes, seconds, suffix
        ),
        Format::DateOnly => format!("{:04}-{:02}-{:02}", year, month, day),
        Format::Hour12 => {
            let (h12, ampm) = if hours == 0 {
                (12, "AM")
            } else if hours < 12 {
                (hours, "AM")
            } else if hours == 12 {
                (12, "PM")
            } else {
                (hours - 12, "PM")
            };
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}",
                year, month, day, h12, minutes, seconds, ampm
            )
        }
        Format::Slash => format!(
            "{:04}/{:02}/{:02} {:02}:{:02}:{:02}{}",
            year, month, day, hours, minutes, seconds, suffix
        ),
        Format::Us => format!(
            "{:02}/{:02}/{:04} {:02}:{:02}:{:02}{}",
            month, day, year, hours, minutes, seconds, suffix
        ),
        Format::Eu => format!(
            "{:02}/{:02}/{:04} {:02}:{:02}:{:02}{}",
            day, month, year, hours, minutes, seconds, suffix
        ),
        Format::Human => format!(
            "{} {:02}, {:04} {:02}:{:02}:{:02}{}",
            MONTHS_ABBR[month as usize - 1],
            day,
            year,
            hours,
            minutes,
            seconds,
            suffix
        ),
        Format::Chinese => format!(
            "{:04}年{:02}月{:02}日 {:02}:{:02}:{:02}{}",
            year, month, day, hours, minutes, seconds, suffix
        ),
    }
}

// ── Conversion ──

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 3600;
const SECONDS_PER_DAY: i64 = 86400;

const MONTHS_LOWERCASE: &[&str] = &[
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const MONTHS_ABBR: &[&str] =
    &["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn date_parts(ts: i64) -> (i64, u32, u32, u32, u32, u32) {
    let rem_secs = ts.rem_euclid(SECONDS_PER_DAY);
    let rem = if rem_secs < 0 { rem_secs + SECONDS_PER_DAY } else { rem_secs };
    let hours = (rem / SECONDS_PER_HOUR) as u32;
    let minutes = ((rem % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE) as u32;
    let seconds = (rem % SECONDS_PER_MINUTE) as u32;

    let days = ts.div_euclid(SECONDS_PER_DAY);
    let d = days + 719468;
    let era = if d >= 0 { d / 146097 } else { (d - 146096) / 146097 };
    let doe = d - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { (mp + 3) as u32 } else { (mp - 9) as u32 };
    let year = if month <= 2 { y + 1 } else { y };

    (year, month, day, hours, minutes, seconds)
}

fn get_local_offset_minutes() -> i64 {
    -js_sys::Date::new_0().get_timezone_offset() as i64
}

/// Use JS Intl.DateTimeFormat to compute the real offset in seconds
/// at a specific timestamp for a given IANA timezone.
/// This handles DST and half-hour timezones correctly.
fn get_tz_offset_seconds(ts_secs: i64, tz_iana: &str) -> i64 {
    let code = format!(
        "var d=new Date({}*1000);var \
         o=Intl.DateTimeFormat('en-US',{{timeZone:'{}',year:'numeric',month:'2-digit',day:'\
         2-digit',hour:'2-digit',minute:'2-digit',second:'2-digit',hour12:false}});var \
         v={{}};o.formatToParts(d).forEach(function(p){{if(p.type!=='literal')v[p.type]=p.\
         value}});var tzDate=new \
         Date(v.year+'-'+v.month+'-'+v.day+'T'+v.hour+':'+v.minute+':'+v.second+'Z');Math.\
         round((tzDate.getTime()-d.getTime())/1000)",
        ts_secs, tz_iana
    );
    js_sys::eval(&code).ok().and_then(|v| v.as_f64()).map(|v| v as i64).unwrap_or(0)
}

fn parse_date_to_timestamp(s: &str) -> Option<i64> {
    let s = s.trim();

    // Try Human: "Jan 22, 1975 14:13:20" or "January 22, 1975"
    let human_ts = (|| {
        let (mon_str, rest) = s.split_once(' ')?;
        if mon_str.len() < 3 {
            return None;
        }

        let mon_lower = mon_str.to_lowercase();
        let mon_idx =
            MONTHS_LOWERCASE.iter().position(|m| m.to_lowercase().starts_with(&mon_lower))?;
        let month = mon_idx as i64 + 1;

        // Handle optional comma: "22, 1975" or "22 1975"
        let rest = rest.trim_start();
        let (day_str, rest2) = if let Some((d, r)) = rest.split_once(',') {
            (d, r.trim_start())
        } else {
            rest.split_once(' ')?
        };
        let day: i64 = day_str.trim().parse().ok()?;

        // split_whitespace is immune to consecutive spaces
        let mut parts = rest2.split_whitespace();
        let year: i64 = parts.next()?.parse().ok()?;

        if let Some(time_str) = parts.next() {
            date_to_ts(year, month, day, time_str)
        } else {
            date_to_ts(year, month, day, "00:00:00")
        }
    })();

    if let Some(ts) = human_ts {
        return Some(ts);
    }

    let (date_part, time_part) = if let Some(idx) = s.find('T') {
        (&s[..idx], &s[idx + 1..])
    } else if let Some(idx) = s.rfind(' ') {
        let maybe_time = &s[idx + 1..];
        if maybe_time.contains(':') { (&s[..idx], maybe_time) } else { (s, "00:00:00") }
    } else {
        (s, "00:00:00")
    };

    let time_part = time_part.trim_end_matches('Z').trim_end_matches('z');

    let dp: Vec<&str> = date_part.split(&['-', '/'][..]).collect();
    if dp.len() != 3 {
        return None;
    }
    let (v1, v2, v3): (i64, i64, i64) =
        (dp[0].parse().ok()?, dp[1].parse().ok()?, dp[2].parse().ok()?);

    // Heuristic: find the year (first value > 31), then assign month/day.
    let (year, month, day) = match (v1 > 31, v2 > 31, v3 > 31) {
        (true, _, _) => (v1, v2, v3), // Y-M-D (e.g. 2023-11-15)
        (_, true, _) => match (v1 > 12, v3 > 12) {
            // rare: year in middle (e.g. 15/2023/11)
            (false, true) => (v2, v1, v3), // M-Y-D
            (true, false) => (v2, v3, v1), // D-Y-M
            _ => (v2, v3, v1),             // ambiguous, default D-Y-M
        },
        (_, _, true) => match (v1 > 12, v2 > 12) {
            // common: year at end
            (false, true) => (v3, v1, v2), // M-D-Y (US, e.g. 11/15/2023)
            (true, false) => (v3, v2, v1), // D-M-Y (EU, e.g. 15/11/2023)
            _ => (v3, v1, v2),             // ambiguous (both ≤ 12), default US
        },
        _ => (v1, v2, v3), // all ≤ 31, assume Y-M-D
    };

    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }
    date_to_ts(year, month, day, time_part)
}

fn date_to_ts(year: i64, month: i64, day: i64, time_part: &str) -> Option<i64> {
    let tp: Vec<&str> = time_part.split(':').collect();
    let hour: i64 = tp.first().and_then(|v| v.parse().ok()).unwrap_or(0);
    let min: i64 = tp.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
    let sec: i64 =
        tp.get(2).and_then(|v| v.split('.').next()).and_then(|v| v.parse().ok()).unwrap_or(0);

    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };
    let days = (365 * y + y / 4 - y / 100 + y / 400 + (153 * (m - 3) + 2) / 5 + day - 1) - 719468;
    Some(days * SECONDS_PER_DAY + hour * SECONDS_PER_HOUR + min * SECONDS_PER_MINUTE + sec)
}

fn do_convert(input: &str, tz: Timezone) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Timestamp → formatted date list
    if let Ok(ts) = trimmed.parse::<i64>() {
        let s = if ts > 1_000_000_000_000 { ts / 1000 } else { ts };
        let (y, mo, d, h, mi, se) = date_parts(s);

        let tz_offset_secs = get_tz_offset_seconds(s, tz.name());
        let local_ts = s + tz_offset_secs;
        let (y2, mo2, d2, h2, mi2, se2) = date_parts(local_ts);

        let label_width = Timezone::all()
            .iter()
            .map(|t| t.name().len())
            .chain(Format::all().iter().map(|f| f.label().len()))
            .max()
            .unwrap_or(18);

        let mut lines = Vec::new();
        lines.push(format!(
            "{:<w$}  {}",
            "UTC",
            format_date(y, mo, d, h, mi, se, true, Format::Iso8601),
            w = label_width
        ));
        lines.push(format!(
            "{:<w$}  {}",
            tz.name(),
            format_date(y2, mo2, d2, h2, mi2, se2, false, Format::Iso8601),
            w = label_width
        ));
        lines.push(String::new());
        for f in Format::all().iter().skip(1) {
            lines.push(format!(
                "{:<w$}  {}",
                f.label(),
                format_date(y2, mo2, d2, h2, mi2, se2, false, *f),
                w = label_width
            ));
        }
        return lines.join("\n");
    }

    // Formatted date → timestamp
    if let Some(ts_as_utc) = parse_date_to_timestamp(trimmed) {
        // If input lacks trailing Z/z, treat as local time in the selected timezone.
        let is_explicit_utc = trimmed.to_lowercase().contains('z');
        let final_ts = if is_explicit_utc {
            ts_as_utc
        } else {
            let offset_secs = get_tz_offset_seconds(ts_as_utc, tz.name());
            ts_as_utc - offset_secs
        };
        return format!("{final_ts} (s)\n{final_ts}000 (ms)");
    }

    String::new()
}
