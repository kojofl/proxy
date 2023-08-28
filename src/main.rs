use std::io;

use actix_proxy::{IntoHttpResponse, SendRequestError};
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer};
use awc::Client;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[get("{url:,*}")]
async fn proxy_get(
    path: web::Path<(String,)>,
    client: web::Data<Client>,
) -> Result<HttpResponse, SendRequestError> {
    let (url,) = path.into_inner();

    let url = format!("http://localhost:3000/{url}");

    // here we use `IntoHttpResponse` to return the request to
    // duckduckgo back to the client that called this endpoint
    Ok(client.get(&url).send().await?.into_http_response())
}

#[post("{url:,*}")]
async fn proxy_post(
    path: web::Path<(String,)>,
    data: web::Json<serde_json::Value>,
    client: web::Data<Client>,
) -> Result<HttpResponse, SendRequestError> {
    let (url,) = path.into_inner();

    let url = format!("http://localhost:3000/{url}");

    // here we use `IntoHttpResponse` to return the request to
    // duckduckgo back to the client that called this endpoint
    Ok(client
        .post(&url)
        .send_json(&data.into_inner())
        .await?
        .into_http_response())
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    println!("Started http server: 127.0.0.1:443");

    // load TLS keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(
            "/etc/letsencrypt/live/blubi.codes/privkey.pem",
            SslFiletype::PEM,
        )
        .unwrap();
    builder
        .set_certificate_chain_file("/etc/letsencrypt/live/blubi.codes/fullchain.pem")
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Client::default()))
            .service(proxy_get)
            .service(proxy_post)
            // enable logger
            .wrap(middleware::Logger::default())
    })
    .bind_openssl("0.0.0.0:443", builder)?
    .bind("0.0.0.0:80")?
    .run()
    .await
}
