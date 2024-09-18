default:
  @just --list

fuzz TARGET:
  mkdir -p fuzz/corpus/{{TARGET}}
  cargo +nightly fuzz run --release {{TARGET}} fuzz/corpus/{{TARGET}} fuzz/seeds -- -dict=fuzz/dict

lint-rs:
  cargo +nightly fmt --all
  cargo clippy --all-features --all-targets
  cd fuzz && cargo +nightly fmt --all
  cd fuzz && cargo clippy --all-features --all-targets

lint-py:
  uvx ruff check crates/jsonschema-py/python crates/jsonschema-py/tests-py crates/jsonschema-py/benches
  uvx ruff check --select I --fix crates/jsonschema-py/python crates/jsonschema-py/tests-py crates/jsonschema-py/benches
  uvx mypy crates/jsonschema-py/python

lint: lint-rs lint-py

test-py *FLAGS:
  uvx --with="crates/jsonschema-py[tests]" pytest crates/jsonschema-py/tests-py -rs {{FLAGS}}

bench-py *FLAGS:
  uvx --with="crates/jsonschema-py[bench]" pytest crates/jsonschema-py/benches/bench.py --benchmark-columns=min {{FLAGS}}

