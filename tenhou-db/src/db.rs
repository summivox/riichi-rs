use std::path::Path;
use anyhow::{Context, ensure};

use chrono::NaiveDateTime;
use rusqlite::{Connection, Result};
use sha2::{Sha256, Digest};
use url::Url;

use crate::parse::{GameLogInfo};

pub fn init_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(include_str!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/db_def.sql")
    )).map(|_| conn)
}

pub fn now_local() -> NaiveDateTime {
    chrono::Utc::now().with_timezone(&chrono_tz::Asia::Tokyo).naive_local()
}

pub fn get_hash_string(json: &str) -> String {
    hex::encode(Sha256::new().chain_update(json.as_bytes()).finalize())
}

pub fn add_archive(
    db: &Connection,
    filename: &str,
    url: Option<&Url>,
    date_time_bin: Option<NaiveDateTime>,
) -> Result<usize> {
    let mut query = db.prepare_cached(r#"
        INSERT INTO archives (filename, url, date_time_bin) VALUES (?1, ?2, ?3)
            ON CONFLICT DO UPDATE SET (url, date_time_bin) = (?2, ?3);
    "#)?;
    query.execute((
        filename,
        url.map(Url::as_str),
        date_time_bin,
    ))
}

pub fn is_archive_in_db(
    db: &Connection,
    filename: &str,
) -> anyhow::Result<bool> {
    let mut query = db.prepare_cached(r#"
        SELECT COUNT(*) FROM archives WHERE filename = ?;
    "#)?;
    let mut rows = query.query_map((filename,), |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(rows.next().context("bad query")?? > 0)
}

pub fn add_game_id_only(db: &Connection, id: &str) -> Result<usize> {
    log::debug!("add game id: {}", id);
    let mut query = db.prepare_cached(r#"
        INSERT INTO games (log_id) VALUES (?1)
            ON CONFLICT DO NOTHING;
    "#)?;
    query.execute((id,))
}

pub fn add_game_meta_only(db: &Connection, entry: &GameLogInfo) -> Result<usize> {
    log::debug!("add game meta id: {}", entry.id);
    let mut query = db.prepare_cached(r#"
        INSERT INTO games (log_id, start_date_time, duration_minutes, rule_id, lobby_id)
        VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT DO NOTHING;
    "#)?;
    query.execute((
        &entry.id,
        entry.start_date_time,
        entry.duration_mins,
        entry.rule_id,
        &entry.lobby_id,
    ))
}

pub fn add_game_full(db: &Connection, entry: &GameLogInfo, json_str: &str) -> Result<usize> {
    log::debug!("add game full id: {}", entry.id);
    let mut query = db.prepare_cached(r#"
        INSERT INTO games (log_id, start_date_time, duration_minutes, rule_id, lobby_id, game_json, game_json_sha256, last_checked)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT DO UPDATE SET (start_date_time, duration_minutes, rule_id, lobby_id, game_json, game_json_sha256, last_checked) =
            (?2, ?3, ?4, ?5, ?6, ?7, ?8);
    "#)?;
    let sha256 = get_hash_string(json_str);
    query.execute((
        &entry.id,
        entry.start_date_time,
        entry.duration_mins,
        entry.rule_id,
        &entry.lobby_id,
        json_str,
        &sha256,
        now_local(),
    ))
}

pub fn add_game_json_for_id(db: &Connection, id: &str, json_str: &str) -> Result<usize> {
    log::debug!("add game json for id: {}", id);
    let mut query = db.prepare_cached(r#"
        UPDATE games SET (game_json, game_json_sha256, last_checked) = (?2, ?3, ?4)
        WHERE log_id = ?1
    "#)?;
    let sha256 = get_hash_string(json_str);
    query.execute((id, json_str, &sha256, now_local()))
}

pub fn get_db_game_by_id(db: &Connection, id: &str) -> anyhow::Result<(GameLogInfo, Option<String>)> {
    log::debug!("get game id: {}", id);
    let mut query = db.prepare_cached(r#"
        SELECT log_id, start_date_time, duration_minutes, rule_id, lobby_id, game_json, game_json_sha256 FROM games
            WHERE log_id = ?1
            LIMIT 1;
    "#)?;
    let (entry, json, sha256_expected) =
        query.query_map((id, ), |row| {
            let start_date_time_str: Option<String> = row.get(1)?;
            let start_date_time = start_date_time_str.and_then(|s|
                NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok());
            let json: Option<String> = row.get(5)?;
            let sha256_expected: Option<String> = row.get(6)?;
            log::debug!("got row; sha256={:?}", sha256_expected);
            Ok((
                GameLogInfo {
                    id: id.to_string(),
                    start_date_time,
                    duration_mins: row.get(2)?,
                    rule_id: row.get(3)?,
                    lobby_id: row.get(4)?,
                },
                json,
                sha256_expected,
            ))
        })?.next()
            .with_context(|| format!("No game id '{}' in DB", id))?
            .context("row error")?;
    if let Some(json) = &json {
        if let Some(sha256_expected) = sha256_expected {
            let sha256_actual = get_hash_string(json);
            ensure!(sha256_expected == sha256_actual,
                "Hash mismatch: expected='{:?}', actual='{}'", sha256_expected, sha256_actual);
        }
    }
    Ok((entry, json))
}
