/*  trivia.rs
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

use rand::*;
use time::{get_time, Timespec, Duration};
use bot::Bot;
use rstox::core::*;
use std::fmt::Write;
use group::{get_group_index, get_peer_index, get_peer_public_key};

// Number of seconds before the answer is given
const QUESTION_TIME_LIMIT: i64 = 30;

// Max number of hints for a question
const MAX_HINTS: u32 = 4;

// Max number of rounds
const MAX_ROUNDS: u32 = 30;

// Seconds to wait between rounds
const ROUND_DELAY: i64 = 3;

pub struct Trivia {
    pub running:     bool,        // True if a game is currently going
    pub rounds:      u32,         // Current round number
    pub hints:       u32,         // Number of hints given for the current round
    pub round_timer: Timespec,    // Time since round began
    pub end_timer:   Timespec,    // Time since last round ended
    pub question:    String,      // Current round's question
    pub answer:      String,      // Current round's answer
    pub winner:      bool,        // True if the round has been won
    pub disabled:    bool,        // True if trivia has been disabled
}

impl Trivia {
    pub fn new() -> Trivia {
        Trivia {
            running: false,
            rounds: 0,
            hints: 0,
            round_timer: Timespec::new(0, 0),
            end_timer: Timespec::new(0, 0),
            question: String::new(),
            answer: String::new(),
            winner: false,
            disabled: false,
        }
    }

    pub fn start(&mut self, tox: &mut Tox, groupnumber: i32) -> bool {  // Returns true if game is started
        if self.running {
            return false;
        }

        if self.disabled {
            self.send_message(tox, groupnumber, "Trivia is disabled.".to_string());
            return false;
        }

        self.running = true;
        true
    }

    pub fn stop(&mut self) {
        if !self.running {
            return;
        }

        self.running = false;
        self.rounds = 0;
        self.hints = 0;
        self.question.clear();
        self.answer.clear();
        self.round_timer = Timespec::new(0, 0);
        self.end_timer = Timespec::new(0, 0);
        self.winner = false;
    }

    pub fn disable(&mut self) {
        self.disabled = true;

        if self.running {
            self.stop();
        }
    }

    pub fn enable(&mut self) {
        self.disabled = false;
    }

    fn next_question(&mut self, tox: &mut Tox, groupnumber: i32, questions: &mut Vec<String>) {
        if self.rounds > 0 && !self.winner {
            if self.answer.len() > 0 {
                let mut message = String::new();
                write!(&mut message, "Time's up! The answer was: {}", self.answer).unwrap();
                self.send_message(tox, groupnumber, message);
                self.end_timer = get_time();
            }
        }

        if self.rounds >= MAX_ROUNDS {
            self.stop();
            self.send_message(tox, groupnumber, "Game over. Type !stats to see the leaderboard.".to_string());
            return;
        }

        self.winner = false;
        self.question.clear();
        self.answer.clear();

        if get_time() - self.end_timer <= Duration::seconds(ROUND_DELAY) {
            return;
        }

        let index = thread_rng().gen_range(0, questions.len());
        let split_entry: Vec<&str> = questions[index].split("`").collect();

        if split_entry.len() != 2 {
            println!("Error parsing question index {}: {:?}", index, split_entry);
            return;
        }

        self.hints = 0;
        self.rounds += 1;
        self.round_timer = get_time();
        self.question = split_entry[0].to_string();
        self.answer = split_entry[1].to_string().to_lowercase();

        let mut message = String::new();
        write!(&mut message, "ROUND {}: {}", self.rounds, self.question).unwrap();
        self.send_message(tox, groupnumber, message);
    }

    pub fn hint(&mut self, tox: &mut Tox, groupnumber: i32) {
        if !self.running {
            return;
        }

        if self.hints >= MAX_HINTS {
            self.send_message(tox, groupnumber, "No more hints.".to_string());
            return;
        }

        self.hints += 1;

        let hint = get_hint(&mut self.answer, self.hints as usize);
        let mut message = String::new();
        write!(&mut message, "Hint {}: {}", self.hints, hint).unwrap();
        self.send_message(tox, groupnumber, message);
    }

    // use this instead of bot.send_group_message
    pub fn send_message(&self, tox: &mut Tox, groupnumber: i32, message: String) {
        match tox.group_message_send(groupnumber, &message) {
            Ok(_)  => (),
            Err(e) => println!("Failed to send message to group {}: {:?}", groupnumber, e),
        }
    }
}

// Returns either the first (hints * 2) characters of the answer, or the first half of the answer
fn get_hint(answer: &mut String, hints: usize) -> String
{
    let mut hint = String::new();
    let num_chars = hints * 2;
    let max_len = (answer.len() / 2) + 1;

    if num_chars > max_len {
        return "No more hints".to_string();
    }

    for (i, ch) in answer.chars().enumerate() {
        hint = hint + &ch.to_string();

        if (i + 1) == num_chars {
            break;
        }
    }

    hint
}

pub fn process_answer(bot: &mut Bot, groupnumber: i32, peernumber: i32, msg: String)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None        => return,
    };

    if !bot.groups[index].trivia.running {
        return;
    }

    if msg.to_lowercase() != bot.groups[index].trivia.answer {
        return;
    }

    let public_key = match get_peer_public_key(bot.tox, groupnumber, peernumber) {
        Some(key) => key,
        None      => return,
    };

    let peername = match bot.tox.group_peername(groupnumber, peernumber) {
        Some(name) => name,
        None       => "Anonymous".to_string(),
    };

    let delta = Duration::seconds(QUESTION_TIME_LIMIT) - (get_time() - bot.groups[index].trivia.round_timer);
    let points = Duration::num_seconds(&delta) + 1;

    let peer_idx = match get_peer_index(&mut bot.groups[index].peers, public_key) {
        Some(idx) => idx,
        None      => return,
    };

    bot.groups[index].peers[peer_idx].update_score(points as i32);

    let score = bot.groups[index].peers[peer_idx].score;
    let mut message = String::new();
    write!(&mut message, "{} got the answer for {} points (total score: {})", peername, points, score).unwrap();
    bot.send_group_message(groupnumber, message);

    bot.groups[index].trivia.winner = true;
    bot.groups[index].trivia.end_timer = get_time();
    bot.groups[index].trivia.round_timer = Timespec::new(0, 0);
}

pub fn do_trivia(bot: &mut Bot)
{
    for group in &mut bot.groups {
        if group.trivia.running {
            if get_time() - group.trivia.round_timer >= Duration::seconds(QUESTION_TIME_LIMIT) {
                group.trivia.next_question(bot.tox, group.groupnumber, &mut bot.questions);
            }
        }
    }
}
