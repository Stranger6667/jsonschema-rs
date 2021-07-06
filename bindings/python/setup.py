from setuptools import setup

try:
    from setuptools_rust import Binding, RustExtension
except ImportError:
    from textwrap import dedent

    raise ImportError(
        dedent(
            """
            `setuptools-rust` is a required dependency to run `setup.py`.
            This should not happen if you're using `pip>=10` as it honors `pyproject.toml`.
            This usually (at least on our workflows) might happen while
            building source-distribution.
            """
        )
    )


def call_setup():
    setup(
        name="jsonschema_rs",
        version="0.11.1",
        description="Fast JSON Schema validation for Python implemented in Rust",
        long_description=open("README.rst", encoding="utf-8").read(),
        long_description_content_type="text/x-rst",
        keywords="jsonschema validation rust",
        author="Dmitry Dygalo",
        author_email="dadygalo@gmail.com",
        maintainer="Dmitry Dygalo",
        maintainer_email="dadygalo@gmail.com",
        python_requires=">=3.6",
        url="https://github.com/Stranger6667/jsonschema-rs/tree/master/python",
        license="MIT",
        rust_extensions=[RustExtension("jsonschema_rs", binding=Binding.PyO3)],
        classifiers=[
            "Development Status :: 3 - Alpha",
            "Intended Audience :: Developers",
            "License :: OSI Approved :: MIT License",
            "Operating System :: OS Independent",
            "Programming Language :: Python :: 3",
            "Programming Language :: Python :: 3.6",
            "Programming Language :: Python :: 3.7",
            "Programming Language :: Python :: 3.8",
            "Programming Language :: Python :: 3.9",
            "Programming Language :: Python :: Implementation :: CPython",
            "Programming Language :: Rust",
        ],
        zip_safe=False,
    )


if __name__ == "__main__":
    call_setup()
