use crate::client::user::User;

mod client;

fn main() {
    let mut user = User::new("lucas").expect("created user");
    let friend_user = User::new("steve")
        .expect("friend created")
        .to_friend()
        .expect("became friend");
    user.add_friend(friend_user).expect("added fried");
    println!("{:?}", user)
}
