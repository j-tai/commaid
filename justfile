[private]
help:
    @just --list

# Create a production build.
build:
    cd frontend && yarn install
    cd frontend && yarn build
    cd backend && cargo build --release
