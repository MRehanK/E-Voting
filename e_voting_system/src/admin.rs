// ============================================================
// File: admin.rs
// Purpose: Provides functionality and interface for the system administrator.
//
// Responsibilities:
// - Create and manage elections
// - Add, update, or remove candidates and voters
// - View and audit election results
// - Coordinate with district officials
// ============================================================

// src/admin.rs
// ====================================================
// Purpose: Provides functionality and interface for the system administrator.
// Responsibilities:
// - Create and manage elections
// - Add, update, or remove candidates and voters
// - View and audit election results
// - Coordinate with district officials

use rusqlite::{params, Connection};
use chrono::Utc;
use crate::models::{Candidate, Election, Voter};
use crate::security::{hash_password};

pub struct AdminService<'a> {
    conn: &'a Connection,
}

impl<'a> AdminService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // ------------------ Election Management ------------------
    pub fn create_election(&self, name: &str, positions: &[&str]) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO elections(name, status, created_at) VALUES(?,?,?)",
            params![name, "Draft", Utc::now().to_rfc3339()])?;
        let eid = self.conn.last_insert_rowid();

        for (idx, title) in positions.iter().enumerate() {
            self.conn.execute(
                "INSERT INTO positions(election_id, idx, title) VALUES(?,?,?)",
                params![eid, idx as i32, title])?;
        }
        Ok(eid)
    }

    pub fn update_election_name(&self, election_id: i64, new_name: &str) -> rusqlite::Result<()> {
        self.conn.execute("UPDATE elections SET name=?1 WHERE id=?2", params![new_name, election_id])?;
        Ok(())
    }

    pub fn delete_election(&self, election_id: i64) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM elections WHERE id=?1", params![election_id])?;
        Ok(())
    }

    // ------------------ Candidate Management ------------------
    pub fn add_candidate(&self, election_id: i64, position_idx: i32, name: &str, party: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO candidates(election_id, position_idx, name, party) VALUES(?,?,?,?)",
            params![election_id, position_idx, name, party])?;
        Ok(())
    }

    pub fn update_candidate(&self, candidate_id: i64, new_name: &str, new_party: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE candidates SET name=?1, party=?2 WHERE id=?3",
            params![new_name, new_party, candidate_id])?;
        Ok(())
    }

    pub fn remove_candidate(&self, candidate_id: i64) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM candidates WHERE id=?1", params![candidate_id])?;
        Ok(())
    }

    // ------------------ Voter Management ------------------
    pub fn register_voter(&self, fullname: &str, dob: &str, pin: &str) -> rusqlite::Result<i64> {
        let pinhash = hash_password(pin);
        self.conn.execute(
            "INSERT INTO voters(fullname, dob, pinhash) VALUES(?,?,?)",
            params![fullname, dob, pinhash])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn remove_voter(&self, voter_id: i64) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM voters WHERE id=?1", params![voter_id])?;
        Ok(())
    }

    // ------------------ Audit & Reporting ------------------
    pub fn list_elections(&self) -> rusqlite::Result<Vec<Election>> {
        let mut stmt = self.conn.prepare("SELECT id, name, status, created_at FROM elections")?;
        let rows = stmt.query_map([], |row| {
            Ok(Election {
                id: row.get(0)?,
                name: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn view_results(&self, election_id: i64) -> rusqlite::Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT position_idx, candidate_id, COUNT(*) FROM votes WHERE election_id=?1 GROUP BY position_idx, candidate_id ORDER BY position_idx, COUNT(*) DESC"
        )?;
        let mut rows = stmt.query(params![election_id])?;
        println!("Election results for #{election_id}:");
        while let Some(r) = rows.next()? {
            let pidx: i32 = r.get(0)?;
            let cid: i64 = r.get(1)?;
            let count: i64 = r.get(2)?;
            let (name, party): (String, String) = self.conn.query_row(
                "SELECT name, party FROM candidates WHERE id=?1",
                params![cid], |row| Ok((row.get(0)?, row.get(1)?)))?;
            println!("  Position {pidx}: {name} ({party}) -> {count} votes");
        }
        Ok(())
    }

    // ------------------ Coordination ------------------
    pub fn coordinate_with_district(&self) {
        println!("Admins can view district statuses and coordinate vote phases.");
        // Placeholder for messaging or API integration logic later
    }
}
