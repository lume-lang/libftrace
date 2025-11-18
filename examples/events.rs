use ftrace::*;

#[derive(Debug)]
struct User {
    name: String,
    friends: Vec<User>,
}

#[traced(level = Info, fields(name = user.name))]
fn process_user(user: User) {
    debug!("processing {} friends", user.friends.len());

    for friend in &user.friends {
        trace!("adding {} as friend", friend.name);
    }
}

fn main() {
    process_user(User {
        name: String::from("John Doe"),
        friends: vec![
            User {
                name: String::from("Jane Doe"),
                friends: Vec::new(),
            },
            User {
                name: String::from("Jax Doe"),
                friends: Vec::new(),
            },
            User {
                name: String::from("John Bow"),
                friends: Vec::new(),
            },
        ],
    });
}
