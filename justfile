nightly := 'cargo +nightly'
rustc_nightly_flags := '-Z randomize-layout'

test:
    RUSTFLAGS='{{rustc_nightly_flags}}' {{nightly}} miri test

