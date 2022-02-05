#!/bin/bash

cargo build
cargo test

assert() {
    expected=$1
    arg1=$2
    arg2=$3

    actual=$(./target/debug/imser "$arg1" "$arg2" 2>&1)

    if [ "$expected" = "$actual" ]; then
        echo "search \"$arg2\" from \"$arg1\" => $actual"
    else
        echo "search \"$arg2\" from \"$arg1\" => $expected, but got \"$actual\""
        exit 1
    fi
}

assert "[5]" "I am Taisuke" "Taisuke"
assert "[0, 5, 16, 21, 43]" "that that is is that that is not is not is that it it is" "that"
assert "term not found: foo" "This is a pen" "foo"

echo OK
