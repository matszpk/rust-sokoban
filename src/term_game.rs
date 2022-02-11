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
use std::io;
use std::io::Write;

use termion::terminal_size;
use termion::clear;
use termion::input::TermRead;
use termion::color::*;
use termion::style::*;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::event::{Event,Key};

use crate::defs::*;

use crate::GameResult;
use crate::LevelState;

pub struct TermGame<'a> {
    state: &'a mut LevelState<'a>,
    term_width: usize,
    term_height: usize,
}

impl<'a> TermGame<'a> {
    pub fn create(ls: &'a mut LevelState<'a>) -> TermGame<'a> {
        let (width, height) = terminal_size().unwrap();
        TermGame{ state: ls, term_width: width as usize, term_height: height as usize }
    }
    
    pub fn state(&'a self) -> &'a LevelState<'a> {
        self.state
    }
    
    // cx, cy - position of level to display at center of the display.
    fn display_level(&self, cx: usize, cy: usize) -> Result<(), Box<dyn Error>> {
        let levelw = self.state.level.width();
        let levelh = self.state.level.height();
        // display dimensions
        let dispw = self.term_width;
        let disph = self.term_height-1;
        let (sdx, slx) = if dispw < levelw {
            if cx > (dispw>>1) { (0, cx-(dispw>>1)) } else { (0, 0) }
            } else { ((dispw>>1)-(levelw>>1), 0) };
        let (sdy, sly) = if disph < levelh {
            if cy > (disph>>1) { (0, cy-(disph>>1)) } else { (0, 0) }
            } else { ((disph>>1)-(levelh>>1), 0) };
        
        for dy in [0..disph] {
        }
        
        Ok(())
    }
    
    fn make_move_fast(&mut self, d: Direction) {
    }
    
    fn undo_move_fast(&mut self) {
    }
    
    fn display_game(&self) {
        
    }
    
    fn make_move(&mut self, d: Direction) -> bool {
        false
    }
    
    fn undo_move(&mut self) -> bool {
        false
    }
    
    pub fn start(&mut self) -> Result<GameResult, Box<dyn Error>> {
        let stdin = io::stdin();
        let mut stdout = io::stdout().into_raw_mode()?;
        
        write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1))?;
        stdout.flush()?;
        
        self.state.reset();
        self.display_game();
        
        for e in std::io::stdin().events() {
        }
        Ok(GameResult::Solved)
    }
}
