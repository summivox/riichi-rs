use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GameLogInfo {
    pub id: String,
    pub start_date_time: Option<NaiveDateTime>,
    pub duration_mins: Option<u8>,
    pub rule_id: Option<u32>,
    pub lobby_id: Option<String>,
}

/// Extracts log ID and relevant metadata from a line in the "HTML" archive.
///
/// Example:
/// ```
/// use chrono::{NaiveDate, NaiveTime};
/// use tenhou_db::parse::{GameLogInfo, parse_archive_line};
/// assert_eq!(
///     parse_archive_line(
///         r#"02:49 | 10 | 四鳳東喰赤－ | <a href="http://tenhou.net/0/?log=2021012802gm-00e1-0000-4735afd5">牌譜</a> | 長島ンガン(+43.0) しーたくん(+11.0) じゃんみー(-17.0) 闘将だんでぃ(-37.0)<br>"#
///     ),
///     Some(GameLogInfo {
///         id: "2021012802gm-00e1-0000-4735afd5".to_string(),
///         start_date_time: Some(NaiveDate::from_ymd(2021, 01, 28).and_hms(02, 49, 00)),
///         duration_mins: Some(10),
///         rule_id: Some(0x00e1),
///         lobby_id: Some("0000".to_string()),
///     }));
/// ```
pub fn parse_archive_line(line: &str) -> Option<GameLogInfo> {
    log::trace!("parse archive line: {}", line);
    static ARCHIVE_LINE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?x)
        (?:\s*(\d\d):(\d\d)\s*\|)?
        (?:\s*(\d+)\s*\|)?.*?
        log=([0-9A-Za-z\-]+)
    "#).unwrap());
    ARCHIVE_LINE_REGEX.captures(line).map(|g| {
        let id = g[4].to_string();
        let parsed_id = parse_log_id(&id);
        let start_hour = g.get(1).and_then(|m| m.as_str().parse().ok());
        let start_minute = g.get(2).and_then(|m| m.as_str().parse().ok());
        let duration_mins = g.get(3).and_then(|m| m.as_str().parse().ok());
        let start_time = start_hour.zip(start_minute)
            .and_then(|(h, m)| NaiveTime::from_hms_opt(h, m, 0));

        let start_date_time = parse_log_id(&id).as_ref().and_then(|parts| {
            if let Some(start_time) = start_time {
                if parts.start_date_hour.hour() == start_time.hour() {
                    // consistent; augment with minutes
                    parts.start_date_hour.with_minute(start_time.minute())
                } else {
                    // inconsistent
                    None
                }
            } else {
                // no minutes available
                Some(parts.start_date_hour)
            }
        });
        let rule_id = parsed_id.as_ref().map(|parts| parts.rule_id);
        let lobby_id = parsed_id.as_ref().map(|parts| parts.lobby_id.to_string());

        GameLogInfo {
            id,
            start_date_time,
            duration_mins,
            rule_id,
            lobby_id,
        }
    })
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct LogIdParts<'a> {
    pub start_date_hour: NaiveDateTime,
    pub rule_id: u32,
    pub lobby_id: &'a str,
    pub hash: &'a str,
}

/// Extracts fields from a log id.
///
/// Example:
/// ```
/// use chrono::{NaiveDate, NaiveDateTime};
/// use tenhou_db::parse::{LogIdParts, parse_log_id};
/// assert_eq!(parse_log_id("2021012802gm-00e1-0000-4735afd5"), Some(LogIdParts{
///     start_date_hour: NaiveDate::from_ymd(2021, 01, 28).and_hms(02, 00, 00),
///     rule_id: 0x00e1,
///     lobby_id: "0000",
///     hash: "4735afd5",
/// }));
/// ```
pub fn parse_log_id(id: &str) -> Option<LogIdParts<'_>> {
    static LOG_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(
        r#"(\d\d\d\d)(\d\d)(\d\d)(\d\d)\w*-(\w+)-(\w+)-(\w+)"#
    ).unwrap());
    LOG_ID_REGEX.captures(id).and_then(|g| {
        let year = g.get(1).unwrap().as_str().parse().ok()?;
        let month = g.get(2).unwrap().as_str().parse().ok()?;
        let day = g.get(3).unwrap().as_str().parse().ok()?;
        let hour = g.get(4).unwrap().as_str().parse().ok()?;
        let rule_id_str = g.get(5).unwrap().as_str();
        let rule_id = u32::from_str_radix(rule_id_str, 16).ok()?;
        let lobby_id = g.get(6).unwrap().as_str();
        let hash = g.get(7).unwrap().as_str();

        let start_date_hour = NaiveDate::from_ymd_opt(year, month, day).and_then(|date|
            date.and_hms_opt(hour, 0, 0))?;

        Some(LogIdParts {
            start_date_hour,
            rule_id,
            lobby_id,
            hash,
        })
    })
}

pub struct RuleIdParts {
    /// false => "四", true => "三"
    pub three: bool,
    /// 0b00 => "般", 0b01 => "上", 0b10 => "特", 0b11 => "鳳"
    pub level: u8,
    /// false => "東", true => "南"
    pub south: bool,
    /// true => "喰"
    pub kuitan: bool,
    /// true => "赤"
    pub red: bool,
    /// true => "速"
    pub fast: bool,
}

/// ## Known rule id interpretations
///
/// | hex    | bin                    | rule (JP) |
/// |--------|------------------------|-----------|
/// | 0x0001 | 0b_0000_0000_0000_0001 | 般東喰赤 |
/// | 0x0007 | 0b_0000_0000_0000_0111 | 般東 |
/// | 0x0009 | 0b_0000_0000_0000_1001 | 般南喰赤 |
/// | 0x000f | 0b_0000_0000_0000_1111 | 般南 |
/// | 0x0029 | 0b_0000_0000_0010_1001 | 特南喰赤 |
/// | 0x0041 | 0b_0000_0000_0100_0001 | 般東喰赤速 |
/// | 0x0089 | 0b_0000_0000_1000_1001 | 上南喰赤 |
/// | 0x00a9 | 0b_0000_0000_1010_1001 | 鳳南喰赤 |
/// | 0x00b1 | 0b_0000_0000_1011_0001 | 三鳳東喰赤 |
/// | 0x00b9 | 0b_0000_0000_1011_1001 | 三鳳南喰赤 |
/// | 0x00c1 | 0b_0000_0000_1100_0001 | 上東喰赤速 |
/// | 0x00e1 | 0b_0000_0000_1110_0001 | 鳳東喰赤速 |
/// | 0x00f1 | 0b_0000_0000_1111_0001 | 三鳳東喰赤速 |
/// | 0x0639 | 0b_0000_0110_0011_1001 | 三琥南喰赤祝５ |
///
pub fn parse_rule_id(rule_id: u32) -> RuleIdParts {
    RuleIdParts {
        three: (rule_id & (1 << 4)) > 0,
        level: (((rule_id >> 5) & 1) | (((rule_id >> 7) & 1) << 1)) as u8,
        south: (rule_id & (1 << 3)) > 0,
        kuitan: (rule_id & (1 << 2)) > 0,  // NOTE: this is guessed (always the same as `red`).
        red: (rule_id & (1 << 1)) > 0,  // NOTE: this is guessed (always the same as `kuitan`).
        fast: (rule_id & (1 << 6)) > 0,
    }
}

/// Extracts the relative URL and the expected size of a (compressed) Tenhou archive from a line in
/// the response of Tenhou's archive list API.
///
/// Example response from said API:
/// ```javascript
/// list([
/// {file:'2022/sca20220101.log.gz',size:44020},
/// {file:'2022/sca20220102.log.gz',size:39374},
/// {file:'2022/sca20220103.log.gz',size:41920},
/// {file:'2022/sca20220104.log.gz',size:26087},
/// /* ... */
/// {file:'2022/scf20220805.html.gz',size:4306}
/// ]);
/// ```
///
/// Parsing results:
/// ```
/// use tenhou_db::parse::parse_html_gz_url_list_line;
/// assert_eq!(parse_html_gz_url_list_line(r#"{file:'2022/sca20220104.html.gz',size:26087},"#),
///            Some(("2022/sca20220104.html.gz", Some(26087))));
/// assert_eq!(parse_html_gz_url_list_line(r#"{file:'2022/sca20220104.html.gz'},"#),
///            Some(("2022/sca20220104.html.gz", None)));  // not really expected, but just in case
/// assert_eq!(parse_html_gz_url_list_line(r#"{file:'2022/sca20220104.log.gz'},size:26087"#),
///            None);  // ignored
/// ```
///
pub fn parse_html_gz_url_list_line(line: &str) -> Option<(&str, Option<usize>)> {
    static LOG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(
        r"file:'([^']+html\.gz)'(?:,size:(\d+))?"
    ).unwrap());
    LOG_REGEX.captures(line)
        .and_then(|captures|
            captures.get(1).map(|file| (file, captures.get(2))))
        .map(|(file, maybe_size)|
            (file.as_str(), maybe_size.and_then(|size| size.as_str().parse().ok())))
}

/// Parses the day (and maybe hour) from the archive.
/// ```
/// use chrono::{NaiveDate, NaiveDateTime};
/// use tenhou_db::parse::{parse_gz_date_hour};
/// assert_eq!(parse_gz_date_hour("2022/sca20220104.log.gz"),
///            Some(NaiveDate::from_ymd(2022, 1, 4).and_hms(0, 0, 0)));
/// assert_eq!(parse_gz_date_hour("scc2022091223.html.gz"),
///            Some(NaiveDate::from_ymd(2022, 9, 12).and_hms(23, 0, 0)));
/// ```
pub fn parse_gz_date_hour(name: &str) -> Option<NaiveDateTime> {
    static GZ_DATE_HOUR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(
        r"(\d\d\d\d)(\d\d)(\d\d)(\d\d)?"
    ).unwrap());
    let fields = GZ_DATE_HOUR_REGEX.captures(name)?;
    let (year, month, day, hour) = (
        fields.get(1)?.as_str().parse().ok()?,
        fields.get(2)?.as_str().parse().ok()?,
        fields.get(3)?.as_str().parse().ok()?,
        fields.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
    );
    NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_opt(hour, 0, 0))
}
