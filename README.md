# rustybot
rustybot is a [Tox](https://tox.chat) groupchat trivia bot written in Rust.

## Features
rustybot is capable of playing multiple games at once in any number of groups. Player statistics including total points accumulated, rounds won, and games won, are stored in a database and persist across restarts. Tox ID's are used as database keys, which means peers will always be tied to the same entry as long as their Tox ID doesn't change.

She reads questions from the `data/questions` file which will need to be provided by the owner. Questions and answers must be divided by the ` character and each line must end in a \n byte. An example list can be found [here](https://gist.github.com/JFreegman/d0cc3952669059b78bf7ec2889384523).

## Usage and ownership
rustybot automatically accepts friend requests and group invites. The person who invites her to the group becomes her owner for that group and may use privileged commands. Additionally, all Tox ID's contained in the `data/masterkeys` file are her permanent owners and may use privileged commands in any group.

### Non-privileged commands
* `!help` - Print a list of non-privileged commands
* `!hint` - Display a hint for the current question
* `!source` - Link to the source code
* `!score` - Print your score
* `!stats` - Print the leaderboard
* `!trivia` - Begin a game of trivia

### Privileged commands
* `!quit` - Leave the groupchat
* `!stop` - End the current trivia game
* `!disable` - Disables the trivia command
* `!enable` - Enables the trivia command

## Compiling and running
`cargo build && cargo run` or just `cargo run`
