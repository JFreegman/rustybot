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

use std::collections::HashMap;

pub struct DBentry {
    pub nick:       String,   // The last nick this entry is associated with
    pub points:     i64,
    pub rounds_won: i32,
    pub games_won:  i32,
}

impl DBentry {
    /* New entries are created only when a peer wins a round */
    pub fn new(points: i64, nick: &str) -> DBentry {
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
        match self.hashmap.get(key) {
            Some(entry) => Some(DBentry { nick: entry.nick.to_string(), games_won: entry.games_won,
                                          rounds_won: entry.rounds_won, points: entry.points }),
            None => None,
        }
    }

    /* Updates db entry's score for key. A zero value for points indicates a game win. */
    pub fn update_score(&mut self, nick: &str, key: &str, points: i64) {
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
}
