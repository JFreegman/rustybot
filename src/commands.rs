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
use group::{get_group_index, get_peer_index, get_peer_public_key};
use check_privilege;

lazy_static! {
    static ref COMMANDS: Vec<Command> = {
        let init = vec![
            Command::new( "!disable",   cmd_disable ),
            Command::new( "!enable",    cmd_enable  ),
            Command::new( "!help",      cmd_help    ),
            Command::new( "!hint",      cmd_hint    ),
            Command::new( "!quit",      cmd_quit    ),
            Command::new( "!score",     cmd_score   ),
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

pub fn execute(bot: &mut Bot, groupnumber: i32, peernumber: i32, command: String)
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

    bot.groups[index].trivia.disable();
    bot.send_group_message(groupnumber, "Trivia has been disabled.".to_string());
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

    bot.send_group_message(groupnumber, "Trivia has been enabled.".to_string());
    bot.groups[index].trivia.enable();
}

fn cmd_help(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    bot.send_group_message(groupnumber, "Commands: !help !trivia !score !stats !hint".to_string());
}

fn cmd_hint(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    bot.groups[index].trivia.hint(bot.tox, groupnumber);
}

fn cmd_quit(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    if !check_privilege(bot, groupnumber, peernumber) {
        return;
    }

    bot.send_group_message(groupnumber, "Goodbye.".to_string());
    bot.del_group(groupnumber);
}

fn cmd_score(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let public_key = match get_peer_public_key(bot.tox, groupnumber, peernumber) {
        Some(key) => key.to_string(),
        None      => return,
    };

    let peername = match bot.tox.group_peername(groupnumber, peernumber) {
        Some(name) => name,
        None       => "Anonymous".to_string(),
    };

    let grp_index = match get_group_index(bot, groupnumber) {
        Some(idx) => idx,
        None      => return,
    };

    let peer_idx = match get_peer_index(&mut bot.groups[grp_index].peers, public_key) {
        Some(idx) => idx,
        None      => return,
    };

    let score = bot.groups[grp_index].peers[peer_idx].get_score();
    let rounds_won = bot.groups[grp_index].peers[peer_idx].get_rounds_won();

    let mut message = String::new();
    write!(&mut message, "{}: Rounds won: {}, total score: {}", peername, rounds_won, score).unwrap();
    bot.send_group_message(groupnumber, message);
}

fn cmd_stats(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let mut message = String::new();
    write!(&mut message, "Leaderboard:\n").unwrap();

    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    bot.groups[index].peers.sort_by(|a, b| a.score.cmp(&b.score).reverse());

    let mut count = 1;

    for peer in bot.groups[index].peers.iter() {
        let score = peer.score;
        let rounds_won = peer.rounds_won;
        let ref peername = peer.nick;

        if score == 0 && rounds_won == 0 {
            continue;
        }

        write!(&mut message, "{}: {}: Total score: {}, rounds won: {}\n", count, peername, score, rounds_won).unwrap();

        if count == 10 {
            break;
        }

        count += 1;
    }

    bot.send_group_message(groupnumber, message);
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

    bot.groups[index].trivia.stop();
    bot.send_group_message(groupnumber, "Trivia time is over.".to_string());
}

fn cmd_trivia(bot: &mut Bot, groupnumber: i32, peernumber: i32)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    if bot.groups[index].trivia.start(bot.tox, groupnumber) {
        bot.send_group_message(groupnumber, "Trivia time!".to_string());
    }
}
