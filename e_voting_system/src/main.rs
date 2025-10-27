// ============================================================
// File: main.rs
// Purpose: Entry point of the e_voting_system program.
// 
// Responsibilities:
// - Initializes the system and database
// - Handles command-line or menu-based user interaction
// - Routes control to the correct module (admin, voter, district)
// - Coordinates authentication and session flow
// ============================================================



// src/main.rs
// ==============================================
// Entry point for the Secure Voting Machine 

mod admin;
mod models;
mod auth;

use clap::{Parser, Subcommand, Args};
use rusqlite::{params, Connection};
use chrono::{NaiveDate, Utc};
use std::path::PathBuf;
use crate::admin::AdminService;
use crate::security::{hash_password, verify_password};

// --------------------------- CLI STRUCTS ---------------------------

#[derive(Parser, Debug)]
#[command(name = "rusttrust", version, about = "Secure Voting Machine CLI")]
struct Cli {
    #[arg(global = true, short, long, default_value = "rusttrust.db")]
    db: PathBuf,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize database and create first admin
    Init {
        #[arg(short = 'u', long)]
        admin_user: String,

        #[arg(short = 'p', long)]
        admin_pass: String,
    },

    /// Admin-level actions
    Admin(AdminCmd),

    /// Simple test to list all elections
    List,
}

// --------------------------- Admin CLI -----------------------------

#[derive(Args, Debug)]
struct AdminCmd {
    #[command(subcommand)]
    sub: AdminSub,
}

#[derive(Subcommand, Debug)]
enum AdminSub {
    /// Create a new election
    CreateElection {
        name: String,
        #[arg(long)]
        positions: String,
    },

    /// Add a candidate
    AddCandidate {
        election_id: i64,
        position_idx: i32,
        name: String,
        party: String,
    },

    /// Register a voter
    RegisterVoter {
        fullname: String,
        dob: String,
        pin: String,
    },

    /// View election results
    ViewResults {
        election_id: i64,
    },

    /// Log in as an admin
    Login {
        #[arg(short = 'u', long)]
        username: String,

        #[arg(short = 'p', long)]
        password: String,
    },
}

// --------------------------- HELPER FUNCTIONS ----------------------

fn connect(db: &PathBuf) -> Connection {
    Connection::open(db).expect("Failed to open database")
}

fn migrate(conn: &Connection) {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS admins (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS elections (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'Draft',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS positions (
            id INTEGER PRIMARY KEY,
            election_id INTEGER NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
            idx INTEGER NOT NULL,
            title TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS candidates (
            id INTEGER PRIMARY KEY,
            election_id INTEGER NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
            position_idx INTEGER NOT NULL,
            name TEXT NOT NULL,
            party TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS voters (
            id INTEGER PRIMARY KEY,
            fullname TEXT NOT NULL,
            dob TEXT NOT NULL,
            pinhash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS votes (
            id INTEGER PRIMARY KEY,
            election_id INTEGER NOT NULL,
            voter_id INTEGER NOT NULL,
            position_idx INTEGER NOT NULL,
            candidate_id INTEGER NOT NULL,
            cast_at TEXT NOT NULL
        );
    "#,
    )
    .unwrap();
    println!("Database migration complete ✅");
}

// --------------------------- MAIN ----------------------------------

fn main() {
    let cli = Cli::parse();
    let conn = connect(&cli.db);
    migrate(&conn);

    match cli.cmd {
        Commands::Init { admin_user, admin_pass } => {
            println!("🛠️  Setting up initial admin: {admin_user}");

            // check if admin already exists
            let exists: bool = conn
                .query_row("SELECT EXISTS(SELECT 1 FROM admins LIMIT 1)", [], |row| row.get(0))
                .unwrap_or(false);

            if exists {
                println!("⚠️  An admin already exists. Only one admin can be initialized this way.");
                return;
            }

            let hash = hash_password(&admin_pass);
            conn.execute(
                "INSERT INTO admins (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
                params![admin_user, hash, Utc::now().to_rfc3339()],
            )
            .expect("Failed to insert admin");

            println!("✅ Admin '{admin_user}' created and stored securely!");
        }

        Commands::Admin(ac) => {
            let admin = AdminService::new(&conn);

            match ac.sub {
                AdminSub::CreateElection { name, positions } => {
                    let pos: Vec<&str> = positions.split(',').map(|p| p.trim()).collect();
                    let eid = admin.create_election(&name, &pos).unwrap();
                    println!("✅ Election '{name}' created with ID {eid}");
                }

                AdminSub::AddCandidate {
                    election_id,
                    position_idx,
                    name,
                    party,
                } => {
                    admin.add_candidate(election_id, position_idx, &name, &party).unwrap();
                    println!("✅ Candidate '{name}' added to election #{election_id}");
                }

                AdminSub::RegisterVoter { fullname, dob, pin } => {
                    let id = admin.register_voter(&fullname, &dob, &pin).unwrap();
                    println!("✅ Voter '{fullname}' registered with ID {id}");
                }

                AdminSub::ViewResults { election_id } => {
                    admin.view_results(election_id).unwrap();
                }

                AdminSub::Login { username, password } => {
                    let result: Result<(String, String), _> = conn.query_row(
                        "SELECT username, password_hash FROM admins WHERE username=?1",
                        params![username],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    );

                    match result {
                        Ok((stored_user, stored_hash)) => {
                            if verify_password(&stored_hash, &password) {
                                println!("✅ Admin '{}' successfully logged in!", stored_user);
                            } else {
                                println!("❌ Incorrect password.");
                            }
                        }
                        Err(_) => println!("⚠️ No such admin found."),
                    }
                }
            }
        }

        Commands::List => {
            let admin = AdminService::new(&conn);
            let elections = admin.list_elections().unwrap();
            println!("📋 Elections:");
            for e in elections {
                println!(" - ID {}: {} [{}]", e.id, e.name, e.status);
            }
        }
    }
}
