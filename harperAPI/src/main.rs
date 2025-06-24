// use actix_web::{web, App, HttpServer};
// // Import the handler from our library file.
// use harperAPI::lint_text;

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     println!("Starting server at http://127.0.0.1:8080");

//     // Start the HTTP server.
//     HttpServer::new(|| {
//         App::new()
//             // Define a POST route at `/lint` that uses our `lint_text` handler.
//             .route("/lint", web::post().to(lint_text))
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }


use actix_web::{web, App, HttpServer};
// Import the handler from our library file.
use harperAPI::lint_text;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Updated to reflect the new address
    println!("Starting server at http://0.0.0.0:8000");

    // Start the HTTP server.
    HttpServer::new(|| {
        App::new()
            // Define a POST route at `/lint` that uses our `lint_text` handler.
            .route("/lint", web::post().to(lint_text))
    })
    // Bind to 0.0.0.0 to make it accessible on all network interfaces
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
