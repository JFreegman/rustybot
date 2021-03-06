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

use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use time::{get_time, Timespec, Duration};
use bot::Bot;
use std::fmt::Write;
use group::{get_group_index, get_peer_index, get_peer_public_key};
use util::*;

const PUNCTUATION: &'static str = " .,':;<>/\\=-()*&^%$#@![]{}|~?\"";

// Number of seconds before the answer is given
const QUESTION_TIME_LIMIT: i64 = 30;

// Minimum number of points to win in a round
const BASE_POINTS: i64 = 30;

// Points multiplier for time bonus
const BONUS_POINTS_MULTIPLIER: i64 = 4;

// Max number of rounds
pub const MAX_ROUNDS: u32 = 30;

// Seconds to wait between rounds
const ROUND_DELAY: i64 = 3;

pub struct Trivia {
    pub question:    String,      // Current round's question
    pub answer:      String,      // Current round's answer
    pub running:     bool,        // True if a game is currently going
    pub rounds:      u32,         // Current round number
    pub hints:       Vec<String>, // Colleciton of current round's hints
    pub hint_count:  usize,       // Number of hints given for the current round
    pub round_timer: Timespec,    // Time since round began
    pub end_timer:   Timespec,    // Time since last round ended
    pub winner:      bool,        // True if the round has been won
    pub disabled:    bool,        // True if trivia has been disabled
    pub owner_key:   String,      // The owner of this particular round (enables !stop command)
}

impl Trivia {
    pub fn new() -> Trivia {
        Trivia {
            question: String::new(),
            answer: String::new(),
            running: false,
            rounds: 0,
            hint_count: 0,
            hints: Vec::new(),
            round_timer: Timespec::new(0, 0),
            end_timer: Timespec::new(0, 0),
            winner: false,
            disabled: false,
            owner_key: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.question.clear();
        self.answer.clear();
        self.running = false;
        self.rounds = 0;
        self.hints.clear();
        self.hint_count = 0;
        self.round_timer = Timespec::new(0, 0);
        self.end_timer = Timespec::new(0, 0);
        self.winner = false;
        self.owner_key.clear();
    }

    pub fn new_game(&mut self, owner_key: &str) {
        self.running = true;
        self.owner_key = owner_key.to_string();
    }

    /* Returns true if a new round is successfully set up */
    pub fn new_round(&mut self, questions: &Vec<String>) -> bool {
        self.winner = false;
        self.question.clear();
        self.answer.clear();
        self.hints.clear();

        if !timed_out(self.end_timer, ROUND_DELAY) {
            return false;
        }

        let mut rng = thread_rng();
        let idx = rng.gen_range(0, questions.len());
        let split: Vec<&str> = questions[idx].split("`").collect();

        self.hint_count = 0;
        self.rounds += 1;

        if split.len() < 2 {
            return false;
        }

        self.question = split[0].trim().to_string();
        self.answer = split[1].trim().to_string();
        self.round_timer = get_time();
        self.hints = generate_hints(&self.answer);

        true
    }

    pub fn get_hint(&mut self) -> String {
        if !self.running {
            return "Cram it".to_string();
        }

        let hint = match self.hints.pop() {
            Some(hint) => hint.to_string(),
            None       => "No more hints".to_string(),
        };

        hint
    }

    /* The score is based on how many seconds are left in the round and how many hints have been used */
    fn get_score(&self) -> u64 {
        let delta = Duration::seconds(QUESTION_TIME_LIMIT) - (get_time() - self.round_timer);
        let t = Duration::num_seconds(&delta) + 1;
        let score = (t * BONUS_POINTS_MULTIPLIER / (self.hint_count as i64 + 1)) + BASE_POINTS;
        score as u64
    }
}

fn answer_is_year(answer: &str) -> bool
{
    if answer.len() != 4 {
        return false;
    }

    for ch in answer.chars() {
        if ch < '0' || ch > '9' {
            return false;
        }
    }

    let first_char = answer.chars().nth(0).unwrap();
    first_char == '1' || first_char == '2'
}

/* Creates a vector of hints for the current answer. Hints are ordered by least to most letters revealed. */
fn generate_hints(answer: &str) -> Vec<String>
{
    let mut hints = Vec::new();
    let len = answer.len();

    if len <= 3 {
        return hints;
    }

    // If answer is a year we always give first and third digit
    if answer_is_year(answer) {
        let mut hint = String::new();

        for (i, ch) in answer.chars().enumerate() {
            hint = if i % 2 == 0 { hint + &ch.to_string() } else { hint + "-" };
        }

        return vec![hint];
    }

    let mut used = vec![false; len];  // Marks used indices
    let mut indices = Vec::new();     // Holds unused indices in random order

    // Spaces and punctuation are freebies
    for (i, ch) in answer.chars().enumerate() {
        let p = ch.to_string();

        if PUNCTUATION.contains(&p) {
            used[i] = true;
        } else {
            indices.push(i);
        }
    }

    let mut rng = thread_rng();
    indices.shuffle(&mut rng);

    let chars_per_hint = len / 4;
    let num_hints = ((len / 2) / chars_per_hint) + 1;

    for _ in 0..num_hints {
        let mut hint = String::new();

        for _ in 0..chars_per_hint {
            let idx = indices.pop().unwrap_or(0);
            used[idx] = true;
        }

        for (i, ch) in answer.chars().enumerate() {
            hint = if used[i] { hint + &ch.to_string() } else { hint + "-" };
        }

        hints.push(hint);
    }

    hints.into_iter().rev().collect()
}

pub fn process_answer(bot: &mut Bot, groupnumber: u32, peernumber: u32, message: &str)
{
    let index = match get_group_index(bot, groupnumber) {
        Some(index) => index,
        None => return println!("get_group_index() failed in process_answer() for groupnumber {}", groupnumber),
    };

    if !bot.groups[index].trivia.running {
        return;
    }

    if bot.groups[index].trivia.winner {
        return;
    }

    if message.to_lowercase() != bot.groups[index].trivia.answer.to_lowercase() {
        return;
    }

    let public_key = match get_peer_public_key(bot.tox, groupnumber, peernumber) {
        Some(key) => key,
        None => return println!("get_peer_public_key() failed in process_answer() for groupnumber {}, peernumber {}",
                                     groupnumber, peernumber),
    };

    let peer_idx = match get_peer_index(&mut bot.groups[index].peers, &public_key) {
        Some(idx) => idx,
        None => return println!("get_peer_index() failed in process_answer() for public_key {:?}", public_key.to_string()),
    };

    let points = bot.groups[index].trivia.get_score();
    bot.groups[index].peers[peer_idx].update_round_score(points);
    let score = bot.groups[index].peers[peer_idx].get_round_score();
    let rounds_won = bot.groups[index].peers[peer_idx].get_rounds_won();
    let peername = bot.groups[index].peers[peer_idx].get_nick();

    let mut response = String::new();
    write!(&mut response, "{} got the answer for {} points (Total: {} points, {} rounds)",
           peername, points, score, rounds_won).unwrap();
    bot.groups[index].send_message(bot.tox, &response);

    bot.groups[index].trivia.winner = true;
    bot.groups[index].trivia.end_timer = get_time();
    bot.groups[index].trivia.round_timer = Timespec::new(0, 0);
}

pub fn do_trivia(bot: &mut Bot)
{
    for group in &mut bot.groups {
        if group.trivia.running {
            if timed_out(group.trivia.round_timer, QUESTION_TIME_LIMIT) {
                group.next_trivia_question(bot.tox, &bot.questions, &mut bot.db);
            }
        }
    }
}
