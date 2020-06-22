import os

from setuptools import find_packages, setup

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
        version="0.3.3",
        description="Fast JSON Schema validation for Python implemented in Rust",
        long_description=open("README.rst", encoding="utf-8").read(),
        long_description_content_type="text/x-rst",
        keywords="jsonschema validation rust",
        author="Dmitry Dygalo",
        author_email="dadygalo@gmail.com",
        maintainer="Dmitry Dygalo",
        maintainer_email="dadygalo@gmail.com",
        python_requires=">=3.5",
        url="https://github.com/Stranger6667/jsonschema-rs/tree/master/python",
        license="MIT",
        rust_extensions=[RustExtension("jsonschema_rs.jsonschema_rs", binding=Binding.PyO3)],
        classifiers=[
            "Development Status :: 3 - Alpha",
            "Intended Audience :: Developers",
            "License :: OSI Approved :: MIT License",
            "Operating System :: OS Independent",
            "Programming Language :: Python :: 3",
            "Programming Language :: Python :: 3.5",
            "Programming Language :: Python :: 3.6",
            "Programming Language :: Python :: 3.7",
            "Programming Language :: Python :: 3.8",
            "Programming Language :: Python :: Implementation :: CPython",
            "Programming Language :: Rust",
        ],
        packages=find_packages(where="pysrc"),
        package_dir={"": "pysrc"},
        zip_safe=False,
    )


if "UNRELEASED_JSONSCHEMA_PATH" in os.environ:
    import sys

    from contextlib import contextmanager
    from tempfile import NamedTemporaryFile
    import toml

    # Modify Cargo.toml to apply crates.io patch to link jsonschema to the version available
    # on the repository.
    # This is done by using a contextmanager in order to ensure that the Cargo.toml modification
    # is discarded once setup.py ends (regardless of success or failure)
    @contextmanager
    def cargo_toml_context():
        tmp_file = NamedTemporaryFile(buffering=False)
        with open("Cargo.toml", "rb") as f:
            tmp_file.writelines(f.readlines())

        cargo_file = toml.load("Cargo.toml")

        cargo_file.setdefault("patch", {}).setdefault("crates-io", {})["jsonschema"] = {
            "path": os.environ["UNRELEASED_JSONSCHEMA_PATH"],
        }

        with open("Cargo.toml", "w") as f:
            toml.dump(cargo_file, f)

        try:
            print(
                "Modified Cargo.toml file by patching jsonschema dependency to {}".format(
                    os.environ["UNRELEASED_JSONSCHEMA_PATH"]
                ),
                file=sys.stderr,
            )
            yield
        except:
            print("Cargo.toml used during the build", file=sys.stderr)
            with open("Cargo.toml", "r") as f:
                print(f.read(), file=sys.stderr)

            raise
        finally:
            with open("Cargo.toml", "wb") as f:
                tmp_file.seek(0)
                f.writelines(tmp_file.readlines())

    with cargo_toml_context():
        call_setup()
else:
    call_setup()
