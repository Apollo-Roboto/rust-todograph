set windows-shell := ["powershell.exe", "-c"]

[private]
default:
    @just --list --unsorted

build:
    cargo build --workspace

test:
    cargo test --workspace

gui:
    cargo run --bin tg-gui-app

cli PATH:
    cargo run --bin tg-cli-app -- {{ PATH }}
