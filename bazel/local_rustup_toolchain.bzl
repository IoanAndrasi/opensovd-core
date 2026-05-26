# SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
# SPDX-License-Identifier: Apache-2.0

load("@rules_rust//rust/platform:triple.bzl", "triple")
load("@rules_rust//rust/platform:triple_mappings.bzl", "triple_to_constraint_set")
load("@rules_rust//rust/private:repository_utils.bzl", "BUILD_for_cargo", "BUILD_for_clippy", "BUILD_for_compiler", "BUILD_for_llvm_tools", "BUILD_for_rust_toolchain", "BUILD_for_stdlib")
load("@rules_rust//rust:repositories.bzl", "toolchain_repository_proxy")

def _local_rustup_toolchain_tools_impl(repository_ctx):
    home = repository_ctx.os.environ.get("HOME")
    if not home:
        fail("HOME must be set to locate the local rustup toolchain")

    toolchain_root = repository_ctx.path(home + "/.rustup/toolchains/" + repository_ctx.attr.rustup_toolchain)
    if not toolchain_root.exists:
        fail("Rustup toolchain not found: {}".format(toolchain_root))

    exec_triple = triple(repository_ctx.attr.exec_triple)
    target_triple = triple(repository_ctx.attr.target_triple)

    for path in ["bin", "lib", "libexec", "share"]:
        source = toolchain_root.get_child(path)
        if source.exists:
            repository_ctx.symlink(source, path)

    build_components = [
        BUILD_for_compiler(exec_triple, include_linker = True, include_objcopy = True),
        BUILD_for_cargo(exec_triple),
        BUILD_for_clippy(exec_triple),
        BUILD_for_llvm_tools(exec_triple),
        BUILD_for_stdlib(target_triple),
        BUILD_for_rust_toolchain(
            name = "rust_toolchain",
            exec_triple = exec_triple,
            target_triple = target_triple,
            version = repository_ctx.attr.version,
            channel = repository_ctx.attr.channel,
            iso_date = repository_ctx.attr.iso_date,
            default_edition = repository_ctx.attr.edition,
            include_llvm_tools = True,
            include_linker = True,
            include_objcopy = True,
        ),
    ]

    repository_ctx.file("WORKSPACE.bazel", "workspace(name = \"{}\")\n".format(repository_ctx.name))
    repository_ctx.file("BUILD.bazel", "\n".join(build_components))
    repository_ctx.file(repository_ctx.name, "")

local_rustup_toolchain_tools_repository = repository_rule(
    implementation = _local_rustup_toolchain_tools_impl,
    local = True,
    attrs = {
        "channel": attr.string(mandatory = True),
        "edition": attr.string(mandatory = True),
        "exec_triple": attr.string(mandatory = True),
        "iso_date": attr.string(),
        "rustup_toolchain": attr.string(mandatory = True),
        "target_triple": attr.string(mandatory = True),
        "version": attr.string(mandatory = True),
    },
)

def local_rustup_toolchain(name, rustup_toolchain, exec_triple, target_triple, version, channel, iso_date, edition):
    tools_name = name + "_tools"
    local_rustup_toolchain_tools_repository(
        name = tools_name,
        channel = channel,
        edition = edition,
        exec_triple = exec_triple,
        iso_date = iso_date,
        rustup_toolchain = rustup_toolchain,
        target_triple = target_triple,
        version = version,
    )
    toolchain_repository_proxy(
        name = name,
        exec_compatible_with = triple_to_constraint_set(exec_triple),
        target_compatible_with = triple_to_constraint_set(target_triple),
        target_settings = ["@rules_rust//rust/toolchain/channel:{}".format(channel)],
        toolchain = "@{}//:rust_toolchain".format(tools_name),
        toolchain_type = "@rules_rust//rust:toolchain",
    )