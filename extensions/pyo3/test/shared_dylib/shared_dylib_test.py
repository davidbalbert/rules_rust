"""Tests that extension modules share the Rust ABI dylibs they depend on."""

import unittest
from test.shared_dylib import ext_a, ext_b


class SharedDylibTest(unittest.TestCase):
    """Test Class."""

    def test_one_counter(self) -> None:
        """Both extensions increment the same counter."""

        self.assertEqual(
            [1, 2, 3], [ext_a.increment(), ext_b.increment(), ext_a.increment()]
        )

    def test_shared_pyclass(self) -> None:
        """A `Square` created by one extension is a `Square` in the other."""

        self.assertEqual(9, ext_b.area(ext_a.make_square(3)))


if __name__ == "__main__":
    unittest.main()
