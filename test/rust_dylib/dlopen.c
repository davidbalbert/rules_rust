// Loads the shared library given as the first argument with dlopen, the way
// Python loads extension modules, and fails if any of its dependencies can't
// be found.

#include <dlfcn.h>
#include <stdio.h>

int main(int argc, char** argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s <shared library>\n", argv[0]);
        return 1;
    }
    void* handle = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
    if (handle == NULL) {
        fprintf(stderr, "dlopen failed: %s\n", dlerror());
        return 1;
    }
    return 0;
}
