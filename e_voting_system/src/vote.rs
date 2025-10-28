// ============================================================
// File: vote.rs
// Purpose: Vote-specific operations (queries and inserts)
// ============================================================

use chrono::Utc;
use rusqlite::{params, Connection};

pub fn has_voted(conn: &Connection, voter_id: i64, election_id: i64) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM votes WHERE election_id=?1 AND voter_id=?2)",
        params![election_id, voter_id],
        |row| row.get(0),
    )
    .unwrap_or(false)
}

pub fn list_elections(conn: &Connection) -> rusqlite::Result<Vec<(i64, String)>> {
    let mut stmt = conn.prepare("SELECT id, name FROM elections ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_candidates(
    conn: &Connection,
    election_id: i64,
) -> rusqlite::Result<Vec<(i64, String, String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, party, position_idx FROM candidates WHERE election_id=?1 ORDER BY id ASC",
    )?;
    let rows = stmt.query_map(params![election_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn record_vote(
    conn: &Connection,
    election_id: i64,
    voter_id: i64,
    position_idx: i32,
    candidate_id: i64,
) -> rusqlite::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO votes (election_id, voter_id, position_idx, candidate_id, cast_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![election_id, voter_id, position_idx, candidate_id, now],
    )?;
    Ok(())
}


