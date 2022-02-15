// main.rs - sokoban game executable
//
// sokoban - Sokoban game
// Copyright (C) 2022  Mateusz Szpakowski
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA

use std::env;
use sokobanlib::*;

fn main() {
    let mut args = env::args();
    if args.len() < 3 {
        println!("No file and level");
    }
    args.next();
    let levelset_path = args.next().unwrap();
    let levelset_index: usize = args.next().unwrap().parse().unwrap();
    println!("Levelset: {}, Level: {}", levelset_path, levelset_index);
    let levelset = LevelSet::from_file(levelset_path).unwrap();
    let levels = levelset.levels();
    if levelset_index >= levels.len() {
        panic!("Beyond levels number");
    }
    if let Ok(ref level) = levels[levelset_index] {
        let mut level_state = LevelState::new(&level).unwrap();
        let mut term_game = TermGame::create(&mut level_state);
        term_game.start().unwrap();
    }
}
