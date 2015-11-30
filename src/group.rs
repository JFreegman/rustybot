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
    pub total_score:     i64,
    pub round_score:     i64,
    pub games_won:       i32,
}

impl Peer {
    pub fn new(public_key: String) -> Peer {
        Peer {
            nick: "Anonymous".to_string(),
            public_key: public_key,
            rounds_won: 0,
            total_score: 0,
            round_score: 0,
            games_won: 0,
        }
    }

    pub fn get_nick(&self) -> String {
        return self.nick.to_string();
    }

    pub fn update_score(&mut self, points: i64) {
        self.round_score += points;
        self.total_score += points;
        self.rounds_won += 1;
    }

    pub fn clear_round_score(&mut self) {
        self.round_score = 0;
    }

    pub fn get_total_score(&self) -> i64 {
        self.total_score
    }

    pub fn get_round_score(&self) -> i64 {
        self.round_score
    }

    pub fn get_rounds_won(&self) -> i32 {
        self.rounds_won
    }

    pub fn get_games_won(&self) -> i32 {
        self.games_won
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

    pub fn end_trivia(&mut self, tox: &mut Tox) {
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

            if p.round_score > best_score {
                best_score = p.round_score;
                winner_pk = p.public_key.to_string();
            }

            p.clear_round_score();
        }

        if best_score == 0 || winner_pk.is_empty() {
            return;
        }

        let mut message = String::new();

        let index = match get_peer_index(&mut self.peers, winner_pk) {
            Some(index) => index,
            None => {
                self.send_message(tox, "Game over. Type !stats to see the leaderboard.".to_string());
                return;
            }
        };

        self.peers[index].games_won += 1;

        let peername = self.peers[index].get_nick();
        write!(&mut message, "{} won the game with {} points. Type !stats to see the leaderboard.",
                peername, best_score).unwrap();

        self.send_message(tox, message);
    }

    pub fn enable_trivia(&mut self) {
        self.trivia.disabled = false;
    }

    pub fn disable_trivia(&mut self, tox: &mut Tox) {
        self.trivia.disabled = true;

        if self.trivia.running {
            self.end_trivia(tox);
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
            self.end_trivia(tox);
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
