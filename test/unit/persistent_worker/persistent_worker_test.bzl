# Copyright 2024 The Bazel Authors. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Tests for persistent worker support in Rust rules."""

load("@bazel_skylib//lib:unittest.bzl", "analysistest", "asserts")
load("@bazel_skylib//rules:write_file.bzl", "write_file")
load("//rust:defs.bzl", "rust_library")
load("//rust:toolchain.bzl", "rust_stdlib_filegroup", "rust_toolchain")

def _worker_toolchain_attribute_test_impl(ctx):
    """Tests that the worker-enabled toolchain is configured correctly."""
    env = analysistest.begin(ctx)
    target = analysistest.target_under_test(env)

    actions = target[DepActionsInfo].actions
    rustc_actions = [act for act in actions if act.mnemonic == "Rustc"]
    asserts.true(
        env,
        len(rustc_actions) > 0,
        "Expected at least one Rustc action, got actions: {}".format([act.mnemonic for act in actions]),
    )

    return analysistest.end(env)

def _extra_worker_toolchain_transition_impl(settings, _attr):
    """Transition to add the worker-enabled toolchain."""
    return {"//command_line_option:extra_toolchains": [
        "@rules_rust//test/unit/persistent_worker:worker_toolchain",
    ] + settings["//command_line_option:extra_toolchains"]}

_extra_worker_toolchain_transition = transition(
    implementation = _extra_worker_toolchain_transition_impl,
    inputs = ["//command_line_option:extra_toolchains"],
    outputs = ["//command_line_option:extra_toolchains"],
)

DepActionsInfo = provider(
    "Contains information about dependencies' actions.",
    fields = {"actions": "List[Action]"},
)

def _collect_dep_actions_aspect_impl(target, ctx):
    actions = []
    actions.extend(target.actions)
    for dep in getattr(ctx.rule.attr, "deps", []):
        if DepActionsInfo in dep:
            actions.extend(dep[DepActionsInfo].actions)
    return [DepActionsInfo(actions = actions)]

collect_dep_actions_aspect = aspect(
    implementation = _collect_dep_actions_aspect_impl,
    attr_aspects = ["deps"],
)

def _worker_toolchain_wrapper_impl(ctx):
    return [ctx.attr.dep[DepActionsInfo]]

worker_toolchain_wrapper = rule(
    implementation = _worker_toolchain_wrapper_impl,
    attrs = {
        "dep": attr.label(aspects = [collect_dep_actions_aspect]),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
    cfg = _extra_worker_toolchain_transition,
)

worker_toolchain_attribute_test = analysistest.make(
    _worker_toolchain_attribute_test_impl,
)

def _define_targets():
    """Defines the test targets and toolchains."""
    rust_library(
        name = "test_lib",
        srcs = ["lib.rs"],
        edition = "2021",
    )

    native.filegroup(
        name = "stdlib_srcs",
        srcs = ["lib.rs"],
    )
    rust_stdlib_filegroup(
        name = "std_libs",
        srcs = [":stdlib_srcs"],
    )
    write_file(
        name = "mock_rustc",
        out = "mock_rustc.exe",
        content = [],
        is_executable = True,
    )
    write_file(
        name = "mock_rustdoc",
        out = "mock_rustdoc.exe",
        content = [],
        is_executable = True,
    )

    rust_toolchain(
        name = "rust_worker_toolchain",
        binary_ext = "",
        dylib_ext = ".so",
        exec_triple = "x86_64-unknown-none",
        target_triple = "x86_64-unknown-none",
        rust_doc = ":mock_rustdoc",
        rust_std = ":std_libs",
        rustc = ":mock_rustc",
        staticlib_ext = ".a",
        stdlib_linkflags = [],
        experimental_use_workers = True,
        visibility = ["//visibility:public"],
    )

    native.toolchain(
        name = "worker_toolchain",
        toolchain = ":rust_worker_toolchain",
        toolchain_type = "@rules_rust//rust:toolchain",
    )

    worker_toolchain_wrapper(
        name = "lib_with_worker_toolchain",
        dep = ":test_lib",
    )

def persistent_worker_test_suite(name):
    """Entry-point macro called from the BUILD file.

    Args:
        name: Name of the test suite.
    """
    _define_targets()

    worker_toolchain_attribute_test(
        name = "worker_toolchain_attribute_test",
        target_under_test = ":lib_with_worker_toolchain",
    )

    native.test_suite(
        name = name,
        tests = [
            ":worker_toolchain_attribute_test",
        ],
    )
