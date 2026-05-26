# SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
# SPDX-License-Identifier: Apache-2.0

def _cc_compatibility_proxy_impl(repository_ctx):
    repository_ctx.file(
        "BUILD",
        """
load("@bazel_skylib//:bzl_library.bzl", "bzl_library")
bzl_library(
  name = "proxy_bzl",
  srcs = ["proxy.bzl"],
  visibility = ["@rules_cc//cc:__subpackages__"],
)
bzl_library(
  name = "symbols_bzl",
  srcs = ["symbols.bzl"],
  deps = [
      "@rules_cc//cc/private/rules_impl:native_cc_common_bzl",
      "@rules_cc//cc/private/rules_impl:native_providers_bzl",
  ],
  visibility = ["@rules_cc//cc:__subpackages__"],
)
""",
    )
    repository_ctx.file(
        "proxy.bzl",
        """
cc_binary = native.cc_binary
cc_import = native.cc_import
cc_library = native.cc_library
cc_shared_library = native.cc_shared_library
cc_static_library = getattr(native, "cc_static_library", None)
cc_test = native.cc_test
objc_import = native.objc_import
objc_library = native.objc_library
fdo_prefetch_hints = native.fdo_prefetch_hints
fdo_profile = native.fdo_profile
memprof_profile = getattr(native, "memprof_profile", None)
propeller_optimize = native.propeller_optimize
cc_toolchain = native.cc_toolchain
cc_toolchain_alias = native.cc_toolchain_alias
""",
    )
    repository_ctx.file(
        "symbols.bzl",
        """
load("@rules_cc//cc/private/rules_impl:native_cc_common.bzl", "native_cc_common")
load("@rules_cc//cc/private/rules_impl:native_providers.bzl", "NativeCcInfo")
load("@rules_cc//cc/private/rules_impl:native_providers.bzl", "NativeDebugPackageInfo")
load("@rules_cc//cc/private/rules_impl:native_providers.bzl", "NativeCcToolchainConfigInfo")
load("@rules_cc//cc/private/rules_impl:native_providers.bzl", "NativeCcSharedLibraryInfo")
cc_common = native_cc_common
CcInfo = NativeCcInfo
merge_cc_infos = cc_common.merge_cc_infos
DebugPackageInfo = NativeDebugPackageInfo
CcToolchainConfigInfo = NativeCcToolchainConfigInfo
ObjcInfo = apple_common.Objc
new_objc_provider = apple_common.new_objc_provider
CcSharedLibraryInfo = NativeCcSharedLibraryInfo
""",
    )

cc_compatibility_proxy_repository = repository_rule(
    implementation = _cc_compatibility_proxy_impl,
    local = True,
)

def cc_compatibility_proxy_repo():
    cc_compatibility_proxy_repository(name = "cc_compatibility_proxy")