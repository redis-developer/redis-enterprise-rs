//! Read-only tour of database action endpoints.
//!
//! Picks the first database on the cluster and demonstrates the
//! status-and-availability action endpoints that don't mutate state:
//! `availability`, `endpoint_availability`, `recover_status`,
//! `optimize_shards_placement` (status), and `metrics`.
//!
//! Destructive actions (`recover`, `export`, `import`, `flush`,
//! `upgrade_redis_version`, `reset_admin_pass`, `stop_traffic`,
//! `resume_traffic`) are intentionally NOT exercised by this example —
//! they're documented on `client.databases()` and the rustdocs explain
//! the per-method contract.
//!
//! Run with:
//! ```bash
//! REDIS_ENTERPRISE_URL=... REDIS_ENTERPRISE_USER=... REDIS_ENTERPRISE_PASSWORD=... \
//!   REDIS_ENTERPRISE_INSECURE=true \
//!   cargo run --example database_actions
//! ```

use redis_enterprise::EnterpriseClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = EnterpriseClient::from_env()?;

    println!("Database actions tour — read-only");
    println!("==================================\n");

    let databases = client.databases().list().await?;
    let Some(db) = databases.first() else {
        println!("No databases on this cluster. Run the `cluster_setup_simple`");
        println!("example first to create a disposable one.");
        return Ok(());
    };

    let uid = db.uid;
    println!("Inspecting database \"{}\" (uid {})", db.name, uid);
    println!(
        "  status={:?}, memory_size={} bytes, port={:?}\n",
        db.status,
        db.memory_size.unwrap_or(0),
        db.port
    );

    // availability — whether the bdb is reachable on the data plane
    println!("availability:");
    match client.databases().availability(uid).await {
        Ok(v) => println!("  {}", serde_json::to_string(&v)?),
        Err(e) => eprintln!("  ⚠ {e}"),
    }

    // endpoint_availability — same check, scoped to the endpoint shape
    println!("\nendpoint_availability:");
    match client.databases().endpoint_availability(uid).await {
        Ok(v) => println!("  {}", serde_json::to_string(&v)?),
        Err(e) => eprintln!("  ⚠ {e}"),
    }

    // recover_status — status of any in-flight recovery
    println!("\nrecover_status:");
    match client.databases().recover_status(uid).await {
        Ok(v) => println!("  {}", serde_json::to_string(&v)?),
        Err(e) => eprintln!("  ⚠ {e}"),
    }

    // optimize_shards_placement — status of the placement optimizer
    println!("\noptimize_shards_placement (status):");
    match client.databases().optimize_shards_placement(uid).await {
        Ok(v) => println!("  {}", serde_json::to_string(&v)?),
        Err(e) => eprintln!("  ⚠ {e}"),
    }

    // metrics — last metrics snapshot for the database
    println!("\nmetrics:");
    match client.databases().metrics(uid).await {
        Ok(v) => {
            let s = serde_json::to_string(&v)?;
            let preview = s.chars().take(200).collect::<String>();
            println!(
                "  {preview}{}",
                if s.len() > 200 { " …(truncated)" } else { "" }
            );
        }
        Err(e) => eprintln!("  ⚠ {e}"),
    }

    println!("\nDone. To run destructive actions, see the per-method rustdoc on");
    println!("`client.databases()` — for example `flush`, `recover`, or");
    println!("`upgrade_redis_version`.");

    Ok(())
}
