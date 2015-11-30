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

use time::get_time;
use std::fmt::Write;
use bot::Bot;
use trivia::*;
use rstox::core::*;

pub struct Peer {
    pub nick:            String,
    pub public_key:      String,
    pub rounds_won:      i32,
    pub score:           i64,    // Score for current round
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

    pub fn get_nick(&self) -> String {
        return self.nick.to_string();
    }

    pub fn update_score(&mut self, points: i64) {
        self.score += points;
        self.rounds_won += 1;
    }

    pub fn get_score(&self) -> i64 {
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

    pub fn update_name(&mut self, tox: &mut Tox, peernumber: i32, public_key: String) {
        let index = match get_peer_index(&mut self.peers, public_key) {
            Some(index) => index,
            None        => return,
        };

        let peername = match tox.group_peername(self.groupnumber, peernumber) {
            Some(name) => name,
            None       => return,
        };

        self.peers[index].nick = peername.to_string();
    }

    pub fn send_message(&self, tox: &mut Tox, message: String) {
        match tox.group_message_send(self.groupnumber, &message) {
            Ok(_)  => (),
            Err(e) => println!("Failed to send message to group {}: {:?}", self.groupnumber, e),
        }
    }

    /* Returns true if game is started */
    pub fn start_trivia(&mut self, tox: &mut Tox) -> bool {
        if self.trivia.running {
            return false;
        }

        if self.trivia.disabled {
            self.send_message(tox, "Trivia is disabled.".to_string());
            return false;
        }

        self.trivia.new_game();
        true
    }

    pub fn stop_trivia(&mut self) {
        if !self.trivia.running {
            return;
        }

        self.trivia.reset();
    }

    pub fn enable_trivia(&mut self) {
        self.trivia.disabled = false;
    }

    pub fn disable_trivia(&mut self) {
        self.trivia.disabled = true;

        if self.trivia.running {
            self.stop_trivia();
        }
    }

    pub fn next_trivia_question(&mut self, tox: &mut Tox, questions: &mut Vec<String>) {
        if self.trivia.rounds > 0 && !self.trivia.winner && !self.trivia.answer.is_empty() {
            let mut message = String::new();
            write!(&mut message, "Time's up! The answer was: {}", self.trivia.answer).unwrap();
            self.send_message(tox, message);
            self.trivia.end_timer = get_time();
        }

        if self.trivia.rounds >= MAX_ROUNDS {
            self.stop_trivia();
            self.send_message(tox, "Game over. Type !stats to see the leaderboard.".to_string());
            return;
        }

        if !self.trivia.new_round(questions) {
            return;
        }

        let mut message = String::new();
        write!(&mut message, "ROUND {}: {}", self.trivia.rounds, self.trivia.question).unwrap();
        self.send_message(tox, message);
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
