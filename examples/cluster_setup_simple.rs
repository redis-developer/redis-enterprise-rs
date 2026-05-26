//! Simple cluster + database setup using the typed fluent API.
//!
//! Creates a small disposable database, polls until it goes active, prints a
//! few facts about it, then cleans up. Intended to be run against a local
//! Redis Enterprise cluster brought up via the
//! [live-validation runbook](../docs/live-validation.md).
//!
//! Run with:
//! ```bash
//! REDIS_ENTERPRISE_URL=https://localhost:9443 \
//! REDIS_ENTERPRISE_USER=admin@redis.local \
//! REDIS_ENTERPRISE_PASSWORD=Redis123! \
//! REDIS_ENTERPRISE_INSECURE=true \
//! cargo run --example cluster_setup_simple
//! ```

use redis_enterprise::{CreateDatabaseRequest, EnterpriseClient};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = EnterpriseClient::from_env()?;

    println!("Redis Enterprise — disposable database walkthrough");
    println!("===================================================\n");

    // Step 1: Cluster status via the typed handler.
    println!("Step 1: cluster status");
    match client.cluster().info().await {
        Ok(cluster) => println!("  ✓ cluster initialized: {}\n", cluster.name),
        Err(e) => {
            eprintln!("  ⚠ cluster info failed: {e}");
            eprintln!("  → the cluster may need bootstrap; see docs/live-validation.md\n");
            return Err(e.into());
        }
    }

    let db_name = "redis-enterprise-example";
    let db_port: u16 = 12000;
    let db_password = env::var("REDIS_ENTERPRISE_DB_PASSWORD")
        .unwrap_or_else(|_| "example-Redis123!".to_string());

    // Step 2: Create a small database via the typed builder.
    println!("Step 2: creating database \"{db_name}\" on port {db_port}");
    let request = CreateDatabaseRequest::builder()
        .name(db_name)
        .memory_size(104_857_600) // 100 MiB
        .port(db_port)
        .replication(false)
        .persistence("aof")
        .eviction_policy("volatile-lru")
        .authentication_redis_pass(&db_password)
        .build();

    let created = client.databases().create(request).await?;
    let uid = created.uid;
    println!("  ✓ created uid={uid}, status={:?}\n", created.status);

    // Step 3: Poll until the new database goes active (or give up after ~30s).
    println!("Step 3: waiting for database to become active");
    let active = {
        let mut info = created;
        for _ in 0..15 {
            if info.status.as_deref() == Some("active") {
                break;
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
            info = client.databases().info(uid).await?;
        }
        info
    };
    println!(
        "  ✓ status={:?}, memory_size={} bytes, port={:?}\n",
        active.status,
        active.memory_size.unwrap_or(0),
        active.port
    );

    // Step 4: List all databases.
    println!("Step 4: listing databases");
    for db in client.databases().list().await? {
        println!("  - {} (uid {}, port {:?})", db.name, db.uid, db.port);
    }
    println!();

    // Step 5: Clean up.
    println!("Step 5: deleting \"{db_name}\"");
    client.databases().delete(uid).await?;
    println!("  ✓ deleted uid={uid}\n");

    println!("Done. While the database existed, you could connect with:");
    println!("  redis-cli -p {db_port} -a <password>");

    Ok(())
}
