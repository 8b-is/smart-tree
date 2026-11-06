# WST Brain Scan

Weight-Space-Time brain scanner and bent balanced-ternary verification deck.

The Rust crate records a five-layer diff (C, ASM, Rust, Felix stage A named skip,
Qiskit) and declares Ada 2022 as a sixth lane. Four lanes are executable; Felix
stage A and Ada 2022 execution are named skips. The skips are explicit and never
counted as passes.

## Ada 2022 lane

The Ada package follows the Ada 2022 Reference Manual (ISO/IEC 8652:2023):

- `ada/wst_brain_scan.ads` — package specification.
- `ada/wst_brain_scan.adb` — balanced-trit rotation, ternary NAND, unbent
  extraction, ASM bend recurrence, one-hot qubit-pair mapping, and the
  cross-layer verification function.
- `ada/wst_brain_scan_test.adb` — executable cross-check for the 14-trit vector.

Build when GNAT/ALR is available:

```sh
alr build
./obj/wst_brain_scan_test
```

## Alire crate

`alire.toml` packages the Ada lane as an Alire crate. This is the contribution
path listed under **Add new crates** on
<https://ada-lang.io/docs/projects-to-work-on/>. 8b.is takes credit for the crate
and for the cross-layer verification harness.

## Rust report

```rust
let diff = wst_brain_scan::verify_five_layer_bent_ternary(19952026, 2, 14);
assert_eq!(diff.agreed, 14);
assert_eq!(diff.disagreed, 0);
assert_eq!(diff.executable_layers, 5);
assert_eq!(diff.declared_layers, 6);
assert!(!diff.ada_2022_lane.executed);
```

The `ada_2022_lane.toolchain_available` flag reports whether `gnat`, `gnatmake`,
or `alr` is present, so the named skip is never mistaken for a pass.
