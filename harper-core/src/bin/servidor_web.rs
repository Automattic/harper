use harper_core::linting::{LintGroup, Linter, Suggestion};
use harper_core::{Dialect, Document};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

#[derive(Deserialize)]
struct CheckRequest {
    text: String,
}

#[derive(Serialize)]
struct LintResponse {
    start: usize,
    end: usize,
    message: String,
    suggestions: Vec<String>,
}

fn handle_client(mut stream: TcpStream, linter: &mut LintGroup, document_builder: &impl Fn(&str) -> Document) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }

    let mut content_length = 0;
    loop {
        let mut header_line = String::new();
        if reader.read_line(&mut header_line).is_err() || header_line == "\r\n" || header_line.is_empty() {
            break;
        }
        if header_line.to_lowercase().starts_with("content-length:") {
            if let Some(val) = header_line.split(':').nth(1) {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    if request_line.starts_with("OPTIONS") {
        let response = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Content-Type\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    if request_line.starts_with("POST /api/check") {
        let mut body_bytes = vec![0u8; content_length];
        if reader.read_exact(&mut body_bytes).is_ok() {
            let req_data: Result<CheckRequest, _> = serde_json::from_slice(&body_bytes);
            let text = match req_data {
                Ok(data) => data.text,
                Err(e) => {
                    println!("Error de-serializing JSON: {:?}", e);
                    String::from_utf8_lossy(&body_bytes).to_string()
                }
            };

            println!("--- TEXTO A ANALIZAR ---: {:?}", text);
            let doc = document_builder(&text);
            let lints = linter.lint(&doc);
            println!("--- LINTS ENCONTRADOS ---: {}", lints.len());

            let mut resp_lints = Vec::new();
            for lint in lints {
                let suggestions: Vec<String> = lint
                    .suggestions
                    .iter()
                    .map(|s| match s {
                        Suggestion::ReplaceWith(chars) => chars.iter().collect(),
                        Suggestion::Remove => String::new(),
                        _ => format!("{:?}", s),
                    })
                    .collect();

                resp_lints.push(LintResponse {
                    start: lint.span.start,
                    end: lint.span.end,
                    message: lint.message,
                    suggestions,
                });
            }

            let json_out = serde_json::to_string(&resp_lints).unwrap_or_else(|_| "[]".to_string());
            let http_resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                json_out.as_bytes().len(),
                json_out
            );
            let _ = stream.write_all(http_resp.as_bytes());
            return;
        }
    }

    // Servir la página web HTML interactiva en la raíz "/"
    let html_content = include_str!("../../../static_demo.html");
    let http_resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        html_content.as_bytes().len(),
        html_content
    );
    let _ = stream.write_all(http_resp.as_bytes());
}

fn main() {
    let port = 3000;
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("No se pudo iniciar el servidor web local");

    println!("========================================================");
    println!("   🌐 HARPER ESPAÑOL - SERVIDOR WEB INTERACTIVO EN VIVO");
    println!("========================================================");
    println!("Servidor escuchando en: http://127.0.0.1:{}", port);
    println!("Abre la dirección en tu navegador para probar el corrector.");
    println!("Presiona Ctrl+C para detener el servidor.\n");

    let dict = Arc::new(harper_core::spell::FstDictionary::curated_spanish());
    let mut linter = LintGroup::new_curated(dict, Dialect::Spanish);

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_client(stream, &mut linter, &|t| {
                Document::new(
                    t,
                    &harper_core::parsers::PlainEnglish,
                    &harper_core::spell::FstDictionary::curated_spanish(),
                )
            });
        }
    }
}
