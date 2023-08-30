mod events;
mod server;
mod session;
use std::{io, time::Instant};

use actix::{Actor, Addr};
use actix_cors::*;
use actix_web::{error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use awc::Client;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use crate::{events::Event, server::Prompt};

async fn connect(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::WsServer>>,
) -> Result<HttpResponse, Error> {
    let res = ws::start(
        session::WsSession {
            id: 0,
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    );
    res
}

async fn webhook(
    web::Json(data): web::Json<Event>,
    ws_server: web::Data<Addr<server::WsServer>>,
) -> Result<HttpResponse, Error> {
    match data {
        Event::Registration(e) => {
            if let Some(data) = e.data {
                let data_string = serde_json::to_string(&data).unwrap();
                let id = data.session_id.unwrap();
                println!("Sending to: {}", id);
                ws_server.do_send(Prompt {
                    data: data_string,
                    id: id.parse().unwrap(),
                });
            }
        }
        Event::Onboarding(e) => {
            println!("{:#?}", e);
        }
        Event::Login(e) => {
            println!("{e:?}");
            let s_id = e.session_id.unwrap();
            if let Some(data) = e.data {
                let data_string = serde_json::to_string(&data).unwrap();
                println!("Sending to: {}", s_id);
                ws_server.do_send(Prompt {
                    data: data_string,
                    id: s_id.parse().unwrap(),
                });
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}

async fn forward(
    req: HttpRequest,
    payload: web::Payload,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let forward_req = client
        .request_from(format!("http://localhost:3000{}", req.path()), req.head())
        .no_decompress();

    let res = forward_req
        .send_stream(payload)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut client_res = HttpResponse::build(res.status());

    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_res.insert_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_res.streaming(res))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    println!("Started http server: 127.0.0.1:443");

    let ws_server = server::WsServer::new().start();

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
            .app_data(web::Data::new(ws_server.clone()))
            .route("/ws", web::get().to(connect))
            .route("/webhook", web::post().to(webhook))
            .default_service(web::to(forward))
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
    })
    .bind_openssl("0.0.0.0:443", builder)?
    .bind("0.0.0.0:80")?
    .run()
    .await
}
