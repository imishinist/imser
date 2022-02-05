#!/bin/bash

cargo build
cargo test

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

assert "[5]" "Taisuke" "I am Taisuke" 
assert "[0, 5, 16, 21, 43]" "that" "that that is is that that is not is not is that it it is"
assert "term not found: foo" "foo" "This is a pen"

echo OK
