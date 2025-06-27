use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer};
// Import the handler from our library file.
use harper_api::lint_text;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Log that the server is starting.
    println!("Starting server at http://0.0.0.0:8000");

    // Start the HTTP server.
    HttpServer::new(|| {
        // Configure CORS middleware.
        // This setup is permissive and suitable for development.
        // For production, you should restrict origins to your specific frontend URL.
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            // Wrap the app in the CORS middleware.
            .wrap(cors)
            // Define a POST route at `/lint` that uses our `lint_text` handler.
            .route("/lint", web::post().to(lint_text))
    })
    // Bind to 0.0.0.0 to make it accessible on all network interfaces.
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
