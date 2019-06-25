/*  bot.rs
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

use time::Timespec;
use rstox::core::*;
use group::{GroupChat, get_group_index, get_peer_index};
use db::*;
use util::*;

pub const PROFILE_DATA_PATH: &'static str = "data/profile.tox";
pub const VERSION: &'static str = "0.2.0";
pub const NAME: &'static str = "rustybot";
pub const STATUS_MESSAGE: &'static str = "Invite me to a group. !trivia starts a game of trivia, !help for other commands.";

pub struct Bot<'a> {
    pub tox:          &'a mut Tox,
    pub groups:       Vec<GroupChat>,
    pub questions:    Vec<String>,    // Stores all of the trivia questions/answers
    pub last_connect: Timespec,
    pub db:           DataBase,
}

impl<'a> Bot<'a> {
    pub fn new(tox: &mut Tox) -> Bot {
        Bot {
            tox: tox,
            groups: Vec::new(),
            questions: Vec::new(),
            last_connect: Timespec::new(0, 0),
            db: DataBase::new(),
        }
    }

    pub fn save(&self) {
        let data = self.tox.save();

        match save_data(PROFILE_DATA_PATH, &data) {
            Ok(_) => (),
            Err(e) => println!("save_data failed: {}", e),
        };
    }

    pub fn add_group(&mut self, friendnumber: u32, cookie: &Cookie) {
        match self.tox.join_conference(friendnumber, cookie) {
            Ok(groupnumber)  => {
                let friend_pk = match self.tox.get_friend_public_key(friendnumber as u32) {
                    Some(friend_pk) => friend_pk.to_string(),
                    None            => "BadKey".to_string(),
                };

                self.groups.push(GroupChat::new(groupnumber, friend_pk));
                let friend_name = self.tox.get_friend_name(friendnumber as u32).unwrap_or("Anonymous".to_string());
                println!("Accepted group invite from {} ({})", friend_name, groupnumber);
            },
            Err(e) => println!("Failed to join group ({:?})", e),
        };
    }

    pub fn del_group(&mut self, groupnumber: u32) {
        let index = match get_group_index(self, groupnumber) {
            Some(index) => index,
            None => return println!("Failed to find index for groupnumber {}", groupnumber),
        };

        self.groups.remove(index);
        self.tox.delete_conference(groupnumber);

        println!("Leaving group {}", groupnumber);
    }

    pub fn leave_all_groups(&mut self) {
        for g in &self.groups {
            self.tox.delete_conference(g.groupnumber);
        }
    }

    /* Updates the nick in both the respective group's peerlist, and in the database */
    pub fn update_nick(&mut self, group_index: usize, nick: &str, public_key: &str) {
        let peer_idx = match get_peer_index(&mut self.groups[group_index].peers, public_key) {
            Some(idx) => idx,
            None      => return,
        };

        self.groups[group_index].peers[peer_idx].set_nick(nick);
        self.db.set_nick(nick, public_key);
    }

    pub fn print_info(&self) {
        println!("{} version {}", NAME, VERSION);
        println!("Name: {}", self.tox.get_name());
        println!("Status message: {}", self.tox.get_status_message());
        println!("Tox ID: {}", self.tox.get_address());
        println!("Friends: {}", self.tox.get_friend_list().len());
    }
}
