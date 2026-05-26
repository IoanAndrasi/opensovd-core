<!--
SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
SPDX-License-Identifier: Apache-2.0
-->

# Bazel Notes

This repository now has a native Bazel build based on `rules_rust` and `crate_universe`.

## What Was Added

- A WORKSPACE-based Bazel bootstrap in [WORKSPACE](/home/ioan/opensovd-fork/opensovd-core/WORKSPACE).
- Workspace-wide Bazel defaults in [.bazelrc](/home/ioan/opensovd-fork/opensovd-core/.bazelrc).
- Root aliases and a root test suite in [BUILD.bazel](/home/ioan/opensovd-fork/opensovd-core/BUILD.bazel).
- Package-local `BUILD.bazel` files for the Rust crates and examples.
- A generated `cargo-bazel-lock.json` used by `crate_universe` for third-party Rust dependencies.
- A local rustup-backed toolchain bridge in [bazel/local_rustup_toolchain.bzl](/home/ioan/opensovd-fork/opensovd-core/bazel/local_rustup_toolchain.bzl).

## Important Repository Details

- Bazel is pinned to `8.3.0` through `.bazelversion`.
- The repository currently runs in WORKSPACE mode, not Bzlmod.
- The Rust toolchain is pinned to local rustup nightly `nightly-2026-05-07-x86_64-unknown-linux-gnu`.
- On this machine, Bazel needs the Java trust-store startup flags in [.bazelrc](/home/ioan/opensovd-fork/opensovd-core/.bazelrc) to fetch crates from `crates.io`.

## Validated Commands

The following command is validated and succeeds:

```bash
bazel build //opensovd-core:opensovd_core
```

Useful root aliases from [BUILD.bazel](/home/ioan/opensovd-fork/opensovd-core/BUILD.bazel):

```bash
bazel build //:opensovd-core
bazel build //:opensovd-gateway
bazel build //:opensovd-examples-client
bazel build //:opensovd-examples-server
bazel test //:tests
```

## When Dependencies Change

If you change Rust dependencies or add a new Cargo manifest that Bazel should understand, repin `crate_universe`:

```bash
CARGO_BAZEL_REPIN=1 bazel sync --only=crate_index
```

That updates the generated crate graph behind `cargo-bazel-lock.json`.

## How To Add A New Example

There are two supported patterns.

### Add A New Example Inside An Existing Example Package

Use this if the new example belongs under [examples/client](/home/ioan/opensovd-fork/opensovd-core/examples/client) or [examples/server](/home/ioan/opensovd-fork/opensovd-core/examples/server).

For a new server example, do the following:

1. Add the Rust source file, for example `examples/server/my_feature/my_feature.rs`.
2. Register it in [examples/server/Cargo.toml](/home/ioan/opensovd-fork/opensovd-core/examples/server/Cargo.toml) as a new `[[example]]` entry.
3. Add a matching `rust_binary` target in [examples/server/BUILD.bazel](/home/ioan/opensovd-fork/opensovd-core/examples/server/BUILD.bazel).
4. If the example needs new crates, update the Cargo manifest and then run `CARGO_BAZEL_REPIN=1 bazel sync --only=crate_index`.
5. Build it with `bazel build //examples/server:my_feature_example`.

Minimal pattern for [examples/server/BUILD.bazel](/home/ioan/opensovd-fork/opensovd-core/examples/server/BUILD.bazel):

```starlark
rust_binary(
    name = "my_feature_example",
    srcs = ["my_feature/my_feature.rs"],
    aliases = aliases(normal = True, proc_macro = True),
    crate_features = ["tls"],  # only if your example needs that feature
    crate_name = "my_feature",
    crate_root = "my_feature/my_feature.rs",
    edition = "2024",
    proc_macro_deps = all_crate_deps(proc_macro = True),
    version = "0.1.1",
    deps = COMMON_DEPS,
)
```

If you want the new example to be included in the root `//:opensovd-examples-server` alias, also add it to the `server_examples` filegroup in [examples/server/BUILD.bazel](/home/ioan/opensovd-fork/opensovd-core/examples/server/BUILD.bazel).

### Add A New Example As A New Cargo Package Under `examples/`

Use this if you want a separate package such as `examples/foo`.

1. Create `examples/foo/Cargo.toml` and the Rust sources.
2. Add `examples/foo` to the workspace members in [Cargo.toml](/home/ioan/opensovd-fork/opensovd-core/Cargo.toml).
3. Create `examples/foo/BUILD.bazel` with the needed `rust_binary` or `rust_library` targets.
4. Add `//examples/foo:Cargo.toml` to the `manifests` list in [WORKSPACE](/home/ioan/opensovd-fork/opensovd-core/WORKSPACE).
5. Optionally add a root alias in [BUILD.bazel](/home/ioan/opensovd-fork/opensovd-core/BUILD.bazel) if you want a stable entrypoint such as `//:opensovd-examples-foo`.
6. Run `CARGO_BAZEL_REPIN=1 bazel sync --only=crate_index`.
7. Build it with `bazel build //examples/foo:<target-name>`.

Example root alias pattern from [BUILD.bazel](/home/ioan/opensovd-fork/opensovd-core/BUILD.bazel):

```starlark
alias(
    name = "opensovd-examples-foo",
    actual = "//examples/foo:foo_example",
)
```

## Practical Checklist For New Examples

When Bazel does not see a new example yet, check these in order:

1. The example source exists and the path in `crate_root` matches.
2. The example is declared in the package `Cargo.toml`.
3. The package has a matching Bazel target in its `BUILD.bazel`.
4. If it is a new package, the root [Cargo.toml](/home/ioan/opensovd-fork/opensovd-core/Cargo.toml) and [WORKSPACE](/home/ioan/opensovd-fork/opensovd-core/WORKSPACE) both reference it.
5. If dependencies changed, `crate_universe` has been repinned.

## Recommended Validation

For an example under an existing package:

```bash
bazel build //examples/server:my_feature_example
```

For a brand-new example package:

```bash
CARGO_BAZEL_REPIN=1 bazel sync --only=crate_index
bazel build //examples/foo:foo_example
```