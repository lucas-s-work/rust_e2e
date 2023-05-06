use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Result};
use inquire::{Confirm, Select, Text};

use crate::connection::start;

use self::{friend::Friend, user::User};

mod crypto;
pub mod friend;
pub mod message;
pub mod user;

enum TermMode {
    SelectUser,
    Connect,
    Select,
    Message,
}

use TermMode::*;

pub fn run() -> Result<()> {
    let mut mode = SelectUser;
    let user = Arc::new(Mutex::new(select_user()?));
    mode = Connect;
    let connection = start(user.clone())?;
    mode = Select;

    Ok(())
}

fn background_updates() {}

fn select_user() -> Result<User> {
    let should_load_user = Confirm::new("Do you wish to load an existing user?")
        .with_default(false)
        .prompt()?;

    match should_load_user {
        true => load_user(),
        false => return create_user(),
    }
}

fn load_user() -> Result<User> {
    bail!("unimplemented")
}

fn create_user() -> Result<User> {
    let nickname = Text::new("enter nickname").prompt()?;
    User::new(&nickname)
}

fn select_friend(user: &User) -> Result<String> {
    let possible_friend_ids = user.get_friend_ids();
    let mut unique_name_to_ids: HashMap<String, String> = HashMap::new();

    for (id, nickname) in possible_friend_ids {
        unique_name_to_ids.insert(format!("{}_{}", nickname, id), id);
    }

    let chosen_unique_name = Select::new(
        "Who do you want to talk with?",
        unique_name_to_ids.keys().collect(),
    )
    .prompt()?;

    Ok(chosen_unique_name.to_string())
}

fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}
