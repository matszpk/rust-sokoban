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

use Field::*;

pub struct TermGame<'a> {
    state: &'a mut LevelState<'a>,
    term_width: usize,
    term_height: usize,
    empty_line: Vec<u8>,
}

// return start display position, start level position, displayed area size
fn determine_display_and_level_position(leveldim: usize, dispdim: usize,
        centered_levelpos: usize) -> (usize, usize, usize) {
    if dispdim >= leveldim {
        // if display dimension is greater han level dimension
        ((dispdim>>1)-(leveldim>>1), 0, leveldim)
    } else {
        // if display dimension is less than level dimension
        if centered_levelpos >= (dispdim>>1) {
            // if position at start is non negative
            if centered_levelpos + (dispdim-(dispdim>>1)) <= leveldim {
                (0, centered_levelpos - (dispdim>>1), dispdim)
            } else { // align to end of level
                (0, leveldim-dispdim, dispdim)
            }
        } else { // align to zero position at start
            (0, 0, dispdim) }
    }
}

impl<'a> TermGame<'a> {
    pub fn create(ls: &'a mut LevelState<'a>) -> TermGame<'a> {
        let (width, height) = terminal_size().unwrap();
        TermGame{ state: ls, term_width: width as usize,
                term_height: height as usize,
                empty_line: vec![b' '; width as usize] }
    }
    
    pub fn state(&'a self) -> &'a LevelState<'a> {
        self.state
    }
    
    // cx, cy - position of level to display at center of the display.
    fn display_level<W: Write>(&self, stdout: &mut W,
                cx: usize, cy: usize) -> Result<(), Box<dyn Error>> {
        
        write!(stdout, "{}{}", cursor::Goto(1, 1), Bg(Black))?;
        let levelw = self.state.level.width();
        let levelh = self.state.level.height();
        // display dimensions
        let dispw = self.term_width;
        let disph = self.term_height-1;
        let (sdx, slx, fdw) = determine_display_and_level_position(levelw, dispw, cx);
        let (sdy, sly, fdh) = determine_display_and_level_position(levelh, disph, cy);
        
        // fill empties
        for dy in [0..sdy] {
            stdout.write(self.empty_line.as_slice())?;
        }
        for dy in sdy..sdy+fdh {
            let state_line = &self.state.area()[(dy-sdy+sly)*levelw + slx..
                        (dy-sdy+sly)*levelw + slx + fdw];
            stdout.write(&self.empty_line.as_slice()[0..sdx])?;
            for dx in sdx..sdx+fdw {
                let fmt_str: String = match state_line[dx-sdx+slx] {
                    Empty => " ".to_string(),
                    Wall => "░".to_string(),
                    Player => "o".to_string(),
                    Pack => "▒".to_string(),
                    Target => format!("{}{}", Bg(Yellow), Bg(Black)),
                    PlayerOnTarget => format!("{}o{}", Bg(Yellow), Bg(Black)),
                    PackOnTarget => format!("{}▒{}", Bg(Yellow), Bg(Black)),
                    _ => { panic!("Unexpected!"); },
                };
                stdout.write(fmt_str.as_bytes())?;
            }
            stdout.write(&self.empty_line.as_slice()[sdx+fdw..dispw])?;
        }
        for dy in [sdy+fdh..disph] {
            stdout.write(self.empty_line.as_slice())?;
        }
        
        Ok(())
    }
    
    fn make_move_fast(&mut self, d: Direction) {
    }
    
    fn undo_move_fast(&mut self) {
    }
    
    fn display_game<W: Write>(&self, stdout: &mut W) {
        
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
        self.display_game(&mut stdout);
        
        for e in std::io::stdin().events() {
        }
        Ok(GameResult::Solved)
    }
}
