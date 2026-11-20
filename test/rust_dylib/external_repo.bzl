"""External repository for `rust_dylib` tests"""

_BUILD_FILE_CONTENT = """
load("@rules_rust//rust:defs.bzl", "rust_dylib_library")

rust_dylib_library(
    name = "dylib_dep",
    srcs = ["dylib_dep.rs"],
    edition = "2021",
)

# Loaded with dlopen from a test in the main repository, the way Python loads
# extension modules, so it must find `dylib_dep` through its rpaths when it is
# in that test's runfiles tree.
rust_dylib_library(
    name = "dylib_with_dylib_dep",
    srcs = ["dylib_with_dylib_dep.rs"],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = [":dylib_dep"],
)
"""

_DYLIB_DEP_RS_CONTENT = """
pub fn one() -> u32 {
    1
}
"""

_DYLIB_WITH_DYLIB_DEP_RS_CONTENT = """
pub fn two() -> u32 {
    dylib_dep::one() + 1
}
"""

def _rust_dylib_in_external_repo_impl(repository_ctx):
    repository_ctx.file("BUILD.bazel", content = _BUILD_FILE_CONTENT)
    repository_ctx.file("dylib_dep.rs", content = _DYLIB_DEP_RS_CONTENT)
    repository_ctx.file("dylib_with_dylib_dep.rs", content = _DYLIB_WITH_DYLIB_DEP_RS_CONTENT)
    if hasattr(repository_ctx, "repo_metadata"):
        return repository_ctx.repo_metadata(reproducible = True)
    return None

_rust_dylib_in_external_repo = repository_rule(
    implementation = _rust_dylib_in_external_repo_impl,
    doc = "A test repository providing a Rust ABI dylib that depends on another one.",
)

def rust_dylib_in_external_repo():
    """Define a test repository with Rust ABI dylibs.

    Returns:
        list[struct(repo=str, is_dev_dep=bool)]: The repositories defined.
    """
    _rust_dylib_in_external_repo(
        name = "rust_dylib_in_external_repo",
    )
    return [struct(repo = "rust_dylib_in_external_repo", is_dev_dep = True)]
