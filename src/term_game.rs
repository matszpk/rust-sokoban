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
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::event::{Event,Key};

use crate::defs::*;

use crate::GameResult;
use crate::LevelState;

use Field::*;
use Direction::*;

/// The game in terminal mode. Structure contains level state and some terminal utilities.
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

fn print_field<W: Write>(stdout: &mut W,f: Field) -> Result<(), Box<dyn Error>> {
    let fmt_str: String = match f {
        Empty => " ".to_string(),
        Wall => "░".to_string(),
        Player => "o".to_string(),
        Pack => "▒".to_string(),
        Target => format!("{} {}", Bg(Yellow), Bg(Black)),
        PlayerOnTarget => format!("{}o{}", Bg(Yellow), Bg(Black)),
        PackOnTarget => format!("{}▒{}", Bg(Yellow), Bg(Black)),
    };
    stdout.write(fmt_str.as_bytes())?;
    Ok(())
}

impl<'a> TermGame<'a> {
    /// Create terminal game.
    pub fn create(ls: &'a mut LevelState<'a>) -> TermGame<'a> {
        let (width, height) = terminal_size().unwrap();
        TermGame{ state: ls, term_width: width as usize,
                term_height: height as usize,
                empty_line: vec![b' '; width as usize] }
    }
    
    /// Get level state.
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
        for _ in 0..sdy {
            stdout.write(self.empty_line.as_slice())?;
        }
        for dy in sdy..sdy+fdh {
            let state_line = &self.state.area()[(dy-sdy+sly)*levelw + slx..
                        (dy-sdy+sly)*levelw + slx + fdw];
            stdout.write(&self.empty_line.as_slice()[0..sdx])?;
            for dx in sdx..sdx+fdw {
                print_field(stdout, state_line[dx-sdx])?;
            }
            stdout.write(&self.empty_line.as_slice()[sdx+fdw..dispw])?;
        }
        for _ in sdy+fdh..disph {
            stdout.write(self.empty_line.as_slice())?;
        }
        // display status bar
        self.display_statusbar(stdout)
    }
    
    fn display_statusbar<W: Write>(&self, stdout: &mut W) ->
                    Result<(), Box<dyn Error>> {
        // display status bar
        write!(stdout, "{}{:<10}  Moves: {:>7}  Pushes: {:>7}",
                cursor::Goto(1, (self.term_height-1+1) as u16),
                self.state.level().name(),
                self.state.moves().len(), self.state.pushes_count())?;
        stdout.flush()?;
        Ok(())
    }
    
    fn display_move_fast<W: Write>(&self, stdout: &mut W,
                player_x: usize, player_y: usize, dir: Direction)
                -> Result<(), Box<dyn Error>> {
        let levelw = self.state.level.width();
        let levelh = self.state.level.height();
        let dispw = self.term_width;
        let disph = self.term_height-1;
        let scx = (dispw>>1)-(levelw>>1);
        let scy = (disph>>1)-(levelh>>1);
        match dir {
            Left|PushLeft|Right|PushRight => {
                write!(stdout, "{}", cursor::Goto((scx+player_x-1+1) as u16,
                    (scy+player_y+1) as u16))?;
                print_field(stdout, self.state.area()[levelw*player_y + player_x-1])?;
                print_field(stdout, self.state.area()[levelw*player_y + player_x])?;
                print_field(stdout, self.state.area()[levelw*player_y + player_x+1])?;
            }
            Up|PushUp|Down|PushDown => {
                write!(stdout, "{}", cursor::Goto((scx+player_x+1) as u16,
                    (scy+player_y-1+1) as u16))?;
                print_field(stdout, self.state.area()[levelw*(player_y-1) + player_x])?;
                write!(stdout, "{}", cursor::Goto((scx+player_x+1) as u16,
                    (scy+player_y+1) as u16))?;
                print_field(stdout, self.state.area()[levelw*(player_y) + player_x])?;
                write!(stdout, "{}", cursor::Goto((scx+player_x+1) as u16,
                    (scy+player_y+1+1) as u16))?;
                print_field(stdout, self.state.area()[levelw*(player_y+1) + player_x])?;
            }
            _ => {}
        };
        self.display_statusbar(stdout)
    }
    
    fn display_game<W: Write>(&self, stdout: &mut W) -> Result<(), Box<dyn Error>> {
        self.display_level(stdout, self.state.player_x, self.state.player_y)
    }
    
    fn display_change<W: Write>(&self, stdout: &mut W,
                player_x: usize, player_y: usize, dir: Direction)
                        -> Result<(), Box<dyn Error>> {
        let levelw = self.state.level.width();
        let levelh = self.state.level.height();
        let dispw = self.term_width;
        let disph = self.term_height-1;
        if levelw < dispw && levelh < disph {
            self.display_move_fast(stdout, player_x, player_y, dir)
        } else {
            self.display_game(stdout)
        }
    }
    
    fn make_move<W: Write>(&mut self, stdout: &mut W, d: Direction) ->
                    Result<bool, Box<dyn Error>> {
        let (mv, _) = self.state.make_move(d);
        if mv { self.display_change(stdout, self.state.player_x, self.state.player_y,
                *self.state.moves().last().unwrap())?; }
        Ok(mv)
    }
    
    fn undo_move<W: Write>(&mut self, stdout: &mut W) ->
                    Result<bool, Box<dyn Error>> {
        let old_player_x = self.state.player_x;
        let old_player_y = self.state.player_y;
        if let Some(l) = self.state.moves().last() {
            let last_dir = *l;
            self.state.undo_move();
            self.display_change(stdout, old_player_x, old_player_y, last_dir)?;
            Ok(true)
        } else { Ok(false) }
    }
    
    /// Start game in terminal.
    pub fn start(&mut self) -> Result<GameResult, Box<dyn Error>> {
        let mut stdout = io::stdout().into_raw_mode()?;
        
        write!(stdout, "{}{}{}{}", Bg(Black), Fg(White), clear::All,
                    cursor::Goto(1, 1))?;
        stdout.flush()?;
        
        let mut stdout = cursor::HideCursor::from(stdout);
        self.state.reset();
        self.display_game(&mut stdout)?;
        
        
        for e in std::io::stdin().events() {
            if self.state.is_done() { break; }
            match e {
                Ok(Event::Key(k)) => {
                    match k {
                        Key::Left => { self.make_move(&mut stdout, Left)?; }
                        Key::Right => { self.make_move(&mut stdout, Right)?; }
                        Key::Up => { self.make_move(&mut stdout, Up)?; }
                        Key::Down => { self.make_move(&mut stdout, Down)?; }
                        Key::Backspace => { self.undo_move(&mut stdout)?; }
                        Key::Esc => { return Ok(GameResult::Canceled); }
                        Key::Char('q') => { return Ok(GameResult::Canceled); }
                        _ => {},
                    };
                }
                _ => ()
            };
            if self.state.is_done() { break; }
        }
        Ok(GameResult::Solved)
    }
}
