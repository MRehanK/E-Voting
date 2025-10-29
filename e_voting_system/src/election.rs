// ============================================================
// File: election.rs
// Purpose: Implements the core election logic and voting process.
//
// Responsibilities:
// - Start and end elections
// - Manage candidate and voter interactions
// - Record and count votes
// - Compute and display election results
// ============================================================

// E-Voting System: District Officials Module

use rusqlite::{params, Connection, Result};// this part allows to import directly from rusqlite crate which is used by SQlite database
use std::collections::HashMap;

/// Represents a basic election record.
#[derive(Debug)]
// First we setup a small SQLite darabase
fn setup_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS elections (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS votes (
            id INTEGER PRIMARY KEY,
            election_id INTEGER,
            position_id INTEGER,
            candidate_id INTEGER
        )",
        [],
    )?;

    // Add a demo election if not already present
    conn.execute(
        "INSERT OR IGNORE INTO elections (id, title, status)
         VALUES (1, 'General Election 2025', 'Draft')",
        [],
    )?;

    // Add some sample votes
    conn.execute("DELETE FROM votes", [])?;// foe better security we delete old vote if it exists
    let sample_votes = vec![               // Basically there are three toples election_id, position_id and candidate_id 
        (1, 101, 201),                     // Two votes for candidate 201 in position 101 of election 1  
        (1, 101, 201),
        (1, 101, 202),
        (1, 102, 203),
        (1, 102, 203),
        (1, 102, 204),
    ];

   for (election_id, position_id, candidate_id) in sample_votes {
        conn.execute(
            "INSERT INTO votes (election_id, position_id, candidate_id)
             VALUES (?1, ?2, ?3)",        // Uses ?1, ?2, ?3 placeholders and params![] for safe insertion
            params![election_id, position_id, candidate_id],
        )?;
    }

    Ok(())
}  
struct Election {
    id: i32,
    title: String,
    status: String,
}


/// Opens an election so voters can start casting ballots.
fn open_election(conn: &Connection, election_id: i32) -> Result<()> {
    conn.execute(
        "UPDATE elections SET status = 'Open' WHERE id = ?1 AND status != 'Open'",
        params![election_id],
    )?;
    println!("Election {} is now OPEN.", election_id);
    Ok(())
}
/// Here it closes an ongoing election to stop further voting.
fn close_election(conn: &Connection, election_id: i32) -> Result<()> {
    conn.execute(
        "UPDATE elections SET status = 'Closed' WHERE id = ?1 AND status = 'Open'",
        params![election_id],
    )?;
    println!("Election {} has been CLOSED.", election_id);
    Ok(())
}




