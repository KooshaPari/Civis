# Release Checklist

- [ ] Version and changelog updated
- [ ] Quality gates green
- [ ] Docs built successfully
- [ ] Security review complete
- [ ] Deployment workflow validated
- [ ] User-visible smoke paths checked for the release scope:
  - `.\scripts\agent-smoke.ps1`
  - `.\scripts\pie-validation.ps1` when the release touches playability or attach flow
- [ ] Performance-sensitive changes have an explicit benchmark or budget gate recorded:
  - `just civis-3d-verify`
  - `cargo criterion --bench <target>` or the repo-specific benchmark referenced in the change
