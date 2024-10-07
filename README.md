# Rust Axum File Upload and Download in Chunks

## Overview

This project demonstrates how to implement efficient file upload and download functionality in chunks using Rust with the Axum framework. It provides a RESTful API for handling large file operations, ensuring reduced memory consumption and improved user experience.

## Features

- **Chunked File Upload**: Upload large files in smaller segments to prevent memory overload.
- **Chunked File Download**: Download files in manageable pieces for optimized performance.
- **Progress Tracking**: Real-time progress updates during file uploads and downloads.

## Technologies Used

- **Rust**: A systems programming language focused on speed, memory safety, and parallelism.
- **Axum**: A web application framework for Rust that focuses on ergonomics and modularity.
- **Tokio**: An asynchronous runtime for Rust, enabling concurrent programming.

## Getting Started

### Prerequisites

- Install [Rust](https://www.rust-lang.org/tools/install) (using `rustup`).
- Install [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (comes bundled with Rust).
- Ensure you have `cargo install` dependencies for running the application.

### Clone the Repository

```bash
git clone https://github.com/AarambhDevHub/backend-file-Chunks.git
cd backend-file-Chunks
```

### Build the Project
Run the following command to build the project

```
cargo build
```

### Run the Server
Start the Axum server:

```
cargo run
```

The server will be available at http://localhost:8000.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.

## Donations

If you find this project useful and would like to support its continued development, you can make a donation via [Buy Me a Coffee](https://buymeacoffee.com/aarambhdevhub).

Thank you for your support!

