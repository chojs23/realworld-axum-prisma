setup:
  cargo prisma migrate deploy

generate:
  cargo prisma generate

run:
  cargo run

watch:
  cargo watch -x run

build:
  cargo build --profile dev

release:
  cargo build --release

test:
  cargo test
