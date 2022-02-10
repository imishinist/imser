#!/bin/bash

echo "+cargo build ===================="
cargo build
echo "================================="

echo "+cargo test ====================="
cargo test
echo "================================="

echo "+cargo check ===================="
cargo check
echo "================================="

echo "+cargo clippy -- -D warnings ===="
cargo clippy -- -D warnings
echo "================================="

echo "+cargo fmt -- --check ==========="
cargo fmt -- --check
echo "================================="

assert() {
    expected=$1
    term=$2
    shift; shift;
    sentences=$@

    actual=$(./target/debug/imser "$term" "$sentences" 2>&1)

    if [ "$expected" = "$actual" ]; then
        echo "search \"$term\" from \"$sentences\" => $actual"
    else
        echo "search \"$term\" from \"$sentences\" => $expected, but got \"$actual\""
        exit 1
    fi
}

assert "0" "Taisuke" "I am Taisuke" 
assert "0" "that" "that that is is that that is not is not is that it it is"
assert "term not found: foo" "foo" "This is a pen"

echo OK
