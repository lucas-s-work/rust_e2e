use user::User;

mod crypto;
mod friend;
mod message;
mod user;

fn main() {
    let mut user = User::new().expect("created user");
    let friend_user = User::new()
        .expect("friend created")
        .to_friend()
        .expect("became friend");
    user.add_friend(friend_user).expect("added fried");
    println!("{:?}", user)
}
