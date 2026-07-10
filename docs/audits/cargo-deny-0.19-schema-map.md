# cargo-deny 0.19 schema map

`cargo-deny` 0.19.8 validates the following configuration updates:

| Previous configuration | 0.19.8-compatible configuration | Policy effect |
| --- | --- | --- |
| `[advisories] unmaintained = "warn"` | `unmaintained = "workspace"` | Reports unmaintained direct workspace dependencies; `"warn"` is not a valid value. |
| `[bans] highlight = ["rustls", ...]` | `highlight = "all"` | Displays all dependency paths for duplicate-version findings; per-crate highlight lists are no longer supported. |
| `[licenses.copyleft]` with `strong` and `weak` keys | Remove the table | Copyleft policy is expressed by the `[licenses].allow` list in 0.19.8. |

The remaining policy continues to deny unknown sources, warn on multiple versions
and wildcard dependency specifications, and enforce the existing license allow-list.
