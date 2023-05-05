# Forged Tooling

- `forged-rs` is a Rust client for Forged.dev operations.
- `forged-cli` is a CLI tool (written in Rust) to interract with the forged.dev tooling.

## Release a new version

### Rust

```
cd forged-rs
# Bump crate version (make sure API incompatibilities are handled properly)
cargo publish
```

# Python

```
# Install the tooling to upload to pip
python -m pip install build twine
```

```
cd forged-py
python -m build
twine check dist/*

# Upload to TestPyPi to check
twine upload -r testpypi dist/*

# Upload to rea PyPi to release
twine upload -r pypi dist/*
```
