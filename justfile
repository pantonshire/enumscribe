nightly := 'cargo +nightly'
rustc_nightly_flags := '-Z randomize-layout'

test:
    RUST_FLAGS='{{rustc_nightly_flags}}' {{nightly}} miri test

