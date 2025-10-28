// src/main.rs - Entry point for the Secure Voting Machine 

mod admin;
mod models;
mod auth;
mod voter;
mod vote;

use clap::{Parser, Subcommand, Args};
use rusqlite::{params, Connection};
use chrono::Utc;
use std::path::PathBuf;
use std::io::{self, Write};
use crate::admin::AdminService;
use crate::auth::{hash_password, verify_password};
use crate::voter::{voter_login, voter_portal};

// --------------------------- CLI STRUCTS ---------------------------

#[derive(Parser, Debug)]
#[command(name = "rusttrust", version, about = "Secure Voting Machine CLI")]
struct Cli {
    #[arg(global = true, short, long, default_value = "rusttrust.db")]
    db: PathBuf,

    #[command(subcommand)]
    cmd: Option<Commands>,
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

    /// Launch interactive menu
    Menu,
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

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn show_main_menu() {
    println!("\n🗳️  === E-Voting System Main Menu ===");
    println!("1. Initialize System (Create Admin)");
    println!("2. Admin Operations");
    println!("3. Voter Login");
    println!("4. List Elections");
    println!("5. Exit");
    println!("=====================================");
}

fn show_admin_menu() {
    println!("\n👨‍💼 === Admin Operations Menu ===");
    println!("1. Create Election");
    println!("2. Add Candidate");
    println!("3. Register Voter");
    println!("4. View Election Results");
    println!("5. Login as Admin");
    println!("6. Back to Main Menu");
    println!("=================================");
}

fn interactive_admin_operations(conn: &Connection) {
    let admin = AdminService::new(conn);
    
    loop {
        show_admin_menu();
        let choice = read_input("Select an option (1-6): ");
        
        match choice.as_str() {
            "1" => {
                let name = read_input("Enter election name: ");
                let positions_input = read_input("Enter positions (comma-separated): ");
                let positions: Vec<&str> = positions_input.split(',').map(|p| p.trim()).collect();
                
                match admin.create_election(&name, &positions) {
                    Ok(eid) => println!("✅ Election '{}' created with ID {}", name, eid),
                    Err(e) => println!("❌ Error creating election: {}", e),
                }
            }
            "2" => {
                let election_id = read_input("Enter election ID: ");
                let position_idx = read_input("Enter position index: ");
                let name = read_input("Enter candidate name: ");
                let party = read_input("Enter party: ");
                
                match (election_id.parse::<i64>(), position_idx.parse::<i32>()) {
                    (Ok(eid), Ok(pidx)) => {
                        match admin.add_candidate(eid, pidx, &name, &party) {
                            Ok(_) => println!("✅ Candidate '{}' added to election #{}", name, eid),
                            Err(e) => println!("❌ Error adding candidate: {}", e),
                        }
                    }
                    _ => println!("❌ Invalid election ID or position index"),
                }
            }
            "3" => {
                let fullname = read_input("Enter voter full name: ");
                let dob = read_input("Enter date of birth: ");
                let pin = read_input("Enter PIN: ");
                
                match admin.register_voter(&fullname, &dob, &pin) {
                    Ok(id) => println!("✅ Voter '{}' registered with ID {}", fullname, id),
                    Err(e) => println!("❌ Error registering voter: {}", e),
                }
            }
            "4" => {
                let election_id = read_input("Enter election ID: ");
                match election_id.parse::<i64>() {
                    Ok(eid) => {
                        match admin.view_results(eid) {
                            Ok(_) => {},
                            Err(e) => println!("❌ Error viewing results: {}", e),
                        }
                    }
                    _ => println!("❌ Invalid election ID"),
                }
            }
            "5" => {
                let username = read_input("Enter admin username: ");
                let password = read_input("Enter admin password: ");
                
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
            "6" => break,
            "" => {
                println!("⚠️  Please enter a valid option (1-6).");
                continue;
            }
            _ => println!("❌ Invalid option. Please select 1-6."),
        }
        
        read_input("\nPress Enter to continue...");
    }
}

fn interactive_menu(conn: &Connection) {
    loop {
        show_main_menu();
        let choice = read_input("Select an option (1-5): ");
        
        match choice.as_str() {
            "1" => {
                println!("\n🛠️  Setting up initial admin...");
                let admin_user = read_input("Enter admin username: ");
                let admin_pass = read_input("Enter admin password: ");
                
                // Check if admin already exists
                let exists: bool = conn
                    .query_row("SELECT EXISTS(SELECT 1 FROM admins LIMIT 1)", [], |row| row.get(0))
                    .unwrap_or(false);

                if exists {
                    println!("⚠️  An admin already exists. Only one admin can be initialized this way.");
                } else {
                    let hash = hash_password(&admin_pass);
                    match conn.execute(
                        "INSERT INTO admins (username, password_hash, created_at) VALUES (?1, ?2, ?3)",
                        params![admin_user, hash, Utc::now().to_rfc3339()],
                    ) {
                        Ok(_) => println!("✅ Admin '{}' created and stored securely!", admin_user),
                        Err(e) => println!("❌ Error creating admin: {}", e),
                    }
                }
            }
            "2" => {
                interactive_admin_operations(conn);
            }
            "3" => {
                if let Some((voter_id, voter_name)) = voter_login(conn) {
                    voter_portal(conn, voter_id, &voter_name);
                }
            }
            "4" => {
                print_elections(conn);
            }
            "5" => {
                println!("👋 Goodbye!");
                break;
            }
            "" => {
                println!("⚠️  Please enter a valid option (1-5).");
                continue;
            }
            _ => println!("❌ Invalid option. Please select 1-5."),
        }
        
        if choice != "5" {
            read_input("\nPress Enter to continue...");
        }
    }
}

fn print_elections(conn: &Connection) {
    let admin = AdminService::new(conn);
    match admin.list_elections() {
        Ok(elections) => {
            println!("\n📋 Elections:");
            if elections.is_empty() {
                println!("No elections found.");
            } else {
                for e in elections {
                    println!(" - ID {}: {} [{}]", e.id, e.name, e.status);
                }
            }
        }
        Err(e) => println!("❌ Error listing elections: {}", e),
    }
}

// voter-related functions moved to voter.rs

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
        Some(Commands::Init { admin_user, admin_pass }) => {
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

        Some(Commands::Admin(ac)) => {
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

        Some(Commands::List) => {
            print_elections(&conn);
        }

        Some(Commands::Menu) => {
            interactive_menu(&conn);
        }
        None => {
            // No subcommand provided: launch interactive menu by default
            interactive_menu(&conn);
        }
    }
}
