use axum::{
    body::Body, extract::{Multipart, Query}, http::{HeaderValue, Method, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Router
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
};

#[tokio::main]
async fn main() {
    // Initialize uploads directory
    fs::create_dir_all("./uploads/temp").unwrap();

    let cors = CorsLayer::new()
    .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST]);

    // Build our app
    let app = Router::new()
        .route("/upload", post(upload_chunk))
        .route("/download", get(download_chunk))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 8000)).await.unwrap();
    println!("Server running on http://localhost:8000");
    axum::serve(listener, app).await.unwrap();
}

// Upload handler
pub async fn upload_chunk(mut multipart: Multipart) -> impl IntoResponse {
    // Variables to hold form data
    let mut file_name = String::new();
    let mut chunk_number = 0;
    let mut total_chunks = 0;
    let mut chunk_data = Vec::new();

    // Process multipart form data
    while let Some(field) = match multipart.next_field().await {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error reading multipart field: {:?}", err);
            return StatusCode::BAD_REQUEST;
        }
    } {
        let field_name = field.name().unwrap_or_default().to_string();
        match field_name.as_str() {
            "fileName" => {
                file_name = field.text().await.unwrap_or_default();
                file_name = sanitize_filename(&file_name);
            }
            "chunkNumber" => {
                chunk_number = field.text().await.unwrap_or_default().parse().unwrap_or(0);
            }
            "totalChunks" => {
                total_chunks = field.text().await.unwrap_or_default().parse().unwrap_or(0);
            }
           "chunk" => {
                match field.bytes().await {
                    Ok(bytes) => chunk_data = bytes.to_vec(), // Convert Bytes to Vec<u8>
                    Err(err) => {
                        eprintln!("Error reading chunk data: {:?}", err);
                        return StatusCode::BAD_REQUEST; // Return BAD_REQUEST on error
                    }
                }
            }
            _ => {}
        }
    }

    // Validate that required fields are provided
    if file_name.is_empty() || chunk_data.is_empty() {
        eprintln!("Missing required fields: file_name: {}, chunk_data: {:?}", file_name, chunk_data);
        return StatusCode::BAD_REQUEST;
    }

    // Create a temporary directory to store the file chunks
    let temp_dir = format!("./uploads/temp/{}", file_name);
    if let Err(err) = fs::create_dir_all(&temp_dir) {
        eprintln!("Failed to create temp directory: {:?}, Error: {:?}", temp_dir, err);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // Save the chunk to a temporary file
    let chunk_path = format!("{}/chunk_{}", temp_dir, chunk_number);
    let mut file = match File::create(&chunk_path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Failed to create chunk file: {:?}, Error: {:?}", chunk_path, err);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    
    if let Err(err) = file.write_all(&chunk_data) {
        eprintln!("Failed to write chunk data: {:?}, Error: {:?}", chunk_path, err);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // If all chunks are uploaded, assemble the file
    if is_upload_complete(&temp_dir, total_chunks) {
        if let Err(err) = assemble_file(&temp_dir, &file_name, total_chunks) {
            eprintln!("Failed to assemble file: {:?}, Error: {:?}", file_name, err);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::OK
}

fn is_upload_complete(temp_dir: &str, total_chunks: usize) -> bool {
    match fs::read_dir(temp_dir) {
        Ok(entries) => entries.count() == total_chunks,
        Err(_) => false,
    }
}

fn assemble_file(temp_dir: &str, file_name: &str, total_chunks: usize) -> std::io::Result<()> {
    let output_path = format!("./uploads/{}", file_name);
    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&output_path)?;

    for chunk_number in 0..total_chunks {
        let chunk_path = format!("{}/chunk_{}", temp_dir, chunk_number);
        let chunk_data = fs::read(&chunk_path)?;
        output_file.write_all(&chunk_data)?;
    }

    // Clean up the temporary chunks
    fs::remove_dir_all(temp_dir)?;

    Ok(())
}

// Sanitize filename to avoid directory traversal attacks
fn sanitize_filename(filename: &str) -> String {
    filename.replace(&['/', '\\'][..], "").replace("..", "")
}

// Download a file in chunks
#[derive(Deserialize)]
struct DownloadParams {
    fileName: String,
    offset: u64,
    chunkSize: usize,
}

async fn download_chunk(Query(params): Query<DownloadParams>) -> impl IntoResponse {
    let file_path = format!("./uploads/{}", sanitize_filename(&params.fileName));
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let mut buffer = vec![0; params.chunkSize];
    if let Err(_) = file.seek(SeekFrom::Start(params.offset)) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if bytes_read == 0 {
        return StatusCode::NO_CONTENT.into_response();
    }

    // Convert the Vec<u8> into a hyper Body
    let body = Body::from(buffer[..bytes_read].to_vec());

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .unwrap()
}
