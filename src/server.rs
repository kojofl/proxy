use std::collections::HashMap;

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(u16)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: u16,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Prompt {
    pub id: u16,
    pub data: String,
}

#[derive(Debug)]
pub struct WsServer {
    sessions: HashMap<u16, Recipient<Message>>,
    rng: ThreadRng,
}

impl WsServer {
    pub fn new() -> WsServer {
        let sessions = HashMap::new();

        WsServer {
            sessions,
            rng: rand::thread_rng(),
        }
    }

    pub fn send_message(&self, id: u16, msg: String) {
        if let Some(addr) = self.sessions.get(&id) {
            println!("Socket found");
            addr.do_send(Message(msg));
        }
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

impl Handler<Prompt> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Prompt, _ctx: &mut Self::Context) -> Self::Result {
        self.send_message(msg.id, msg.data);
    }
}

impl Handler<Connect> for WsServer {
    type Result = u16;

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        let id = self.rng.gen::<u16>();
        println!("New session connected: {}", id);
        self.sessions.insert(id, msg.addr);
        id
    }
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        println!("Session Disconnect: {}", &msg.id);
        self.sessions.remove(&msg.id);
    }
}
