"""Analysis tests for the `CcSharedLibraryInfo` of Rust shared libraries."""

load("@rules_cc//cc/common:cc_shared_library_info.bzl", "CcSharedLibraryInfo")
load("@rules_testing//lib:analysis_test.bzl", "analysis_test", "test_suite")
load("@rules_testing//lib:truth.bzl", "matching")

def _labels(*names):
    return [str(Label(":" + name)) for name in names]

def _owners(dynamic_deps):
    return [str(entry.linker_input.owner) for entry in dynamic_deps.to_list()]

def _expect_info(env, target, *, exports, link_once_static_libs, dynamic_deps):
    info = target[CcSharedLibraryInfo]
    env.expect.that_collection(info.exports).contains_exactly(exports)
    env.expect.that_collection(info.link_once_static_libs).contains_exactly(link_once_static_libs)
    env.expect.that_collection(_owners(info.dynamic_deps)).contains_exactly(dynamic_deps)
    env.expect.that_str(str(info.linker_input.owner)).equals(str(target.label))
    env.expect.that_collection(info.linker_input.libraries).has_size(1)

def _test_cdylib_impl(env, target):
    _expect_info(
        env,
        target,
        exports = [],
        link_once_static_libs = _labels("cdylib", "shared_crate", "c_lib"),
        dynamic_deps = [],
    )

def _test_cdylib(name):
    analysis_test(name = name, impl = _test_cdylib_impl, target = ":cdylib")

def _test_cdylib_with_crate_through_c_lib_impl(env, target):
    _expect_info(
        env,
        target,
        exports = [],
        link_once_static_libs = _labels("cdylib_with_crate_through_c_lib", "c_lib_with_crate_dep", "shared_crate"),
        dynamic_deps = [],
    )

def _test_cdylib_with_crate_through_c_lib(name):
    analysis_test(name = name, impl = _test_cdylib_with_crate_through_c_lib_impl, target = ":cdylib_with_crate_through_c_lib")

def _test_dylib_impl(env, target):
    _expect_info(
        env,
        target,
        exports = _labels("dylib"),
        link_once_static_libs = _labels("dylib"),
        dynamic_deps = [],
    )

def _test_dylib(name):
    analysis_test(name = name, impl = _test_dylib_impl, target = ":dylib")

def _test_dylib_with_deps_impl(env, target):
    # Native libraries are linked statically, but not exported: a dylib only
    # exports the symbols of its crates.
    _expect_info(
        env,
        target,
        exports = _labels("dylib_with_deps", "shared_crate"),
        link_once_static_libs = _labels("dylib_with_deps", "shared_crate", "c_lib"),
        dynamic_deps = [],
    )

def _test_dylib_with_deps(name):
    analysis_test(name = name, impl = _test_dylib_with_deps_impl, target = ":dylib_with_deps")

def _expect_links_dylib_with_deps(env, target):
    # The crates and native libraries in the dylib are linked through it.
    _expect_info(
        env,
        target,
        exports = [],
        link_once_static_libs = _labels(target.label.name),
        dynamic_deps = _labels("dylib_with_deps"),
    )

def _test_cdylib_with_dylib_dep_impl(env, target):
    _expect_links_dylib_with_deps(env, target)

def _test_cdylib_with_dylib_dep(name):
    analysis_test(name = name, impl = _test_cdylib_with_dylib_dep_impl, target = ":cdylib_with_dylib_dep")

def _test_cdylib_with_dylib_through_rlib_impl(env, target):
    _expect_info(
        env,
        target,
        exports = [],
        link_once_static_libs = _labels("cdylib_with_dylib_through_rlib", "dylib_rlib"),
        dynamic_deps = _labels("dylib_with_deps"),
    )

def _test_cdylib_with_dylib_through_rlib(name):
    analysis_test(name = name, impl = _test_cdylib_with_dylib_through_rlib_impl, target = ":cdylib_with_dylib_through_rlib")

def _test_cdylib_with_dylib_diamond_impl(env, target):
    # The dylib is reached twice, but listed once.
    _expect_info(
        env,
        target,
        exports = [],
        link_once_static_libs = _labels("cdylib_with_dylib_diamond", "dylib_rlib"),
        dynamic_deps = _labels("dylib_with_deps"),
    )

def _test_cdylib_with_dylib_diamond(name):
    analysis_test(name = name, impl = _test_cdylib_with_dylib_diamond_impl, target = ":cdylib_with_dylib_diamond")

def _test_dylib_with_dylib_dep_impl(env, target):
    _expect_info(
        env,
        target,
        exports = _labels("dylib_with_dylib_dep"),
        link_once_static_libs = _labels("dylib_with_dylib_dep"),
        dynamic_deps = _labels("dylib_with_deps"),
    )

def _test_dylib_with_dylib_dep(name):
    analysis_test(name = name, impl = _test_dylib_with_dylib_dep_impl, target = ":dylib_with_dylib_dep")

def _test_cdylib_with_dylib_chain_impl(env, target):
    # The crates in the dylib's own dylib dependency are linked through that.
    _expect_info(
        env,
        target,
        exports = [],
        link_once_static_libs = _labels("cdylib_with_dylib_chain"),
        dynamic_deps = _labels("dylib_with_dylib_dep", "dylib_with_deps"),
    )

def _test_cdylib_with_dylib_chain(name):
    analysis_test(name = name, impl = _test_cdylib_with_dylib_chain_impl, target = ":cdylib_with_dylib_chain")

def _test_duplicate_crate_fails_impl(env, target):
    env.expect.that_target(target).failures().contains_predicate(
        matching.contains("link the same  library statically"),
    )

def _test_duplicate_crate_fails(name):
    analysis_test(
        name = name,
        impl = _test_duplicate_crate_fails_impl,
        target = ":binary_with_duplicate_crate",
        expect_failure = True,
    )

def cc_shared_library_info_test_suite(name):
    """Analysis tests for the `CcSharedLibraryInfo` of Rust shared libraries.

    Args:
        name (str): The name of the test suite.
    """
    test_suite(
        name = name,
        tests = [
            _test_cdylib,
            _test_cdylib_with_crate_through_c_lib,
            _test_dylib,
            _test_dylib_with_deps,
            _test_cdylib_with_dylib_dep,
            _test_cdylib_with_dylib_through_rlib,
            _test_cdylib_with_dylib_diamond,
            _test_dylib_with_dylib_dep,
            _test_cdylib_with_dylib_chain,
            _test_duplicate_crate_fails,
        ],
    )
