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
use db::*;
use rstox::core::*;

pub struct Peer {
    pub nick:            String,
    pub public_key:      String,
    pub round_score:     u64,
}

impl Peer {
    pub fn new(public_key: String, nick: String) -> Peer {
        Peer {
            nick: nick,
            public_key: public_key,
            round_score: 0,
        }
    }

    pub fn get_nick(&self) -> String {
        self.nick.to_string()
    }

    pub fn set_nick(&mut self, nick: &str) {
        self.nick = nick.to_string();
    }

    pub fn update_round_score(&mut self, points: u64) {
        self.round_score += points;
    }

    pub fn get_round_score(&self) -> u64 {
        self.round_score
    }

    pub fn clear_round(&mut self) {
        self.round_score = 0;
    }
}

pub struct GroupChat {
    pub groupnumber: u32,
    pub trivia:      Trivia,
    pub peers:       Vec<Peer>,
    pub owner_pk:    String,   // Public key of the friend who invited the bot to the group
}

impl GroupChat {
    pub fn new(groupnumber: u32, public_key: String) -> GroupChat {
        GroupChat {
            groupnumber: groupnumber,
            trivia: Trivia::new(),
            peers: Vec::new(),
            owner_pk: public_key,
        }
    }

    pub fn send_message(&self, tox: &mut Tox, message: &str) {
        match tox.send_conference_message(self.groupnumber, MessageType::Normal, message) {
            Ok(_)  => (),
            Err(e) => println!("Failed to send message to group {}: {:?}", self.groupnumber, e),
        };
    }

    /* Returns true if game is started */
    pub fn start_trivia(&mut self, tox: &mut Tox) -> bool {
        if self.trivia.running {
            return false;
        }

        if self.trivia.disabled {
            self.send_message(tox, "Trivia is disabled.");
            return false;
        }

        self.trivia.new_game();
        true
    }

    pub fn end_trivia(&mut self, tox: &mut Tox, db: &mut DataBase) {
        if !self.trivia.running {
            return;
        }

        self.trivia.reset();

        let mut winner_pk = String::new();
        let mut best_score = 0;

        for p in &mut self.peers {
            if p.round_score == 0 {
                continue;
            }

            let pk = p.public_key.to_string();

            if p.round_score > best_score && !pk.is_empty() {
                best_score = p.round_score;
                winner_pk = pk;
            }

            p.clear_round();
        }

        if best_score == 0 || winner_pk.is_empty() {
            self.send_message(tox, "Game over. Type !stats to see the leaderboard.");
            return;
        }

        let mut message = String::new();

        let index = match get_peer_index(&mut self.peers, &winner_pk) {
            Some(index) => index,
            None => {
                self.send_message(tox, "Game over. Type !stats to see the leaderboard.");
                return;
            }
        };


        let peername = self.peers[index].get_nick();
        write!(&mut message, "{} won the game with {} points. Type !stats to see the leaderboard.",
                peername, best_score).unwrap();

        self.send_message(tox, &message);

        db.update_score(&peername, &winner_pk, 0);
        db.save();
    }

    pub fn enable_trivia(&mut self) {
        self.trivia.disabled = false;
    }

    pub fn disable_trivia(&mut self) {
        self.trivia.disabled = true;
    }

    pub fn next_trivia_question(&mut self, tox: &mut Tox, questions: &Vec<String>, db: &mut DataBase) {
        if self.trivia.rounds > 0 && !self.trivia.winner && !self.trivia.answer.is_empty() {
            let mut message = String::new();
            write!(&mut message, "Time's up! The answer was: {}", self.trivia.answer).unwrap();
            self.send_message(tox, &message);
            self.trivia.end_timer = get_time();
        }

        if self.trivia.rounds >= MAX_ROUNDS {
            self.end_trivia(tox, db);
            return;
        }

        if !self.trivia.new_round(questions) {
            return;
        }

        let mut message = String::new();
        write!(&mut message, "ROUND {}: {}", self.trivia.rounds, self.trivia.question).unwrap();
        self.send_message(tox, &message);
    }
}

pub fn get_group_index(bot: &mut Bot, groupnumber: u32) -> Option<usize>
{
    let index = match bot.groups.iter().position(|g| g.groupnumber == groupnumber) {
        Some(index) => Some(index),
        None        => None,
    };

    index
}

pub fn get_peer_index(peers: &mut Vec<Peer>, public_key: &str) -> Option<usize>
{
    let index = match peers.iter().position(|p| p.public_key == public_key) {
        Some(index) => Some(index),
        None        => None,
    };

    index
}

pub fn get_peer_public_key(tox: &mut Tox, groupnumber: u32, peernumber: u32) -> Option<String>
{
    let public_key = match tox.get_peer_public_key(groupnumber, peernumber) {
        Ok(key) => Some(key.to_string()),
        Err(_)      => None,
    };

    public_key
}
