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

use std::error::Error;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::OpenOptions;
use std::path::Path;
use time::Timespec;
use rstox::core::*;
use group::{GroupChat, get_group_index};

pub const PROFILE_DATA_PATH: &'static str = "data/profile.tox";
pub const VERSION: &'static str = "0.1.0";
pub const NAME: &'static str = "rustybot";
pub const STATUS_MESSAGE: &'static str = "Invite me to a group. !trivia starts a game of trivia, !help for other commands.";

pub struct Bot<'a> {
    pub tox: &'a mut Tox,
    pub groups: Vec<GroupChat>,
    pub questions: Vec<String>,    // Stores all of the trivia questions/answers
    pub last_connect: Timespec,
    pub last_group_cleanup: Timespec,
}

impl<'a> Bot<'a> {
    pub fn new(tox: &mut Tox) -> Bot {
        Bot {
            tox: tox,
            groups: Vec::new(),
            questions: Vec::new(),
            last_connect: Timespec::new(0, 0),
            last_group_cleanup: Timespec::new(0, 0),
        }
    }

    pub fn save(&self) {
        let path = Path::new(PROFILE_DATA_PATH);
        let display = path.display();

        let mut options = OpenOptions::new();
        options.write(true);

        let fp = match options.open(&path) {
            Ok(fp) => fp,
            Err(e) => {
                println!("save() failed to open tox data file {}: {}", display, Error::description(&e));
                return;
            }
        };

        let data = self.tox.save();
        let mut writer = BufWriter::new(&fp);

        match writer.write(&data) {
            Ok(_)  => (),
            Err(e) => {
                println!("save() failed to write tox data to save file: {}", Error::description(&e));
                return;
            }
        }
    }

    pub fn add_group(&mut self, friendnumber: i32, key: Vec<u8>) {
        match self.tox.join_groupchat(friendnumber, &key) {
            Ok(groupnumber)  => {
                let friend_pk = match self.tox.get_friend_public_key(friendnumber as u32) {
                    Some(friend_pk) => friend_pk.to_string(),
                    None            => "BadKey".to_string(),
                };

                self.groups.push(GroupChat::new(groupnumber, friend_pk));

                let friend_name = match self.tox.get_friend_name(friendnumber as u32) {
                    Some(name) => name,
                    None       => "Anonymous".to_string(),
                };

                println!("Accepted group invite from {} ({})", friend_name, groupnumber);
            },
            Err(e) => println!("Failed to join group ({:?})", e),
        };
    }

    pub fn del_group(&mut self, groupnumber: i32) {
        let index = match get_group_index(self, groupnumber) {
            Some(index) => index,
            None => {
                println!("Failed to find index for groupnumber {}", groupnumber);
                return;
            }
        };

        self.groups.remove(index);

        match self.tox.del_groupchat(groupnumber) {
            Ok(_)  => (),
            Err(e) => {
                println!("Core failed to delete group{}: {:?}", groupnumber, e);
                return;
            }
        }

        println!("Leaving group {}", groupnumber);
    }

    pub fn send_group_message(&mut self, groupnumber: i32, message: String) {
        match self.tox.group_message_send(groupnumber, &message) {
            Ok(_)  => (),
            Err(e) => println!("Failed to send message to group {}: {:?}", groupnumber, e),
        }
    }

    pub fn print_info(&self) {
        println!("{} version {}", NAME, VERSION);
        println!("Name: {}", self.tox.get_name());
        println!("Status message: {}", self.tox.get_status_message());
        println!("Tox ID: {}", self.tox.get_address());
        println!("Friend count: {}", self.tox.get_friend_list().len());
    }
}
