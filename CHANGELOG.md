# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.8.7...v0.9.0) - 2026-05-26

### Added

- *(modules)* add user-defined module management endpoints ([#90](https://github.com/redis-developer/redis-enterprise-rs/pull/90))
- *(cluster)* add SSO and SAML metadata endpoints ([#89](https://github.com/redis-developer/redis-enterprise-rs/pull/89))
- *(crdb)* add flush, health_report, purge, and updates endpoints ([#88](https://github.com/redis-developer/redis-enterprise-rs/pull/88))
- *(cluster)* add change_password_hashing_algorithm endpoint ([#87](https://github.com/redis-developer/redis-enterprise-rs/pull/87))
- *(cluster, nodes, users)* add check() endpoints and RBAC-friendly user role/role_uids ([#72](https://github.com/redis-developer/redis-enterprise-rs/pull/72))

### Fixed

- *(alerts)* list_cluster_alerts returns HashMap<String, ClusterAlertState> ([#83](https://github.com/redis-developer/redis-enterprise-rs/pull/83))
- *(bootstrap)* reshape status response to match the documented spec wrapper ([#82](https://github.com/redis-developer/redis-enterprise-rs/pull/82))
- *(types)* unwrap {crdbs} / {bdb_groups} wrappers on list endpoints ([#81](https://github.com/redis-developer/redis-enterprise-rs/pull/81))
- *(proxies)* make optional fields Optional and widen maxmemory_clients to u64 ([#80](https://github.com/redis-developer/redis-enterprise-rs/pull/80))
- *(actions)* correctly decode the {actions, state-machines} wrapper; string-typed progress/node_uid ([#79](https://github.com/redis-developer/redis-enterprise-rs/pull/79))
- *(builder)* reject missing credentials ([#44](https://github.com/redis-developer/redis-enterprise-rs/pull/44))
- *(ocsp)* test() uses POST (the documented verb), not GET ([#78](https://github.com/redis-developer/redis-enterprise-rs/pull/78))
- *(nodes)* execute_action POSTs to /v1/nodes/{uid}/actions/{action}, the documented path ([#77](https://github.com/redis-developer/redis-enterprise-rs/pull/77))
- *(bdb)* flush() and reset_admin_pass() use the documented PUT path-segment endpoints ([#76](https://github.com/redis-developer/redis-enterprise-rs/pull/76))
- *(crdb)* UPDATE uses PATCH verb (the documented one), not PUT ([#75](https://github.com/redis-developer/redis-enterprise-rs/pull/75))
- *(stats, crdb_tasks)* handle real-API response shapes and use the documented action endpoints ([#71](https://github.com/redis-developer/redis-enterprise-rs/pull/71))

### Other

- README + examples refresh for v0.9.0 ([#96](https://github.com/redis-developer/redis-enterprise-rs/pull/96))
- remove fictional services / jsonschema / usage_report routes ([#95](https://github.com/redis-developer/redis-enterprise-rs/pull/95))
- *(bdb)* remove fictional actions/{backup,restore,upgrade} ([#94](https://github.com/redis-developer/redis-enterprise-rs/pull/94))
- *(alerts)* remove fictional /v1/alerts* top-level routes ([#93](https://github.com/redis-developer/redis-enterprise-rs/pull/93))
- *(audit)* per-route triage for the 122 non-spec SDK routes ([#92](https://github.com/redis-developer/redis-enterprise-rs/pull/92))
- *(live-validation)* add TLS, licensing, and teardown sections; link from README ([#91](https://github.com/redis-developer/redis-enterprise-rs/pull/91))
- enforce #![deny(missing_docs)] and rustdoc::broken_intra_doc_links ([#86](https://github.com/redis-developer/redis-enterprise-rs/pull/86))
- *(missing-docs)* document fields and enum variants across the crate ([#85](https://github.com/redis-developer/redis-enterprise-rs/pull/85))
- *(coverage)* add route-coverage test against the docs API inventory ([#84](https://github.com/redis-developer/redis-enterprise-rs/pull/84))
- update dependency versions ([#46](https://github.com/redis-developer/redis-enterprise-rs/pull/46))
- *(env)* document CA cert variable ([#45](https://github.com/redis-developer/redis-enterprise-rs/pull/45))
- *(live)* add opt-in live integration smoke tests against a real Redis Enterprise cluster ([#74](https://github.com/redis-developer/redis-enterprise-rs/pull/74))
- *(infra)* add API audit docs + scripts and modernize docker-compose ([#70](https://github.com/redis-developer/redis-enterprise-rs/pull/70))

## [0.8.7](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.8.6...v0.8.7) - 2026-03-19

### Fixed

- handle empty 200 response from PUT /v1/license ([#39](https://github.com/redis-developer/redis-enterprise-rs/pull/39))

### Other

- release v0.9.0 ([#37](https://github.com/redis-developer/redis-enterprise-rs/pull/37))

## [0.8.6](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.8.5...v0.8.6) - 2026-03-19

### Fixed

- distinguish TLS certificate errors from connection failures ([#36](https://github.com/redis-developer/redis-enterprise-rs/pull/36))

### Other

- Fix Python packaging for Linux wheel installs ([#35](https://github.com/redis-developer/redis-enterprise-rs/pull/35))

## [0.8.5](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.8.4...v0.8.5) - 2026-02-10

### Fixed

- sync Python package version with Rust crate ([#31](https://github.com/redis-developer/redis-enterprise-rs/pull/31))

### Other

- Fix module platforms deserialization - use HashMap instead of Vec ([#33](https://github.com/redis-developer/redis-enterprise-rs/pull/33))

## [0.8.4](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.8.3...v0.8.4) - 2026-02-05

### Other

- update rust-version to 1.89 and author email ([#29](https://github.com/redis-developer/redis-enterprise-rs/pull/29))

## [0.8.3](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.8.2...v0.8.3) - 2026-02-04

### Added

- add TypedBuilder to bootstrap types and error helper ([#26](https://github.com/redis-developer/redis-enterprise-rs/pull/26))

## [0.8.2](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.8.1...v0.8.2) - 2026-02-03

### Added

- add custom CA certificate support and upgrade reqwest to 0.13 ([#21](https://github.com/redis-developer/redis-enterprise-rs/pull/21))

### Other

- add CA cert tests and client_builder() for test support ([#24](https://github.com/redis-developer/redis-enterprise-rs/pull/24))
- update bytes to 1.11.1 (RUSTSEC-2026-0007) ([#22](https://github.com/redis-developer/redis-enterprise-rs/pull/22))

## [0.8.1](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.8.0...v0.8.1) - 2026-01-31

### Added

- add test-support feature for consumer testing ([#17](https://github.com/redis-developer/redis-enterprise-rs/pull/17))

### Fixed

- add AlertFixture and alerts mocking support ([#19](https://github.com/redis-developer/redis-enterprise-rs/pull/19))

## [0.8.0](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.7.5...v0.8.0) - 2026-01-30

### Added

- add more specific error variants ([#12](https://github.com/redis-developer/redis-enterprise-rs/pull/12))
- add fluent API and align with redis-cloud patterns ([#10](https://github.com/redis-developer/redis-enterprise-rs/pull/10))

### Fixed

- add README to PyPI package ([#3](https://github.com/redis-developer/redis-enterprise-rs/pull/3))

### Other

- cleanup README and Python bindings ([#15](https://github.com/redis-developer/redis-enterprise-rs/pull/15))
- split database tests into modular structure ([#14](https://github.com/redis-developer/redis-enterprise-rs/pull/14))
- reduce handler boilerplate with macros ([#13](https://github.com/redis-developer/redis-enterprise-rs/pull/13))
- remove extra: Value fields from all response types ([#11](https://github.com/redis-developer/redis-enterprise-rs/pull/11))

## [0.7.5](https://github.com/redis-developer/redis-enterprise-rs/compare/v0.7.4...v0.7.5) - 2026-01-30

### Added

- add Python bindings ([#2](https://github.com/redis-developer/redis-enterprise-rs/pull/2))
- initial standalone redis-enterprise crate

## [0.7.4](https://github.com/redis-developer/redisctl/compare/redis-enterprise-v0.7.3...redis-enterprise-v0.7.4) - 2026-01-23

### Added

- Add Python bindings via PyO3 ([#578](https://github.com/redis-developer/redisctl/pull/578))

### Fixed

- use local README.md for crates to fix sdist build ([#580](https://github.com/redis-developer/redisctl/pull/580))

## [0.7.3](https://github.com/redis-developer/redisctl/compare/redis-enterprise-v0.7.2...redis-enterprise-v0.7.3) - 2026-01-12

### Added

- add MCP server for AI integration ([#531](https://github.com/redis-developer/redisctl/pull/531))

## [0.7.2](https://github.com/redis-developer/redisctl/compare/redis-enterprise-v0.7.1...redis-enterprise-v0.7.2) - 2025-12-17

### Fixed

- support JMESPath backtick string literals and improve module upload error ([#511](https://github.com/redis-developer/redisctl/pull/511))

### Other

- update documentation URLs to new hosting location ([#509](https://github.com/redis-developer/redisctl/pull/509))

## [0.7.1](https://github.com/redis-developer/redisctl/compare/redis-enterprise-v0.7.0...redis-enterprise-v0.7.1) - 2025-12-16

### Other

- switch to GHCR for Docker images ([#500](https://github.com/redis-developer/redisctl/pull/500))
- update repository URLs for redis-developer org ([#499](https://github.com/redis-developer/redisctl/pull/499))

## [0.7.0](https://github.com/joshrotenberg/redisctl/compare/redis-enterprise-v0.6.4...redis-enterprise-v0.7.0) - 2025-12-09

### Added

- add user agent header to HTTP requests ([#473](https://github.com/joshrotenberg/redisctl/pull/473))
- *(enterprise)* add database watch command for real-time status monitoring ([#458](https://github.com/joshrotenberg/redisctl/pull/458))
- *(redis-enterprise)* add stats streaming with --follow flag ([#455](https://github.com/joshrotenberg/redisctl/pull/455))
- Add optional Tower service integration to API clients ([#447](https://github.com/joshrotenberg/redisctl/pull/447))
- add database upgrade command for Redis version upgrades ([#442](https://github.com/joshrotenberg/redisctl/pull/442))

### Fixed

- *(redis-enterprise)* remove non-existent database action methods ([#443](https://github.com/joshrotenberg/redisctl/pull/443))
- *(release)* improve Homebrew formula auto-update ([#433](https://github.com/joshrotenberg/redisctl/pull/433))

## [0.6.4](https://github.com/joshrotenberg/redisctl/compare/redis-enterprise-v0.6.3...redis-enterprise-v0.6.4) - 2025-10-29

### Added

- Add streaming logs support with --follow flag (Issue #70) ([#404](https://github.com/joshrotenberg/redisctl/pull/404))

### Other

- add comprehensive presentation outline and rladmin comparison ([#415](https://github.com/joshrotenberg/redisctl/pull/415))
- rewrite README for presentation readiness ([#408](https://github.com/joshrotenberg/redisctl/pull/408))
- implement fixture-based validation for Enterprise API ([#352](https://github.com/joshrotenberg/redisctl/pull/352)) ([#398](https://github.com/joshrotenberg/redisctl/pull/398))

## [0.6.3](https://github.com/joshrotenberg/redisctl/compare/redis-enterprise-v0.6.2...redis-enterprise-v0.6.3) - 2025-10-07

### Other

- add support package optimization and upload documentation
- add Homebrew installation instructions

## [0.6.1](https://github.com/joshrotenberg/redisctl/compare/redis-enterprise-v0.6.0...redis-enterprise-v0.6.1) - 2025-09-16

### Added

- add serde_path_to_error for better deserialization error messages ([#349](https://github.com/joshrotenberg/redisctl/pull/349))

### Fixed

- *(redis-enterprise)* correct max_aof_file_size type from String to u64 ([#351](https://github.com/joshrotenberg/redisctl/pull/351))
- *(redis-enterprise)* correct master_persistence type from String to bool ([#348](https://github.com/joshrotenberg/redisctl/pull/348))