//! Database (BDB) endpoint tests for Redis Enterprise
//!
//! This module is organized into submodules:
//! - `crud`: Basic CRUD operations (list, get, create, delete)
//! - `actions`: Database actions (export, import, backup, restore, upgrade)
//! - `monitoring`: Monitoring endpoints (shards, alerts, peer stats)

#[path = "bdb/common.rs"]
mod common;

#[path = "bdb/actions.rs"]
mod actions;

#[path = "bdb/crud.rs"]
mod crud;

#[path = "bdb/monitoring.rs"]
mod monitoring;
