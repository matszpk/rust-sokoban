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

use std::io;
use std::env;
use sokobanlib::*;
use termion::raw::IntoRawMode;
use termion::cursor;

fn main() {
    let mut args = env::args();
    if args.len() < 2 {
        panic!("No file");
    }
    args.next();
    let levelset_path = args.next().unwrap();
    let levelset = LevelSet::from_file(levelset_path).unwrap();
    let stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdout = cursor::HideCursor::from(stdout);
    let mut term_levelset = TermLevelSet::create(&mut stdout, &levelset);
    term_levelset.start().unwrap();
}
