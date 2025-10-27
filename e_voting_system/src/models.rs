// ============================================================
// File: models.rs
// Purpose: Defines the core data structures (models) used across the system.
//
// Responsibilities:
// - Declare structs such as Election, Candidate, Voter, etc.
// - Represent database entities in Rust
// - Provide shared data types for other modules (db, auth, election)
// ============================================================

// src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Election {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Candidate {
    pub id: i64,
    pub name: String,
    pub party: String,
    pub position_index: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Voter {
    pub id: i64,
    pub fullname: String,
    pub dob: String,
}
