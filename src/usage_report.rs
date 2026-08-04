//! Usage reporting
//!
//! Redis Software returns `GET /v1/usage_report` as newline-delimited JSON
//! (NDJSON), followed by a final MD5 checksum line. It is not a JSON array.

use crate::client::RestClient;
use crate::error::{RestError, Result};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::pin::Pin;

/// Maximum buffered size of one usage-report line.
///
/// The response as a whole is streamed; this bound prevents a malformed
/// response without newlines from growing memory without limit.
pub const MAX_USAGE_REPORT_LINE_BYTES: usize = 1024 * 1024;

/// One database record from the usage report.
///
/// Fields are optional because the report evolves across supported Redis
/// Software releases. Unknown fields are retained in [`Self::additional_fields`]
/// so callers do not lose additive server data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageReport {
    /// Cluster name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_name: Option<String>,
    /// Cluster UUID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_uuid: Option<String>,
    /// Report timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Redis Software version that produced the record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_version: Option<String>,
    /// Usage report API version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    /// Database UID, encoded as a string by the documented report format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdb_uid: Option<String>,
    /// Database type, such as `core`, `premium`, or `auto_tiering`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// Shard type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_type: Option<String>,
    /// Dominant shard-selection criterion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dominant_shard_criteria: Option<String>,
    /// Provisioned database memory in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisioned_memory: Option<u64>,
    /// Used database memory in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_memory: Option<u64>,
    /// Number of primary shards.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_shards_count: Option<u32>,
    /// Whether the database uses a no-eviction policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_eviction: Option<bool>,
    /// Whether persistence is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<bool>,
    /// Whether backup is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup: Option<bool>,
    /// Whether the database uses Redis Search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub using_redis_search: Option<bool>,
    /// Consolidated operations per second for the database.
    #[serde(alias = "ops/sec", skip_serializing_if = "Option::is_none")]
    pub ops_sec: Option<f64>,
    /// Whether in-memory replication is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replication: Option<bool>,
    /// Whether Active-Active is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_active: Option<bool>,
    /// License usage recorded with this database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<UsageReportLicense>,
    /// Additive or version-specific fields not yet modeled by the crate.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// License information embedded in a usage-report record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageReportLicense {
    /// License activation timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_date: Option<String>,
    /// License expiration timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    /// RAM shards currently in use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_shards_in_use: Option<u64>,
    /// Licensed RAM shard limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_shards_limit: Option<u64>,
    /// Flash shards currently in use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flash_shards_in_use: Option<u64>,
    /// Licensed flash shard limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flash_shards_limit: Option<u64>,
    /// Total licensed shard limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shards_limit: Option<u64>,
    /// Additive or version-specific license fields.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// One line from the usage-report response.
#[derive(Debug, Clone, PartialEq)]
pub enum UsageReportRecord {
    /// A database usage record decoded from one NDJSON line.
    Report(Box<UsageReport>),
    /// The documented final MD5 checksum line.
    Checksum(String),
}

/// Streaming usage-report response.
pub type UsageReportStream =
    Pin<Box<dyn Stream<Item = Result<UsageReportRecord>> + Send + 'static>>;

/// Usage report handler.
pub struct UsageReportHandler {
    client: RestClient,
}

impl UsageReportHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        UsageReportHandler { client }
    }

    /// Stream the usage report one bounded line at a time.
    ///
    /// JSON lines are returned as [`UsageReportRecord::Report`]. The
    /// documented final MD5 line is returned as
    /// [`UsageReportRecord::Checksum`]. An empty HTTP 200 body produces an
    /// empty stream. Malformed JSON, a record after the checksum, a missing
    /// checksum after JSON records, or a line larger than
    /// [`MAX_USAGE_REPORT_LINE_BYTES`] produces a [`RestError::ParseError`]
    /// without including the response line in the diagnostic.
    pub async fn stream(&self) -> Result<UsageReportStream> {
        let response = self
            .client
            .get_streaming_response("/v1/usage_report")
            .await?;
        let mut chunks = response.bytes_stream();

        Ok(Box::pin(async_stream::stream! {
            let mut buffer = Vec::new();
            let mut line_number = 0usize;
            let mut saw_record = false;
            let mut saw_checksum = false;

            while let Some(chunk) = chunks.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        yield Err(RestError::RequestFailed(format!(
                            "usage report response stream failed: {error}"
                        )));
                        return;
                    }
                };

                for byte in chunk.iter().copied() {
                    if byte == b'\n' {
                        line_number += 1;
                        match parse_record(&buffer, line_number) {
                            Ok(Some(record)) => {
                                if saw_checksum {
                                    yield Err(RestError::ParseError(format!(
                                        "usage report record {line_number} appeared after the final checksum"
                                    )));
                                    return;
                                }
                                saw_record = true;
                                saw_checksum = matches!(record, UsageReportRecord::Checksum(_));
                                yield Ok(record);
                            }
                            Ok(None) => {}
                            Err(error) => {
                                yield Err(error);
                                return;
                            }
                        }
                        buffer.clear();
                    } else {
                        buffer.push(byte);
                        if buffer.len() > MAX_USAGE_REPORT_LINE_BYTES {
                            yield Err(RestError::ParseError(format!(
                                "usage report record {} exceeds the {} byte limit",
                                line_number + 1,
                                MAX_USAGE_REPORT_LINE_BYTES
                            )));
                            return;
                        }
                    }
                }
            }

            if !buffer.is_empty() {
                line_number += 1;
                match parse_record(&buffer, line_number) {
                    Ok(Some(record)) => {
                        if saw_checksum {
                            yield Err(RestError::ParseError(format!(
                                "usage report record {line_number} appeared after the final checksum"
                            )));
                            return;
                        }
                        saw_record = true;
                        saw_checksum = matches!(record, UsageReportRecord::Checksum(_));
                        yield Ok(record);
                    }
                    Ok(None) => {}
                    Err(error) => {
                        yield Err(error);
                        return;
                    }
                }
            }

            if saw_record && !saw_checksum {
                yield Err(RestError::ParseError(
                    "usage report response ended without the final MD5 checksum".to_string(),
                ));
            }
        }))
    }

    /// Collect all database records from the streamed usage report.
    ///
    /// The final checksum is consumed but not included in the returned vector.
    /// Prefer [`Self::stream`] for large reports so records can be processed
    /// without retaining the full response in memory.
    pub async fn list(&self) -> Result<Vec<UsageReport>> {
        let mut stream = self.stream().await?;
        let mut reports = Vec::new();

        while let Some(record) = stream.next().await {
            if let UsageReportRecord::Report(report) = record? {
                reports.push(*report);
            }
        }

        Ok(reports)
    }
}

fn parse_record(line: &[u8], line_number: usize) -> Result<Option<UsageReportRecord>> {
    let line = trim_ascii_whitespace(line);
    if line.is_empty() {
        return Ok(None);
    }

    if line.len() == 32 && line.iter().all(u8::is_ascii_hexdigit) {
        let checksum = std::str::from_utf8(line).map_err(|_| {
            RestError::ParseError(format!(
                "usage report checksum at record {line_number} is not valid ASCII"
            ))
        })?;
        return Ok(Some(UsageReportRecord::Checksum(checksum.to_string())));
    }

    let deserializer = &mut serde_json::Deserializer::from_slice(line);
    let report = serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let path = error.path().to_string();
        RestError::ParseError(format!(
            "failed to deserialize usage report record {line_number} at field '{path}': {}",
            error.inner()
        ))
    })?;
    Ok(Some(UsageReportRecord::Report(Box::new(report))))
}

fn trim_ascii_whitespace(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}
