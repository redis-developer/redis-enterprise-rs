# redis-enterprise Harmonization Plan

Goal: Review redis-enterprise patterns and make minor adjustments for consistency with redis-cloud.

## Current State

redis-enterprise is **already well-structured** with:
- TypedBuilder on request types
- Rich error helpers (7 methods)
- Direct return types (no nested wrappers)
- Consistent handler pattern

## Changes Overview

### 1. Verify TypedBuilder Coverage

**Priority: High**

Audit all request types to ensure TypedBuilder is applied consistently.

**Files to audit:**
- [ ] `src/bdb.rs` - `CreateDatabaseRequest`, `DatabaseUpgradeRequest`
- [ ] `src/bootstrap.rs` - `BootstrapConfig`
- [ ] `src/cluster.rs` - Cluster config types
- [ ] `src/nodes.rs` - Node request types
- [ ] `src/users.rs` - User request types
- [ ] `src/roles.rs` - Role request types
- [ ] `src/redis_acls.rs` - ACL request types
- [ ] `src/ldap_mappings.rs` - LDAP request types

### 2. Add Missing Error Helpers (if any)

**Priority: Low**

Current helpers:
- `is_not_found()`
- `is_unauthorized()`
- `is_server_error()`
- `is_timeout()`
- `is_rate_limited()`
- `is_conflict()`
- `is_cluster_busy()`
- `is_retryable()`

Consider adding for parity with Cloud additions:
```rust
impl RestError {
    pub fn is_bad_request(&self) -> bool {
        matches!(self, RestError::ValidationError(_))
    }
}
```

**Files to update:**
- [ ] `src/error.rs`

### 3. Handler Method Naming Audit

**Priority: Medium**

Verify consistency across all handlers:

| Handler | List | Get | Create | Update | Delete |
|---------|------|-----|--------|--------|--------|
| DatabaseHandler | `list` | `get`/`info` | `create` | `update` | `delete` |
| NodeHandler | `list` | `get` | - | - | `remove` |
| ClusterHandler | - | `info` | - | `update` | - |
| UserHandler | `list` | `get` | `create` | `update` | `delete` |

Ensure all handlers follow the same pattern where applicable.

**Files to audit:**
- [ ] `src/bdb.rs`
- [ ] `src/nodes.rs`
- [ ] `src/cluster.rs`
- [ ] `src/users.rs`
- [ ] `src/roles.rs`
- [ ] `src/shards.rs`
- [ ] `src/stats.rs`

### 4. Documentation Consistency

**Priority: Low**

Ensure all handlers have:
- Module-level doc comments with examples
- Method-level doc comments
- `# Example` sections where helpful

### 5. Raw Method Consistency

**Priority: Low**

Verify raw methods match Cloud's interface:
- `get_raw(path)`
- `post_raw(path, body)`
- `put_raw(path, body)`
- `patch_raw(path, body)`
- `delete_raw(path)`

**Files to check:**
- [ ] `src/client.rs`

## Non-Changes

These are architectural differences that are correct for Enterprise:

- **Synchronous returns** - Enterprise API is sync, returns resources directly
- **No subscription context** - Cluster-scoped by design
- **Direct vectors** - No need for wrapper types

## New Features to Consider

### Watch/Stream Pattern

Enterprise has `watch_database()` returning a stream. Consider if Cloud should have similar for task polling (but that's probably Layer 2).

### Action-based Operations

Enterprise uses action endpoints (`/actions/backup`, `/actions/flush`). This is well-modeled. No changes needed.

## Testing

- [ ] Ensure all existing tests pass
- [ ] Add tests for any new error helpers
- [ ] Verify TypedBuilder coverage with compile tests

## Version

This will be a **patch version bump** (e.g., 0.8.2 -> 0.8.3) if only adding helpers, or **minor** if any breaking changes discovered.
