/*  group.rs
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

use bot::Bot;
use trivia::Trivia;
use rstox::core::*;

pub struct Peer {
    pub nick:            String,
    pub public_key:      String,
    pub rounds_won:      i32,
    pub score:           i32,    // Score for current round
}

impl Peer {
    pub fn new(public_key: String) -> Peer {
        Peer {
            nick: "Anonymous".to_string(),
            public_key: public_key,
            rounds_won: 0,
            score: 0,
        }
    }

    pub fn update_score(&mut self, points: i32) {
        self.score += points;
        self.rounds_won += 1;
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn get_rounds_won(&self) -> i32 {
        self.rounds_won
    }
}

pub struct GroupChat {
    pub groupnumber: i32,
    pub trivia:      Trivia,
    pub peers:       Vec<Peer>,
    pub owner_pk:    String,   // Public key of the friend who invited the bot to the group
}

impl GroupChat {
    pub fn new(groupnumber: i32, public_key: String) -> GroupChat {
        GroupChat {
            groupnumber: groupnumber,
            trivia: Trivia::new(),
            peers: Vec::new(),
            owner_pk: public_key,
        }
    }

    pub fn add_peer(&mut self, public_key: String) {
        self.peers.push(Peer::new(public_key));
    }

    pub fn del_peer(&mut self, public_key: String) {
        let index = match get_peer_index(&mut self.peers, public_key) {
            Some(index) => index,
            None        => return,
        };

        self.peers.remove(index);
    }

    pub fn update_name(&mut self, tox: &mut Tox, groupnumber: i32, peernumber: i32, public_key: String) {
        let index = match get_peer_index(&mut self.peers, public_key) {
            Some(index) => index,
            None        => return,
        };

        let peername = match tox.group_peername(groupnumber, peernumber) {
            Some(name) => name,
            None       => return,
        };

        self.peers[index].nick = peername.to_string();
    }
}

pub fn get_group_index(bot: &mut Bot, groupnumber: i32) -> Option<usize>
{
    let index = match bot.groups.iter().position(|g| g.groupnumber == groupnumber) {
        Some(index) => Some(index),
        None        => None,
    };

    index
}

pub fn get_peer_index(peers: &mut Vec<Peer>, public_key: String) -> Option<usize>
{
    let index = match peers.iter().position(|p| p.public_key == public_key) {
        Some(index) => Some(index),
        None        => None,
    };

    index
}

pub fn get_peer_public_key(tox: &mut Tox, groupnumber: i32, peernumber: i32) -> Option<String>
{
    let public_key = match tox.group_peer_pubkey(groupnumber, peernumber) {
        Some(key) => Some(key.to_string()),
        None      => None,
    };

    public_key
}
