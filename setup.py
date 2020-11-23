from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="r2048",
    version="1.0",
    rust_extensions=[RustExtension("r2048.r2048", binding=Binding.PyO3)],
    packages=["r2048"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
