// ============================================================
// File: voter.rs
// Purpose: Provides the voter-facing interface and functionality.
// ============================================================

use rusqlite::{params, Connection};

use crate::auth::verify_password;
use crate::vote::{has_voted, list_candidates, list_elections, record_vote};

use crate::read_input; // from main.rs

pub fn voter_login(conn: &Connection) -> Option<(i64, String)> {
    println!("\n🔐 Voter Login");
    let fullname = read_input("Full name: ");
    let pin = read_input("PIN: ");

    let result: rusqlite::Result<(i64, String)> = conn.query_row(
        "SELECT id, pinhash FROM voters WHERE fullname=?1",
        params![fullname],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    match result {
        Ok((voter_id, pinhash)) => {
            if verify_password(&pinhash, &pin) {
                println!("✅ Welcome, {}!", fullname);
                Some((voter_id, fullname))
            } else {
                println!("❌ Incorrect PIN.");
                None
            }
        }
        Err(_) => {
            println!("⚠️  No voter found with that name.");
            None
        }
    }
}

pub fn voter_portal(conn: &Connection, voter_id: i64, voter_name: &str) {
    loop {
        println!("\n👤 Voter Portal - {}", voter_name);
        println!("1. Vote");
        println!("2. Back to Main Menu");
        let choice = read_input("Select an option (1-2): ");

        match choice.as_str() {
            "1" => {
                voter_vote_flow(conn, voter_id);
            }
            "2" => break,
            "" => {
                println!("⚠️  Please enter a valid option (1-2).");
                continue;
            }
            _ => println!("❌ Invalid option. Please select 1-2."),
        }
        read_input("\nPress Enter to continue...");
    }
}

pub fn voter_vote_flow(conn: &Connection, voter_id: i64) {
    loop {
        println!("\n🗳️  Voting Menu");
        println!("1. Select an election");
        println!("2. Exit the system");
        let choice = read_input("Select an option (1-2): ");

        match choice.as_str() {
            "1" => {
                // List elections
                let elections = match list_elections(conn) { Ok(v) => v, Err(e) => { println!("Error: {}", e); return; } };
                if elections.is_empty() { println!("No elections available."); return; }
                println!("\nAvailable Elections:");
                for (id, name) in &elections { println!(" - {}: {}", id, name); }
                let election_input = read_input("Enter election ID: ");
                let election_id = match election_input.parse::<i64>() { Ok(v) => v, Err(_) => { println!("❌ Invalid election ID"); continue; } };

                // Find election name
                let election_name = elections.iter().find(|(id, _)| *id == election_id).map(|(_, n)| n.clone());
                if election_name.is_none() { println!("❌ Election not found."); continue; }
                let election_name = election_name.unwrap();

                // Double-vote check
                if has_voted(conn, voter_id, election_id) {
                    println!("This person has already voted for the '{}', a voter can only vote once", election_name);
                    return;
                }

                // List candidates for the election
                let candidates = match list_candidates(conn, election_id) { Ok(v) => v, Err(e) => { println!("Error: {}", e); return; } };
                if candidates.is_empty() { println!("No candidates for this election."); return; }
                println!("\nCandidates:");
                for (cid, cname, party, _) in &candidates { println!(" - {}: {} ({})", cid, cname, party); }
                let cand_input = read_input("Enter candidate ID: ");
                let candidate_id = match cand_input.parse::<i64>() { Ok(v) => v, Err(_) => { println!("❌ Invalid candidate ID"); continue; } };

                let selected = candidates.iter().find(|(cid, _, _, _)| *cid == candidate_id).cloned();
                if selected.is_none() { println!("❌ Candidate not found."); continue; }
                let (candidate_id, candidate_name, _party, position_idx) = selected.unwrap();

                // Confirm
                println!("You are selecting '{}' for the '{}', are you sure?", candidate_name, election_name);
                let confirm = read_input("Type 'Yes' to confirm, or 'No' to cancel: ");
                if confirm.eq_ignore_ascii_case("Yes") {
                    if let Err(e) = record_vote(conn, election_id, voter_id, position_idx, candidate_id) {
                        println!("❌ Error recording vote: {}", e);
                        return;
                    }
                    println!("Thank you for your Vote!");
                    std::process::exit(0);
                } else {
                    // Go back to previous menu
                    continue;
                }
            }
            "2" => {
                println!("👋 Goodbye!");
                std::process::exit(0);
            }
            "" => { println!("⚠️  Please enter a valid option (1-2)."); continue; }
            _ => println!("❌ Invalid option. Please select 1-2."),
        }
    }
}

