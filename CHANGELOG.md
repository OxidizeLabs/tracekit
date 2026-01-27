# Changelog

All notable changes to tracekit will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### tracekit-formats
- **ARC trace format parser** - Space-separated format from ARC research (`timestamp key [size]`)
  - Source: [moka-rs/cache-trace](https://github.com/moka-rs/cache-trace)
  - Use case: Academic research traces (IBM, storage systems)
- **LIRS trace format parser** - One block number per line from LIRS paper
  - Source: LIRS paper traces, Caffeine simulator resources
  - Use case: Storage and database workload traces
- **CSV trace format parser** - Configurable CSV with flexible column mapping
  - Supports custom column ordering, headers, and delimiters
  - Pre-configured modes: key-only, TSV
- **Cachelib trace format parser** - Facebook/Meta Cachelib CSV format
  - Source: [Cachelib Cachebench](https://cachelib.org/docs/Cache_Library_User_Guides/Cachebench_FB_HW_eval/)
  - String key support (automatically hashed to u64)
  - Production trace patterns (CDN, social media)

#### tracekit-cli
- Extended `simulate` command to support new trace formats: `arc`, `lirs`, `csv`, `cachelib`
- Extended `rewrite` command to convert between all supported formats

#### Documentation
- `tracekit-formats/README.md` - Comprehensive format documentation with usage examples
- `docs/REAL_TRACES.md` - Complete workflow guide for working with real-world traces
  - Trace analysis best practices
  - Large trace handling techniques
  - Troubleshooting guide
  - Links to trace repositories
- `tracekit/examples/real_trace.rs` - Example demonstrating trace analysis

#### Features
- Feature flags for trace formats: `arc`, `lirs`, `csv`, `cachelib`
- `full` feature flag to enable all trace format parsers

### Changed
- `tracekit-formats` default features now include `arc`, `lirs`, and `csv`
- Main README updated with trace format examples and links to trace sources
