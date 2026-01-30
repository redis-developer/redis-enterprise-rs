//! Basic example of using the Redis Enterprise API client
//!
//! This example shows how to:
//! - Connect to a Redis Enterprise cluster
//! - Get cluster information
//! - List databases and nodes
//!
//! Run with: cargo run --example basic_enterprise

use redis_enterprise::EnterpriseClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get cluster credentials from environment variables
    let url =
        env::var("REDIS_ENTERPRISE_URL").unwrap_or_else(|_| "https://localhost:9443".to_string());
    let username =
        env::var("REDIS_ENTERPRISE_USER").unwrap_or_else(|_| "admin@redis.local".to_string());
    let password = env::var("REDIS_ENTERPRISE_PASSWORD")
        .expect("REDIS_ENTERPRISE_PASSWORD environment variable not set");

    // Check if we should skip SSL verification (for development/self-signed certs)
    let insecure = env::var("REDIS_ENTERPRISE_INSECURE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    // Create the client using the builder pattern
    let client = EnterpriseClient::builder()
        .base_url(&url)
        .username(&username)
        .password(&password)
        .insecure(insecure)
        .build()?;

    // Get cluster information using fluent API
    println!("Fetching cluster information...");
    let cluster_info = client.cluster().info().await?;
    println!("Cluster Name: {}", cluster_info.name);
    println!();

    // List all nodes using fluent API
    println!("Fetching nodes...");
    let nodes = client.nodes().list().await?;

    println!("Found {} node(s):", nodes.len());
    for node in &nodes {
        println!("  - Node {}: {:?} ({})", node.uid, node.addr, node.status);
    }
    println!();

    // List all databases (BDBs) using fluent API
    println!("Fetching databases...");
    let databases = client.databases().list().await?;

    println!("Found {} database(s):", databases.len());
    for db in &databases {
        println!(
            "  - BDB {}: {} (Memory: {} MB, Status: {})",
            db.uid,
            db.name,
            db.memory_size.unwrap_or(0) / (1024 * 1024),
            db.status.as_deref().unwrap_or("unknown")
        );
    }

    Ok(())
}
