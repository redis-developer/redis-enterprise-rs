//! Pre-built fixtures for testing Redis Enterprise API responses
//!
//! All fixtures use the builder pattern for customization.
//!
//! # Example
//!
//! ```
//! use redis_enterprise::testing::fixtures::{DatabaseFixture, NodeFixture, ClusterFixture};
//!
//! // Create a database with defaults
//! let db = DatabaseFixture::new(1, "my-cache").build();
//!
//! // Customize as needed
//! let db = DatabaseFixture::new(2, "sessions")
//!     .memory_size(2 * 1024 * 1024 * 1024) // 2GB
//!     .port(12001)
//!     .status("creating")
//!     .build();
//! ```

use serde_json::{Value, json};

/// Builder for database (BDB) fixtures
#[derive(Debug, Clone)]
pub struct DatabaseFixture {
    uid: u32,
    name: String,
    memory_size: u64,
    port: u16,
    status: String,
    db_type: String,
    replication: bool,
    persistence: String,
    shards_count: u32,
}

impl DatabaseFixture {
    /// Create a new database fixture with required fields
    pub fn new(uid: u32, name: impl Into<String>) -> Self {
        Self {
            uid,
            name: name.into(),
            memory_size: 1024 * 1024 * 1024, // 1GB default
            port: 12000 + (uid as u16),
            status: "active".to_string(),
            db_type: "redis".to_string(),
            replication: false,
            persistence: "disabled".to_string(),
            shards_count: 1,
        }
    }

    /// Set memory size in bytes
    pub fn memory_size(mut self, size: u64) -> Self {
        self.memory_size = size;
        self
    }

    /// Set the database port
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the database status
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Set the database type
    pub fn db_type(mut self, db_type: impl Into<String>) -> Self {
        self.db_type = db_type.into();
        self
    }

    /// Enable or disable replication
    pub fn replication(mut self, enabled: bool) -> Self {
        self.replication = enabled;
        self
    }

    /// Set persistence mode
    pub fn persistence(mut self, mode: impl Into<String>) -> Self {
        self.persistence = mode.into();
        self
    }

    /// Set number of shards
    pub fn shards_count(mut self, count: u32) -> Self {
        self.shards_count = count;
        self
    }

    /// Build the JSON fixture
    pub fn build(self) -> Value {
        json!({
            "uid": self.uid,
            "name": self.name,
            "type": self.db_type,
            "memory_size": self.memory_size,
            "port": self.port,
            "status": self.status,
            "replication": self.replication,
            "data_persistence": self.persistence,
            "shards_count": self.shards_count
        })
    }
}

/// Builder for node fixtures
#[derive(Debug, Clone)]
pub struct NodeFixture {
    uid: u32,
    addr: String,
    status: String,
    total_memory: u64,
    cores: u32,
    rack_id: Option<String>,
    os_version: String,
}

impl NodeFixture {
    /// Create a new node fixture with required fields
    pub fn new(uid: u32, addr: impl Into<String>) -> Self {
        Self {
            uid,
            addr: addr.into(),
            status: "active".to_string(),
            total_memory: 8 * 1024 * 1024 * 1024, // 8GB default
            cores: 4,
            rack_id: None,
            os_version: "Ubuntu 22.04".to_string(),
        }
    }

    /// Set node status
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Set total memory in bytes
    pub fn total_memory(mut self, memory: u64) -> Self {
        self.total_memory = memory;
        self
    }

    /// Set number of CPU cores
    pub fn cores(mut self, cores: u32) -> Self {
        self.cores = cores;
        self
    }

    /// Set rack ID
    pub fn rack_id(mut self, rack_id: impl Into<String>) -> Self {
        self.rack_id = Some(rack_id.into());
        self
    }

    /// Set OS version
    pub fn os_version(mut self, version: impl Into<String>) -> Self {
        self.os_version = version.into();
        self
    }

    /// Build the JSON fixture
    pub fn build(self) -> Value {
        let mut obj = json!({
            "uid": self.uid,
            "addr": self.addr,
            "status": self.status,
            "total_memory": self.total_memory,
            "cores": self.cores,
            "os_version": self.os_version
        });

        if let Some(rack_id) = self.rack_id {
            obj["rack_id"] = json!(rack_id);
        }

        obj
    }
}

/// Builder for cluster info fixtures
#[derive(Debug, Clone)]
pub struct ClusterFixture {
    name: String,
    nodes: Vec<u32>,
}

impl ClusterFixture {
    /// Create a new cluster fixture
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: vec![1],
        }
    }

    /// Set the list of node UIDs in the cluster
    pub fn nodes(mut self, nodes: Vec<u32>) -> Self {
        self.nodes = nodes;
        self
    }

    /// Build the JSON fixture
    pub fn build(self) -> Value {
        json!({
            "name": self.name,
            "nodes": self.nodes
        })
    }
}

/// Builder for user fixtures
#[derive(Debug, Clone)]
pub struct UserFixture {
    uid: u32,
    email: String,
    name: Option<String>,
    role: String,
    status: String,
}

impl UserFixture {
    /// Create a new user fixture
    pub fn new(uid: u32, email: impl Into<String>) -> Self {
        Self {
            uid,
            email: email.into(),
            name: None,
            role: "admin".to_string(),
            status: "active".to_string(),
        }
    }

    /// Set user's display name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set user's role
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }

    /// Set user's status
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Build the JSON fixture
    pub fn build(self) -> Value {
        let mut obj = json!({
            "uid": self.uid,
            "email": self.email,
            "role": self.role,
            "status": self.status
        });

        if let Some(name) = self.name {
            obj["name"] = json!(name);
        }

        obj
    }
}

/// Builder for license fixtures
#[derive(Debug, Clone)]
pub struct LicenseFixture {
    expired: bool,
    shards_limit: u32,
    expiration_date: Option<String>,
}

impl LicenseFixture {
    /// Create a new license fixture (non-expired by default)
    pub fn new() -> Self {
        Self {
            expired: false,
            shards_limit: 100,
            expiration_date: Some("2030-12-31".to_string()),
        }
    }

    /// Create an expired license fixture
    pub fn expired() -> Self {
        Self {
            expired: true,
            shards_limit: 0,
            expiration_date: Some("2020-01-01".to_string()),
        }
    }

    /// Set shards limit
    pub fn shards_limit(mut self, limit: u32) -> Self {
        self.shards_limit = limit;
        self
    }

    /// Set expiration date
    pub fn expiration_date(mut self, date: impl Into<String>) -> Self {
        self.expiration_date = Some(date.into());
        self
    }

    /// Build the JSON fixture
    pub fn build(self) -> Value {
        let mut obj = json!({
            "expired": self.expired,
            "shards_limit": self.shards_limit
        });

        if let Some(date) = self.expiration_date {
            obj["expiration_date"] = json!(date);
        }

        obj
    }
}

impl Default for LicenseFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for action response fixtures
#[derive(Debug, Clone)]
pub struct ActionFixture {
    action_uid: String,
    status: String,
}

impl ActionFixture {
    /// Create a new action fixture
    pub fn new(action_uid: impl Into<String>) -> Self {
        Self {
            action_uid: action_uid.into(),
            status: "completed".to_string(),
        }
    }

    /// Set action status
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Build the JSON fixture
    pub fn build(self) -> Value {
        json!({
            "action_uid": self.action_uid,
            "status": self.status
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_fixture_defaults() {
        let db = DatabaseFixture::new(1, "test-db").build();
        assert_eq!(db["uid"], 1);
        assert_eq!(db["name"], "test-db");
        assert_eq!(db["status"], "active");
    }

    #[test]
    fn test_database_fixture_customized() {
        let db = DatabaseFixture::new(2, "custom")
            .memory_size(2 * 1024 * 1024 * 1024)
            .port(12345)
            .status("creating")
            .replication(true)
            .build();

        assert_eq!(db["uid"], 2);
        assert_eq!(db["memory_size"], 2 * 1024 * 1024 * 1024u64);
        assert_eq!(db["port"], 12345);
        assert_eq!(db["status"], "creating");
        assert_eq!(db["replication"], true);
    }

    #[test]
    fn test_node_fixture() {
        let node = NodeFixture::new(1, "10.0.0.1")
            .rack_id("rack-a")
            .cores(8)
            .build();

        assert_eq!(node["uid"], 1);
        assert_eq!(node["addr"], "10.0.0.1");
        assert_eq!(node["rack_id"], "rack-a");
        assert_eq!(node["cores"], 8);
    }

    #[test]
    fn test_license_fixture() {
        let license = LicenseFixture::new().shards_limit(50).build();
        assert_eq!(license["expired"], false);
        assert_eq!(license["shards_limit"], 50);

        let expired = LicenseFixture::expired().build();
        assert_eq!(expired["expired"], true);
    }
}
