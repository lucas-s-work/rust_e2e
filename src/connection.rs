use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, Thread},
    time::Duration,
};

use anyhow::{bail, Result};
use inquire::Text;

use crate::client::{message::EncryptedMessage, user::User};

pub fn start(user: Arc<Mutex<User>>) -> Result<Sender<EncryptedMessage>> {
    let mut stream = connect()?;
    let (send, recv) = mpsc::channel::<EncryptedMessage>();
    thread::spawn(move || run(stream, user, recv).unwrap());
    Ok(send)
}

fn run(
    mut stream: TcpStream,
    user: Arc<Mutex<User>>,
    recv: Receiver<EncryptedMessage>,
) -> Result<()> {
    loop {
        // Check if we have any messages to send
        match recv.try_recv() {
            Ok(message) => send_message(&mut stream, message)?,
            Err(TryRecvError::Disconnected) => return Ok(()),
            Err(TryRecvError::Empty) => (),
        };

        // Now check if any messages have been received
        let mut msg = String::new();
        stream.read_to_string(&mut msg)?;
        // This is technically incorrect, if we receive multiple messages near simultaneously
        // then we could end up with multiple things in here, or even partial data in here.
        let components: Vec<_> = msg.split("\n").map(|s| s.to_string()).collect();
    }
}

fn connect() -> Result<TcpStream> {
    let url = Text::new("enter relay URL").prompt()?;
    let stream = TcpStream::connect(url)?;
    stream.set_read_timeout(Some(Duration::new(1, 0)))?;
    Ok(stream)
}

fn send_message(stream: &mut TcpStream, message: EncryptedMessage) -> Result<()> {
    let message_json = serde_json::to_string(&message)?;
    let request = format!("send\n{}", message_json);
    stream.write(request.as_bytes())?;
    Ok(())
}

fn recv_message(user: &Arc<Mutex<User>>, full_message: Vec<&str>) -> Result<()> {
    // We could have either received a message or a set of users.
    match full_message[0] {
        "message" => recv_encrypted_message(user, full_message[1]),
        "users" => recv_users(user, &full_message[1..]),
        _ => bail!("Unknown message directive recieved"),
    }
}

fn recv_encrypted_message(user: &Arc<Mutex<User>>, message: &str) -> Result<()> {
    user.lock()
        .unwrap()
        .receive_message(serde_json::from_str(message)?)
}

fn recv_users(user: &Arc<Mutex<User>>, users: &[&str]) -> Result<()> {
    Ok(())
}
