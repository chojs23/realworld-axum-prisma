![Realworld Rust](logo.png)

# Realworld Rust Axum Prisma

This project utilizes Rust with the Axum v0.7 framework along with the Prisma Client Rust to build a realworld application. For Prisma Client Rust ORM, refer to [Prisma](https://github.com/prisma/prisma) and [Prisma Client Rust Repository](https://github.com/Brendonovich/prisma-client-rust) for more information.

## Prerequisites

Make sure you have the following installed:

- Rust
- Docker
- MySQL

## Getting Started

### Installation

1. Clone this repository.
2. Set up your environment variables by creating a `.env` file in the root directory. Use the provided examples in the README as a guide.
3. Ensure your MySQL server is running.
4. Apply migrations
5. Generate Prisma Client

### Running the Application

You can use the provided `justfile` for various commands:

- `just setup`: Apply migrations using `prisma-cli`.
- `just generate`: Generate the Prisma Client using `prisma-cli`.
- `just run`: Run the application using `cargo run`.
- `just watch`: Use `cargo watch` to automatically reload the application on file changes.
- `just build`: Build the application using the development profile.
- `just release`: Build the application for release.
- `just test`: Run tests using `cargo test`.

### Docker Setup

If you prefer Docker, follow these steps:

1. Build the Docker image:

   The Dockerfile includes steps to build the project within a Docker container. Particularly, the `cargo prisma generate` command is used to generate the Prisma Client during the Docker image build process. This command parses the Prisma schema and creates a client tailored to the defined database structure.

   ```bash
   docker-compose build
   ```

2. Start the containers:

   ```bash
   docker-compose up
   ```

## Environment Variables

Ensure your environment variables are appropriately set. Here are some examples:

- `DATABASE_URL`: MySQL database URL.
- `PORT`: Port on which the application runs.
- `RUST_LOG`: Rust logging level.
- `JWT_SECRET`: Secret key for JWT authentication.
- `MYSQL_ROOT_PASSWORD`: If using docker-compose make sure to set MySQL root password.

## ⚠️ Important Note

This project cannot be built in release mode using Rust stable toolchain version 1.7.4 (`stable-aarch64-apple-darwin`) due to a compilation error with the `psl-core` library. However, it can be successfully built in debug mode using this toolchain.

## Contributing

Contributions are welcome! Feel free to open issues or pull requests.

## License

This project is licensed under the [MIT License](LICENSE).
