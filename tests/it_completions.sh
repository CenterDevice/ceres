#!/bin/bash

cd ..
cargo run -- completions --shell bash > /dev/null 2>&1 || (echo "Bash completions failed."; exit 1)
cargo run -- completions --shell fish > /dev/null 2>&1 || (echo "Fish completions failed."; exit 1)
cargo run -- completions --shell zsh > /dev/null 2>&1  || (echo "Zsh completions failed."; exit 1)

