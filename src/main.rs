/*  main.rs
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

#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate rand;
extern crate time;
extern crate rstox;
extern crate byteorder;

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::BufReader;
use std::cmp::*;
use rand::*;
use time::get_time;
use rstox::core::*;

mod db;
mod util;
use util::*;
mod trivia;
use self::trivia::*;
mod group;
use self::group::*;
mod bot;
use self::bot::*;
mod commands;
use self::commands::execute;

const QUESTIONS_PATH: &'static str = "data/questions";
const MASTERKEYS_PATH: &'static str = "data/masterkeys";
const DHT_NODES_PATH: &'static str = "data/DHTnodes";

// Time to wait between bootstrap attempts
const BOOTSTRAP_INTERVAL: i64 = 10;

// Number of random bootstrap nodes to connect to per try
const MAX_BOOTSTRAP_NODES: usize = 5;

// Use in case DHTnodes file fails to load
const BOOTSTRAP_IP: &'static str = "144.76.60.215";
const BOOTSTRAP_PORT: u16 = 33445;
const BOOTSTRAP_KEY: &'static str = "04119E835DF3E78BACF0F84235B300546AF8B936F035185E2A8E9E0A67C8924F";

fn load_tox() -> Option<Tox>
{
    let options = ToxOptions::new();

    let fp = match open_file(PROFILE_DATA_PATH, false) {
        Some(fp) => fp,
        None => {
            let tox = match Tox::new(options, None) {
                Ok(tox) => tox,
                Err(e)  => {
                    println!("Tox instance failed to initialize ({:?})", e);
                    return None;
                }
            };

            return Some(tox);
        }
    };

    let mut buf = Vec::new();
    let mut reader = BufReader::new(&fp);

    match reader.read_to_end(&mut buf) {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to read tox data to buffer: {}", Error::description(&e));
            return None;
        }
    };

    let tox = match Tox::new(options, Some(&mut buf)) {
        Ok(tox) => tox,
        Err(e)  => {
            println!("Tox instance failed to initialize ({:?})", e);
            return None;
        }
    };

    return Some(tox);
}

fn init_tox(tox: &mut Tox)
{
    match tox.set_name(NAME) {
        Ok(_)  => (),
        Err(e) => println!("Failed to set name ({:?})", e),
    };

    match tox.set_status_message(STATUS_MESSAGE) {
        Ok(_)  => (),
        Err(e) => println!("Failed to set status message ({:?})", e),
    };
}

fn bootstrap_backup(tox: &mut Tox)
{
    println!("Trying backup bootstrap server...");

    match tox.bootstrap(BOOTSTRAP_IP, BOOTSTRAP_PORT, BOOTSTRAP_KEY.parse().unwrap()) {
        Ok(_)  => (),
        Err(e) => println!("Failed to bootstrap with backup ({:?}).", e),
    }
}

fn bootstrap_tox(bot: &mut Bot)
{
    if !timed_out(bot.last_connect, BOOTSTRAP_INTERVAL) {
        return;
    }

    bot.last_connect = get_time();
    println!("Bootstrapping to DHT network...");

    let path = Path::new(DHT_NODES_PATH);
    let display = path.display();

    let mut fp = match File::open(&path) {
        Ok(fp) => fp,
        Err(e) => {
            println!("Failed to open file {}: {}", display, Error::description(&e));
            bootstrap_backup(bot.tox);
            return;
        }
    };

    let mut nodes_str = String::new();

    match fp.read_to_string(&mut nodes_str) {
        Ok(_)  => (),
        Err(e) => {
            println!("Failed to read file {}: {}", display, Error::description(&e));
            bootstrap_backup(bot.tox);
            return;
        }
    };

    let nodes: Vec<&str> = nodes_str.split("\n").collect();
    let num_nodes = nodes.len();
    let mut rng = thread_rng();

    for _ in 0..min(MAX_BOOTSTRAP_NODES, num_nodes as usize) {
        let idx = rng.gen_range(0, num_nodes);
        let node: Vec<&str> = nodes[idx].split(" ").collect();

        if node.len() != 3 {
            continue;
        }

        let ip = node[0];

        let port = match node[1].to_string().parse::<u16>() {
            Ok(port) => port,
            Err(_)   => continue,
        };

        let key = match node[2].parse() {
            Ok(key) => key,
            Err(_)  => continue,
        };

        match bot.tox.bootstrap(ip, port, key) {
            Ok(_)  => (),
            Err(e) => println!("Bootstrap failed: {:?}", e),
        }
    }
}

fn load_trivia_questions(bot: &mut Bot) -> Result<(), String>
{

    println!("Loading trivia questions...");

    let path = Path::new(QUESTIONS_PATH);
    let display = path.display();
    let mut questions = String::new();

    let mut fp = try!(File::open(&path).map_err(|e| format!("Open failed on file {}: {}", display, Error::description(&e))));
    try!(fp.read_to_string(&mut questions).map_err(|e| format!("Read failed on file {}: {}", display, Error::description(&e))));

    for line in questions.split("\n") {
        bot.questions.push(line.to_string());
    }

    Ok(())
}

// Returns true if peernumber is in the masterkeys list or is the owner of groupnumber
fn check_privilege(bot: &mut Bot, groupnumber: u32, peernumber: u32) -> bool
{
    let public_key = match get_peer_public_key(bot.tox, groupnumber, peernumber) {
        Some(key) => key.to_string(),
        None => {
            println!("Failed to fetch peer {}'s key in group {}", peernumber, groupnumber);
            return false;
        }
    };

    let path = Path::new(MASTERKEYS_PATH);
    let display = path.display();

    let mut fp = match File::open(&path) {
        Ok(fp) => fp,
        Err(e) => {
            println!("Failed to open file {}: {}", display, Error::description(&e));
            return false;
        }
    };

    let mut keys = String::new();

    match fp.read_to_string(&mut keys) {
        Ok(_)  => (),
        Err(e) => {
            println!("Failed to read file {}: {}", display, Error::description(&e));
            return false;
        }
    };

    for key in keys.split("\n") {
        if key.contains(&public_key) {
            return true;
        }
    }

    for g in &bot.groups {
        if g.groupnumber != groupnumber {
            continue;
        }

        if g.owner_pk == public_key {
            return true;
        }

        break;
    }

    false
}

fn cb_connection_status(bot: &mut Bot, status: Connection)
{
    match status {
        Connection::None => bot.last_connect = get_time(),
        _ => (),
    }

    println!("DHT connection status: {:?}", status);
}

fn cb_friend_request(bot: &mut Bot, id: PublicKey, message: &str)
{
    let id_string = id.to_string();
    println!("Friend request from:\n{}", id_string);
    println!("Message: {}", message);

    match bot.tox.add_friend_norequest(&id) {
        Ok(_)  => {
            println!("Friend added.");
            bot.save();
        }
        Err(e) => println!("Failed to add friend ({:?})", e),
    };
}

fn cb_group_invite(bot: &mut Bot, friendnumber: u32, _kind: ConferenceType, cookie: &Cookie)
{
    bot.add_group(friendnumber, cookie);
}

fn cb_group_peerlist_change(bot: &mut Bot, groupnumber: u32)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    let num_peers = bot.tox.conference_peer_count(groupnumber).unwrap();
    let mut new_list: Vec<Peer> = Vec::new();

    for i in 0..num_peers {
        let public_key = match bot.tox.get_peer_public_key(groupnumber, i) {
            Ok(public_key) => public_key.to_string(),
            Err(_) => continue,
        };

        match get_peer_index(&mut bot.groups[index].peers, &public_key) {
            Some(peer_idx) => {
                let old_nick = &bot.groups[index].peers[peer_idx].nick;
                new_list.push(Peer::new(public_key, old_nick.to_string()));
            },
            None => new_list.push(Peer::new(public_key, "Anonymous".to_string())),
        };
    }

    bot.groups[index].peers = new_list;
}

fn cb_group_message(bot: &mut Bot, groupnumber: u32, peernumber: u32, message: &str)
{
    if message.is_empty() {
        return;
    }

    if message.as_bytes()[0] == b'!' {
        execute(bot, groupnumber, peernumber, message);
    } else {
        process_answer(bot, groupnumber, peernumber, message);
    }
}

fn cb_group_peername_change(bot: &mut Bot, groupnumber: u32, peernumber: u32, name: &str)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index as u32,
        None        => return,
    };

    let public_key = match get_peer_public_key(bot.tox, groupnumber, peernumber) {
        Some(public_key) => public_key,
        None             => return,
    };

    bot.update_nick(index as usize, &name, &public_key);
}

fn do_tox(bot: &mut Bot)
{
    for event in bot.tox.iter() {
        match event {
            ConnectionStatus(status) =>
                cb_connection_status(bot, status),
            FriendRequest(id, message) =>
                cb_friend_request(bot, id, &message),
            ConferenceInvite { friend, kind, cookie } =>
                cb_group_invite(bot, friend, kind, &cookie),
            ConferencePeerListChanged { conference } =>
                cb_group_peerlist_change(bot, conference),
            ConferencePeerName { conference, peer, name } =>
                cb_group_peername_change(bot, conference, peer, &name),
            ConferenceMessage { conference, peer, kind: _, message } =>
                cb_group_message(bot, conference, peer, &message),
            _ => (),
        }
    }

    bot.tox.wait();
}

fn do_connection(bot: &mut Bot)
{
    match bot.tox.get_connection_status() {
        Connection::None => bootstrap_tox(bot),
        _ => (),
    }
}
fn do_rustybot(bot: &mut Bot)
{
    do_tox(bot);
    do_trivia(bot);
    do_connection(bot);
}

fn main()
{
    let mut tox = match load_tox() {
        Some(tox) => tox,
        None      => return,
    };

    init_tox(&mut tox);
    let mut bot = Bot::new(&mut tox);
    bot.save();
    bot.print_info();
    bot.db.load();

    match load_trivia_questions(&mut bot) {
        Ok(_)  => println!("Loaded."),
        Err(e) => println!("Trivia questions failed to load: {}", e),
    }

    loop {
        do_rustybot(&mut bot);
    }
}
