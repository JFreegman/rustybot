# rustybot
rustybot is a [Tox](https://tox.chat) groupchat trivia bot.

## Usage and ownership
rustybot automatically accepts friend requests as well as group invites. The person who invites her
to the group becomes her owner for that group and may use privileged commands in that group. Additionally, all Tox ID's contained in the `data/masterkeys` file are her permanent owners and may use privileged commands in any group.

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
`cargo build && cargo run`
