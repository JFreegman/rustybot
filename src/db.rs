/*  db.rs
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
use std::io::BufReader;
use std::collections::HashMap;
use std::str::from_utf8;
use util::*;
use rstox::core::*;

const DATABASE_PATH: &'static str = "data/scores.db";

// Fixed nick size for serialization
const DB_NICK_SIZE: usize = 32;

// Size of the database keys
const DB_KEY_SIZE: usize = (PUBLIC_KEY_SIZE * 2);

// Number of bytes in a database entry in serialized form.
// Key, nick length, nick, points, rounds won, games won
const DB_ENTRY_FORMAT_SIZE: usize = (DB_KEY_SIZE + SIZE_U32 + DB_NICK_SIZE + SIZE_U64 + SIZE_U32 + SIZE_U32);

pub struct DBentry {
    pub nick:       String,   // The last nick this entry is associated with
    pub points:     u64,
    pub rounds_won: u32,
    pub games_won:  u32,
}

impl DBentry {
    /* New entries are created only when a peer wins a round */
    pub fn new(points: u64, nick: &str) -> DBentry {
        DBentry {
            nick: nick.to_string(),
            points: points,
            rounds_won: 1,
            games_won: 0,
        }
    }
}

pub struct DataBase {
    hashmap: HashMap<String, DBentry>,
}

impl DataBase {
    pub fn new() -> DataBase {
        DataBase { hashmap: HashMap::new() }
    }

    pub fn set_nick(&mut self, nick: &str, key: &str) {
        if let Some(entry) = self.hashmap.get_mut(key) {
            entry.nick = nick.to_string();
        }
    }

    /*
     * Returns a vector of all database values sorted by points in descending order.
     * Note: This is very inefficient for a large database but fuck it YOLO
     */
    pub fn get_sorted_values(&self) -> Vec<&DBentry> {
        let mut list = Vec::new();

        for v in self.hashmap.values() {
            list.push(v);
        }

        list.sort_by(|a, b| a.points.cmp(&b.points).reverse());
        list
    }

    /* Returns a DBentry for a given key if it exists. */
    pub fn get_entry(&self, key: &str) -> Option<DBentry> {
        self.hashmap.get(key).map(|e| DBentry { nick: e.nick.to_string(),
                                                games_won: e.games_won,
                                                rounds_won: e.rounds_won,
                                                points: e.points
                                              })
    }

    /* Updates db entry's score for key. A zero value for points indicates a game win. */
    pub fn update_score(&mut self, nick: &str, key: &str, points: u64) {
        if let Some(entry) = self.hashmap.get_mut(key) {
            if points != 0 {
                entry.points += points;
                entry.rounds_won += 1;
            } else {
                entry.games_won += 1;
            }

            return;
        };

        self.hashmap.insert(key.to_string(), DBentry::new(points, nick));
    }

    pub fn save(&self) {
        if self.hashmap.is_empty() {
            return;
        }

        let mut data: Vec<u8> = Vec::new();

        for (key, val) in self.hashmap.iter() {
            string_to_nbytes(&key, &mut data, DB_KEY_SIZE);
            u32_to_bytes_le(val.nick.len() as u32, &mut data);
            string_to_nbytes(&val.nick, &mut data, DB_NICK_SIZE);
            u64_to_bytes_le(val.points, &mut data);
            u32_to_bytes_le(val.rounds_won, &mut data);
            u32_to_bytes_le(val.games_won, &mut data);
        }

        save_data(DATABASE_PATH, &data);
    }

    pub fn load(&mut self) {
        let fp = match open_file(DATABASE_PATH, true) {
            Some(fp) => fp,
            None   => return,
        };

        let mut buf: Vec<u8> = Vec::new();
        let mut reader = BufReader::new(&fp);

        let size = match reader.read_to_end(&mut buf) {
            Ok(size) => size,
            Err(e) => return println!("Failed to read database to buffer: {}", Error::description(&e)),
        };

        if size == 0 {
            return;
        }

        if size % DB_ENTRY_FORMAT_SIZE != 0 {
            return println!("Failed to load trivia database: Bad format.");
        }

        let num = size / DB_ENTRY_FORMAT_SIZE;

        for i in 0..num {
            let mut start = i * DB_ENTRY_FORMAT_SIZE;
            let mut end = start + DB_KEY_SIZE;
            let utf8_key = &buf[start..end];

            // Get key
            let key = match from_utf8(utf8_key) {
                Ok(key) => key,
                Err(_)  => continue,
            };

            // Get nick len
            start = end;
            end = end + SIZE_U32;
            let nick_len = bytes_le_to_u32(&buf[start..end]) as usize;

            if nick_len > DB_NICK_SIZE {
                continue;
            }

            // Get nick
            start = end;
            end = end + nick_len;
            let utf8_nick = &buf[start..end];

            let nick = match from_utf8(utf8_nick) {
                Ok(nick) => nick,
                Err(_) => continue,
            };

            // Skip nick padding
            if nick_len < DB_NICK_SIZE {
                let padding = DB_NICK_SIZE - nick_len;
                start = start + padding;
                end = end + padding;
            }

            // Get points
            start = end;
            end = end + SIZE_U64;
            let points = bytes_le_to_u64(&buf[start..end]);
            // Get rounds won
            start = end;
            end = end + SIZE_U32;
            let rounds_won = bytes_le_to_u32(&buf[start..end]);
            // Get games won
            start = end;
            end = end + SIZE_U32;
            let games_won = bytes_le_to_u32(&buf[start..end]);

            let entry = DBentry { nick: nick.to_string(),
                                  points: points,
                                  rounds_won: rounds_won,
                                  games_won: games_won
                                };

            self.hashmap.insert(key.to_string(), entry);
        }
    }
}
