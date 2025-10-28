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
/// Sets up a small SQLite database for testing.
fn setup_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS elections (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL
        )",
        [],
    )?;
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

