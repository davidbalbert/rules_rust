load("@rules_cc//cc:defs.bzl", "cc_test")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_dylib_library", "rust_test")

# Integration tests for rust_dylib_library: verifies that a binary depending
# on a Rust ABI dylib can be built and run, exercising runfiles inclusion
# and RPATH resolution end-to-end.

NOT_WINDOWS = select({
    "@platforms//os:linux": [],
    "@platforms//os:macos": [],
    "//conditions:default": ["@platforms//:incompatible"],
})

rust_dylib_library(
    name = "rust_dylib_lib",
    srcs = ["lib.rs"],
    edition = "2021",
    link_std_dylib = True,
    visibility = ["//test:__subpackages__"],
)

rust_binary(
    name = "dylib_dep_binary",
    srcs = ["main.rs"],
    edition = "2021",
    link_std_dylib = True,
    target_compatible_with = NOT_WINDOWS,
    deps = [":rust_dylib_lib"],
)

rust_test(
    name = "dylib_dep_test",
    srcs = ["main.rs"],
    edition = "2021",
    link_std_dylib = True,
    target_compatible_with = NOT_WINDOWS,
    deps = [":rust_dylib_lib"],
)

# A Rust ABI dylib in an external repository, with a dylib dep, loaded with
# dlopen from this test's runfiles tree. Exercises the runfiles-relative RPATHs
# for outputs in external repositories, whose `_solib_<cpu>` deps live in the
# main repository.
cc_test(
    name = "external_dylib_dlopen_test",
    srcs = ["dlopen.c"],
    args = ["$(rootpath @rust_dylib_in_external_repo//:dylib_with_dylib_dep)"],
    data = ["@rust_dylib_in_external_repo//:dylib_with_dylib_dep"],
    linkopts = select({
        "@platforms//os:linux": ["-ldl"],
        "//conditions:default": [],
    }),
    target_compatible_with = NOT_WINDOWS,
)
