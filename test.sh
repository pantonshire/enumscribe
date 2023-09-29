#!/bin/bash
RUSTFLAGS='-Z randomize-layout' cargo +nightly miri test

