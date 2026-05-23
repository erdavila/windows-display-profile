# Runs checks, tests, formatting, and clippy on all packages
all: check test fmt clippy


check:
  cargo check --workspace \
    --exclude display-profile-lib \
    --exclude display-profile-cli
  @# Checks the different combinations of features:
  cargo check -p windows-display-config --no-default-features
  cargo check -p windows-display-config --no-default-features --features dump
  cargo check -p display-profile-lib --no-default-features
  cargo check -p display-profile-lib --no-default-features --features serde
  cargo check -p display-profile-lib --no-default-features --features dump
  cargo check -p display-profile-cli --no-default-features
  cargo check -p display-profile-cli --no-default-features --features dump


# Tests all packages
test:
  cargo test --workspace


# Formats all packages
fmt:
  cargo +nightly fmt --all -- --config group_imports=StdExternalCrate --config imports_granularity=Module


# Runs clippy on all packages
clippy:
  cargo clippy --all-targets --all-features --workspace


doc *ARGS:
  cargo +nightly doc --all-features --no-deps \
    -p windows-display-config \
    -p display-profile-lib \
    {{ARGS}}

doc-open: (doc '--open')


# Runs the display-profile CLI
[group('binary execution')]
profile ACTION FILE: (_profile '""' ACTION FILE)

# Runs the display-profile CLI with the "dump" feature
[group('binary execution')]
profile-dump ACTION FILE: (_profile 'dump' ACTION FILE)

_profile FEATURES ACTION FILE:
  cargo run -q -p display-profile-cli --features {{FEATURES}} -- {{ACTION}} {{FILE}}
