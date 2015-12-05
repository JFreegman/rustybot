/*  commands.rs
 *
 *
 *  Copyright (C) 2015 rustybot All Rights Reserved.
 *
 *  This file is part of rustybot.
 *
 *  rustybot is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  rustybot is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with rustybot. If not, see <http://www.gnu.org/licenses/>.
 *
 */

use std::fmt::Write;
use bot::Bot;
use group::{get_group_index, get_peer_public_key};
use check_privilege;

// Maximum number of entries to print for the stats command
const MAX_LEADERBOARD_ENTRIES: i32 = 10;

lazy_static! {
    static ref COMMANDS: Vec<Command> = {
        let init = vec![
            Command::new( "!disable",   cmd_disable ),
            Command::new( "!enable",    cmd_enable  ),
            Command::new( "!help",      cmd_help    ),
            Command::new( "!hint",      cmd_hint    ),
            Command::new( "!quit",      cmd_quit    ),
            Command::new( "!score",     cmd_score   ),
            Command::new( "!source",    cmd_source  ),
            Command::new( "!stats",     cmd_stats   ),
            Command::new( "!stop",      cmd_stop    ),
            Command::new( "!trivia",    cmd_trivia  ),
        ];

        init
    };
}

struct Command {
    name: String,
    func: fn(bot: &mut Bot, groupnumber: i32, peernumber: i32),
}

impl Command {
    fn new(name: &str, func: fn(bot: &mut Bot, groupnumber: i32, peernumber: i32)) -> Command {
        Command {
            name: name.to_string(),
            func: func,
        }
    }

    fn do_command(&self, bot: &mut Bot, groupnumber: i32, peernumber: i32) {
        let func = self.func;
        func(bot, groupnumber, peernumber);
    }
}

pub fn execute(bot: &mut Bot, groupnumber: i32, peernumber: i32, command: &str)
{
    for c in COMMANDS.iter() {
        if c.name == command {
            return c.do_command(bot, groupnumber, peernumber);
        }
    }
}

fn cmd_disable(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    if !check_privilege(bot, groupnumber, peernumber) {
        return;
    }

    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    bot.groups[index].disable_trivia();
    bot.groups[index].send_message(bot.tox, "Trivia has been disabled");
}

fn cmd_enable(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    if !check_privilege(bot, groupnumber, peernumber) {
        return;
    }

    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    bot.groups[index].send_message(bot.tox, "Trivia has been enabled.");
    bot.groups[index].enable_trivia();
}

fn cmd_help(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    bot.groups[index].send_message(bot.tox, "Commands: !trivia !hint !score !stats !source");
}

fn cmd_hint(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    let hint = bot.groups[index].trivia.get_hint();
    let mut message = String::new();
    write!(&mut message, "Hint: {}", hint).unwrap();
    bot.groups[index].send_message(bot.tox, &message);
}

fn cmd_quit(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    if !check_privilege(bot, groupnumber, peernumber) {
        return;
    }

    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    bot.groups[index].send_message(bot.tox, "Goodbye.");
    bot.del_group(groupnumber);
}

fn cmd_score(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let public_key = match get_peer_public_key(bot.tox, groupnumber, peernumber) {
        Some(key) => key.to_string(),
        None      => return,
    };

    let mut message = String::new();

    match bot.db.get_entry(&public_key) {
        Some(stats) => write!(&mut message, "{}: Games won: {}, Rounds won: {}, Total points: {}",
                             stats.nick, stats.games_won, stats.rounds_won, stats.points).unwrap(),
        None => write!(&mut message, "No entry found").unwrap(),
    }

    let grp_index = match get_group_index(bot, groupnumber) {
        Some(idx) => idx,
        None      => return,
    };

    bot.groups[grp_index].send_message(bot.tox, &message);
}


fn cmd_source(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    bot.groups[index].send_message(bot.tox, "https://github.com/JFreegman/rustybot/");
}

fn cmd_stats(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    let entries = bot.db.get_sorted_values();

    if entries.is_empty() {
        bot.groups[index].send_message(bot.tox, "Leaderboard is empty. Type !trivia to play!");
        return;
    }

    let mut count = 1;
    let mut message = String::new();
    write!(&mut message, "Leaderboard:\n").unwrap();

    for e in entries {
        write!(&mut message, "{}. {}....{} points....{} rounds....{} games\n",
               count, e.nick, e.points, e.rounds_won, e.games_won).unwrap();

        if count >= MAX_LEADERBOARD_ENTRIES {
            break;
        }

        count += 1;
    }

    bot.groups[index].send_message(bot.tox, &message);
}

fn cmd_stop(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    if !check_privilege(bot, groupnumber, peernumber) {
        return;
    }

    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    bot.groups[index].end_trivia(bot.tox, &mut bot.db);
}

fn cmd_trivia(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    if bot.groups[index].start_trivia(bot.tox) {
        bot.groups[index].send_message(bot.tox, "Trivia time!");
    }
}
