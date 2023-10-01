nightly := 'cargo +nightly'
rustc_nightly_flags := '-Z randomize-layout -Z macro-backtrace'

test:
    RUSTFLAGS='{{rustc_nightly_flags}}' {{nightly}} miri test

