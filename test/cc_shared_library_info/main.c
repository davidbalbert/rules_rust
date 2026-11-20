// Calls two Rust shared libraries that share a crate through a Rust ABI dylib.

unsigned int cdylib_a_value(void);
unsigned int cdylib_b_value(void);

int main(void) { return cdylib_a_value() + cdylib_b_value() == 2 ? 0 : 1; }
