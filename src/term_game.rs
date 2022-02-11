// term_game.rs - main library of sokoban
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

use std::error::Error;

use termion::clear;
use termion::input::TermRead;
use termion::color::*;
use termion::style::*;
use termion::cursor;
use termion::event::{Event,Key};

use crate::defs::*;

use crate::GameResult;
use crate::LevelState;

pub struct TermGame<'a> {
    state: &'a LevelState<'a>,
}

impl<'a> TermGame<'a> {
    pub fn create(ls: &'a LevelState<'a>) -> TermGame<'a> {
        TermGame{ state: ls }
    }
    
    pub fn state(&self) -> &'a LevelState<'a> {
        self.state
    }
    
    fn display_level(&self, x: usize, y: usize) {
    }
    
    fn make_move_fast(&mut self, d: Direction) {
    }
    
    fn make_move_scroll(&mut self, d: Direction) {
    }
    
    fn display_game(&self) {
    }
    
    fn make_move(&mut self, d: Direction) -> bool {
        false
    }
    
    pub fn start(&mut self) -> Result<GameResult, Box<dyn Error>> {
        Ok(GameResult::Solved)
    }
}
