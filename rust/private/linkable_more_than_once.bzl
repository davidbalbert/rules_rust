"""An aspect collecting the libraries tagged `LINKABLE_MORE_THAN_ONCE`.

A library tagged `LINKABLE_MORE_THAN_ONCE`, a crate or a native library, has no
global state, so two copies of it in one process only cost size. As for
`cc_shared_library`, a Rust shared library leaves it out of the
`link_once_static_libs` of its `CcSharedLibraryInfo`, so that rules_cc doesn't
fail when two shared libraries in one binary link it statically.

A rule can't read the tags of its dependencies, so, as rules_cc does with its
`graph_structure_aspect`, the Rust shared library rules apply this aspect to
their dependencies to find them.
"""

load("@rules_cc//cc/common:cc_info.bzl", "CcInfo")
load(":providers.bzl", "CrateGroupInfo", "CrateInfo")

LINKABLE_MORE_THAN_ONCE = "LINKABLE_MORE_THAN_ONCE"

LinkableMoreThanOnceInfo = provider(
    doc = "The libraries tagged `LINKABLE_MORE_THAN_ONCE` among a target and its dependencies.",
    fields = {
        "labels": "depset[str]: The labels of the tagged libraries, as strings.",
    },
)

# The attributes through which libraries are linked into a Rust shared library:
# a crate's `deps` and `link_deps`, and a `cc_library`'s `deps` and
# `implementation_deps`. Not `proc_macro_deps`, build script dependencies, or
# tools, which are never linked into it.
_LINKED_ATTRS = ["deps", "implementation_deps", "link_deps"]

def _linkable_more_than_once_aspect_impl(target, ctx):
    transitive = [
        dep[LinkableMoreThanOnceInfo].labels
        for attr_name in _LINKED_ATTRS
        for dep in getattr(ctx.rule.attr, attr_name, [])
        if LinkableMoreThanOnceInfo in dep
    ]
    direct = []
    if LINKABLE_MORE_THAN_ONCE in getattr(ctx.rule.attr, "tags", []):
        direct.append(str(target.label))
    return [LinkableMoreThanOnceInfo(labels = depset(direct, transitive = transitive))]

linkable_more_than_once_aspect = aspect(
    implementation = _linkable_more_than_once_aspect_impl,
    attr_aspects = _LINKED_ATTRS,
    required_providers = [[CcInfo], [CrateInfo], [CrateGroupInfo]],
    doc = "Collects the libraries tagged `LINKABLE_MORE_THAN_ONCE` among a target's linked dependencies.",
)
