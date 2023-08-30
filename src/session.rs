use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::server;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsSession {
    pub id: u16,
    pub hb: Instant,
    pub addr: Addr<server::WsServer>,
}

impl WsSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting due to client timeout");
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.id = res;
                        ctx.text(format!("{{\"type\": \"id\", \"value\": {} }}", act.id));
                    }
                    Err(_) => ctx.stop(),
                }

                fut::ready(())
            })
            .wait(ctx);
    }
}

impl Handler<server::Message> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0.to_string())
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let Ok(msg) = item else {
            ctx.stop();
            return;
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Text(t) => {
                println!("{}", &t);
                ctx.text(t);
            }
            _ => {}
        }
    }
}
