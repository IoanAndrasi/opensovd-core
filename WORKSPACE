# SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
# SPDX-License-Identifier: Apache-2.0

workspace(name = "opensovd_core")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("//:bazel/cc_compatibility_proxy.bzl", "cc_compatibility_proxy_repo")

cc_compatibility_proxy_repo()

http_archive(
	name = "bazel_skylib",
	sha256 = "e3fea03ff75a9821e84199466799ba560dbaebb299c655b5307f4df1e5970696",
	strip_prefix = "bazel-skylib-1.7.1",
	urls = ["https://github.com/bazelbuild/bazel-skylib/archive/refs/tags/1.7.1.tar.gz"],
)

http_archive(
	name = "platforms",
	sha256 = "324f5381753a610e472f79563d44e2026438195042aae4dc660b8c021f7de7f5",
	strip_prefix = "platforms-1.1.0",
	urls = ["https://github.com/bazelbuild/platforms/archive/refs/tags/1.1.0.tar.gz"],
)

http_archive(
	name = "rules_cc",
	sha256 = "92fed78a5a310f86c060dcaed20f396ef0198cc3d46a46fdea7c469042cf02ce",
	strip_prefix = "rules_cc-0.2.1",
	urls = ["https://github.com/bazelbuild/rules_cc/archive/refs/tags/0.2.1.tar.gz"],
)

http_archive(
	name = "rules_rust",
	sha256 = "44cb81880665f425ecccaa6d06e511db1dcc03124b6c4c541955149e721679db",
	strip_prefix = "rules_rust-0.70.0",
	urls = ["https://github.com/bazelbuild/rules_rust/archive/refs/tags/0.70.0.tar.gz"],
)

http_archive(
	name = "bazel_features",
	sha256 = "6a727a78c0134b1b912c97c0937e1c956f35775934ae3e1f4af4156f8d5d1ff4",
	strip_prefix = "bazel_features-1.47.1",
	urls = ["https://github.com/bazel-contrib/bazel_features/releases/download/v1.47.1/bazel_features-v1.47.1.tar.gz"],
)

load("//:bazel/local_rustup_toolchain.bzl", "local_rustup_toolchain")
load("@rules_cc//cc:repositories.bzl", "rules_cc_dependencies", "rules_cc_toolchains")
load("@rules_rust//crate_universe:defs.bzl", "crates_repository")
load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies")

rules_cc_dependencies()
rules_cc_toolchains()
rules_rust_dependencies()

local_rustup_toolchain(
	name = "rust_linux_x86_64__x86_64-unknown-linux-gnu__nightly",
	channel = "nightly",
	edition = "2024",
	exec_triple = "x86_64-unknown-linux-gnu",
	iso_date = "2026-05-07",
	rustup_toolchain = "nightly-2026-05-07-x86_64-unknown-linux-gnu",
	target_triple = "x86_64-unknown-linux-gnu",
	version = "1.97.0",
)

register_toolchains("@rust_linux_x86_64__x86_64-unknown-linux-gnu__nightly//:toolchain")

crates_repository(
	name = "crate_index",
	cargo_lockfile = "//:Cargo.lock",
	generator_sha256s = {
		"x86_64-unknown-linux-gnu": "56bb13f6e0320ebd6e3e3bd340c159877b9f8b28294fca341752239b45073f86",
	},
	generator_urls = {
		"x86_64-unknown-linux-gnu": "https://github.com/bazelbuild/rules_rust/releases/download/0.70.0/cargo-bazel-x86_64-unknown-linux-gnu",
	},
	lockfile = "//:cargo-bazel-lock.json",
	manifests = [
		"//:Cargo.toml",
		"//benches:Cargo.toml",
		"//examples/client:Cargo.toml",
		"//examples/server:Cargo.toml",
		"//opensovd-cli/gateway:Cargo.toml",
		"//opensovd-cli/lib:Cargo.toml",
		"//opensovd-cli/mcp:Cargo.toml",
		"//opensovd-client:Cargo.toml",
		"//opensovd-core:Cargo.toml",
		"//opensovd-extra:Cargo.toml",
		"//opensovd-mocks:Cargo.toml",
		"//opensovd-models:Cargo.toml",
		"//opensovd-providers:Cargo.toml",
		"//opensovd-server:Cargo.toml",
	],
	rust_version = "nightly/2026-05-07",
)

load("@crate_index//:defs.bzl", "crate_repositories")
load("@rules_rust//cargo/3rdparty/crates:defs.bzl", rules_rust_crate_repositories = "crate_repositories")

rules_rust_crate_repositories()
crate_repositories()