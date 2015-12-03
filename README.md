# rustybot
rustybot is a [Tox](https://tox.chat) groupchat trivia bot written in Rust. 

## Features
rustybot is capable of playing multiple games at once in any number of groups. Player statistics including total points accumulated, rounds won, and games won, are stored in a database and persist across restarts. Tox ID's are used as database keys, which means peers will always be tied to the same entry as long as their Tox ID doesn't change.

She comes with a giant list of trivia questions of varying degrees of obscurity in `data/questions`. The default list can easily be replaced or modified as long as questions and answers are divided by the ` chacater, and each line ends in a \n byte.

## Usage and ownership
rustybot automatically accepts friend requests and group invites. The person who invites her to the group becomes her owner for that group and may use privileged commands. Additionally, all Tox ID's contained in the `data/masterkeys` file are her permanent owners and may use privileged commands in any group.

### Non-privileged commands
* `!help` - Print a list of non-privileged commands
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
