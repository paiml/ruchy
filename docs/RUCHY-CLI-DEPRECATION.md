# ruchy-cli Deprecation Notice

**Date**: 2025-10-13  
**Status**: DEPRECATED AND YANKED

## Summary

The `ruchy-cli` crate has been permanently deprecated and all 46 versions have been yanked from crates.io.

## Reason

The `ruchy-cli` package was MUDA (waste) - it duplicated functionality already available in the main `ruchy` crate. Having two separate packages created:
- Maintenance overhead
- User confusion
- Version synchronization issues
- Unnecessary repository complexity

## Migration

**Old (deprecated)**:
```bash
cargo install ruchy-cli
```

**New (correct)**:
```bash
cargo install ruchy
```

All CLI functionality is available in the main `ruchy` binary.

## Yanked Versions

All 46 versions yanked on 2025-10-13:
- 0.11.3, 0.11.2, 0.11.1, 0.11.0
- 0.10.2, 0.10.1, 0.10.0
- 0.9.12, 0.9.8, 0.9.7, 0.9.6, 0.9.5, 0.9.4, 0.9.0
- 0.8.0
- 0.7.22, 0.7.21, 0.7.20, 0.7.19, 0.7.18, 0.7.17, 0.7.16, 0.7.15, 0.7.14, 0.7.13, 0.7.12, 0.7.7, 0.7.6, 0.7.5, 0.7.4, 0.7.2, 0.7.1, 0.7.0
- 0.6.0
- 0.5.0
- 0.4.13, 0.4.12, 0.4.11, 0.4.6, 0.4.4, 0.4.3, 0.4.2, 0.4.1
- 0.3.2, 0.3.1
- 0.2.1
- 0.1.0

## Actions Taken

1. ✅ Yanked all 46 versions from crates.io (2025-10-13)
2. ✅ Removed all references from CLAUDE.md
3. ✅ Removed all references from Makefile
4. ✅ Removed publication logic from scripts/publish-crates.sh
5. ✅ Verified no ruchy-cli directory in repository

## Verification

```bash
# Verify all versions are yanked
curl -s https://crates.io/api/v1/crates/ruchy-cli | jq -r '.versions[] | "\(.num) - yanked: \(.yanked)"'

# All should show "yanked: true"
```

## Toyota Way Principle

**MUDA Elimination**: Removed waste (duplicate package) to simplify maintenance and reduce user confusion.
