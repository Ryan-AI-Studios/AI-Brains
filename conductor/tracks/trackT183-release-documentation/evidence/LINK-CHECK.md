# T183 Relative link check

**Date:** 2026-08-01  
**Method:** PowerShell extract of markdown links `[text](target)` from:

- `README.md`
- `Docs/README.md`
- `Docs/INSTALL.md`
- `Docs/SECURITY-LIMITS.md`
- `SECURITY.md`
- (also checked pack-adjacent) `Docs/ARCHITECTURE.md`, `Docs/CAPABILITIES.md`, `CHANGELOG.md`

Skip: `http(s)://`, `mailto:`, pure `#fragment` anchors.

Resolve relative to the file’s directory; `Test-Path` on absolute resolved path.

## Result

```
total_links=144 ok=144 fail=0
ALL_RELATIVE_LINKS_OK
```

No broken relative targets at implement time.
