# 8b.is Ada 2022 credit

- Project: **Add new crates to Alire**
- Source: <https://ada-lang.io/docs/projects-to-work-on/#alire>
- Contribution: `wst_brain_scan` Alire crate — Ada 2022 bent balanced-ternary
  verification lane for the 8b.is Weight-Space-Time deck.
- Credit: **8b.is** — `authors`, `maintainers`, and `website` in `alire.toml`.
- Upstream submission target: <https://github.com/alire-project/alire-index>

## Prepared submission artifacts

- `alire.toml` — workspace/crate manifest (name, version, license, tags,
  `[[depends-on]] gnat = ">=12"`).
- `wst_brain_scan.gpr` — GNAT project (Ada 2022, `-gnat2022 -gnata -gnatwa`).
- `ada/wst_brain_scan.ads` / `.adb` / `wst_brain_scan_test.adb` — package spec,
  body, and 14-trit cross-check executable.
- `LICENSE` — MIT (required for indexing; `licenses = "MIT"` in the manifest).
- `alire-index/wst_brain_scan-0.1.0.toml` — the Alire index entry. It uses a
  monorepo `[origin]` with `subdir = "crates/wst_brain_scan"` pointing into
  `github.com/8b-is/smart-tree`, so no standalone repo is required.

Index path in `alire-project/alire-index`:

```text
index/ws/wst_brain_scan/wst_brain_scan-0.1.0.toml
```

## Remaining steps (blocked in this environment)

1. **Install the Ada toolchain.** `alr`, `gprbuild`, and `gnat` are not
   Homebrew formulas on this machine (`brew search alr gnat alire gprbuild`
   finds nothing). Install Alire from its GitHub release
   (`alire-project/alire`) or a tap, then `alr toolchain --select`.
2. **Verify the build.** `alr build` followed by `./obj/wst_brain_scan_test`
   must print `Ada 2022 lane: agreed 14, disagreed 0`.
3. **Pin the origin commit.** The index entry's `[origin] commit` is the
   current `smart-tree` HEAD; refresh it at submission time to a commit/tag
   where `crates/wst_brain_scan/` is in its final state.
4. **Open the PR.** Fork `alire-project/alire-index`, add
   `index/ws/wst_brain_scan/wst_brain_scan-0.1.0.toml`, and open a pull request
   (`gh auth login` is required — GitHub authentication is not configured here).
5. **Confirm the compiler dependency form.** `[[depends-on]] gnat = ">=12"`
   mirrors `alire.toml`; the Alire maintainers may ask for `gnat_external` or
   `gnat_native` during review.

The crate is intentionally small and self-contained so the Alire index review
can reproduce it with `alr build` and `./obj/wst_brain_scan_test`.
