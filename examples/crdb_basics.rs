//! Read-only Active-Active (CRDB) walkthrough.
//!
//! Lists the cluster's CRDBs and, if at least one exists, fetches its details
//! and runs the per-CRDB health report. Strictly read-only — never creates,
//! deletes, flushes, or purges anything.
//!
//! Run with:
//! ```bash
//! REDIS_ENTERPRISE_URL=... REDIS_ENTERPRISE_USER=... REDIS_ENTERPRISE_PASSWORD=... \
//!   REDIS_ENTERPRISE_INSECURE=true \
//!   cargo run --example crdb_basics
//! ```
//!
//! The flush / purge / updates endpoints exposed by `client.crdb()` are
//! destructive against a live multi-region deployment; see
//! `docs/api-inventory.csv` and the rustdocs for those operations.

use redis_enterprise::EnterpriseClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = EnterpriseClient::from_env()?;

    println!("Active-Active (CRDB) walkthrough — read-only");
    println!("==============================================\n");

    let crdbs = client.crdb().list().await?;

    if crdbs.is_empty() {
        println!("No Active-Active databases configured on this cluster.");
        println!("Create one through the Admin UI or `client.crdb().create(...)`");
        println!("before re-running this example to see the per-CRDB sections.");
        return Ok(());
    }

    println!("Found {} CRDB(s):\n", crdbs.len());
    for crdb in &crdbs {
        println!(
            "  - {} (guid {}, status {}, {} instance(s))",
            crdb.name,
            crdb.guid,
            crdb.status,
            crdb.instances.len()
        );
    }
    println!();

    // Pick the first CRDB and explore it in more detail.
    let first = &crdbs[0];
    println!("Details for \"{}\" (guid {}):", first.name, first.guid);
    let detailed = client.crdb().get(&first.guid).await?;
    println!("  memory_size: {} bytes", detailed.memory_size);
    println!(
        "  encryption: {}",
        detailed
            .encryption
            .map(|b| b.to_string())
            .unwrap_or_else(|| "(unset)".into())
    );
    println!(
        "  data_persistence: {}",
        detailed.data_persistence.as_deref().unwrap_or("(unset)")
    );

    println!("\n  instances:");
    for instance in &detailed.instances {
        println!(
            "    - id {}, cluster {} ({:?}), status {}",
            instance.id, instance.cluster, instance.cluster_name, instance.status
        );
    }

    // The /health_report endpoint shipped in v0.9.0. It can return a rich,
    // version-specific document so we print it as raw JSON.
    println!("\n  health report:");
    match client.crdb().health_report(&first.guid).await {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report)?),
        Err(e) => eprintln!("    ⚠ health_report failed: {e}"),
    }

    Ok(())
}
