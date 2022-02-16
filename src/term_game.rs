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

use std::io;
use std::io::Write;

use termion::terminal_size;
use termion::clear;
use termion::input::TermRead;
use termion::color::*;
use termion::cursor;
use termion::event::Key;

use crate::defs::*;

use crate::GameResult;
use crate::{LevelState,LevelSet};

use Field::*;
use Direction::*;

/// The levelset game in terminal mode.
pub struct TermLevelSet<'a, W: Write> {
    levelset: &'a LevelSet,
    stdout: &'a mut W,
    term_width: usize,
    term_height: usize,
}

fn display_message<W: Write>(term_width: usize, term_height: usize, stdout: &mut W,
                    text: &str) -> io::Result<()> {
    let mut lines = vec![];
    let mut i = 0;
    let maxlen = term_width-4;
    let textb = text.as_bytes();
    loop {
        let mut next_line = i+maxlen;
        
        if next_line < text.len() {
            if let Some(pos) = text[i..next_line].find('\n') {
                next_line = i + pos+1;
                lines.push(&text[i..i+pos]);
            } else if let Some(pos) = text[i..next_line].rfind(
                        |c| c==' ' || c=='.' || c==';' || c==',' || c=='\t') {
                next_line = i+pos+1;
                let mut p = pos;
                while p>1 && (textb[i+p]==b' ' || textb[i+p]==b'\n' ||
                        textb[i+p]==b'\t') {
                    p-=1;
                }
                lines.push(&text[i..i+p+1]);
            }
            i = next_line;
        } else { // push last line
            if let Some(pos) = text[i..].find('\n') {
                lines.push(&text[i..i+pos]);
                i += pos+1;
                if i >= text.len() { break; }
            } else {
                lines.push(&text[i..]);
                break;
            }
        }
    }
    let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or_default();
    let startx = (term_width - max_line_len - 4)>>1;
    let starty = (term_height - lines.len() - 4)>>1;
    
    // draw message
    // prepare lines
    let mut horiz_line = String::new();
    for _ in 0..max_line_len+2 {
        horiz_line.push('─');
    }
    let mut empty_line = String::new();
    empty_line += "│";
    for _ in 0..max_line_len+2 {
        empty_line.push(' ');
    }
    empty_line += "│";
    
    write!(stdout, "{}┌", cursor::Goto((startx+1) as u16, (starty+1) as u16))?;
    stdout.write(horiz_line.as_bytes())?;
    write!(stdout, "┐{}", cursor::Goto((startx+1) as u16, (starty+1+1) as u16))?;
    stdout.write(empty_line.as_bytes())?;
    
    for i in 0..lines.len() {
        let l = lines[i];
        write!(stdout, "{}│ ", cursor::Goto((startx+1) as u16,
                        (starty+i+2+1) as u16))?;
        write!(stdout, "{:^width$}", l, width=max_line_len)?;
        write!(stdout, " │")?;
    }
    
    write!(stdout, "{}", cursor::Goto((startx+1) as u16,
                    (starty+2+lines.len()+1) as u16))?;
    stdout.write(empty_line.as_bytes())?;
    write!(stdout, "{}└", cursor::Goto((startx+1) as u16,
                    (starty+3+lines.len()+1) as u16))?;
    stdout.write(horiz_line.as_bytes())?;
    stdout.write("┘".as_bytes())?;
    stdout.flush()?;
    
    // wait for key.
    if let Some(e) = std::io::stdin().keys().next() { e?; }
    
    Ok(())
}

impl<'a, W: Write> TermLevelSet<'a, W> {
    /// Create terminal levelset game.
    pub fn create(stdout: &'a mut W,
                    levelset: &'a LevelSet) -> TermLevelSet<'a, W> {
        let (width, height) = terminal_size().unwrap();
        TermLevelSet{ levelset, stdout, term_width: width as usize,
                term_height: height as usize }
    }
    
    /// Start game in terminal.
    pub fn start(&mut self) -> io::Result<()> {
        write!(self.stdout, "{}{}{}{}", Bg(Black), Fg(White), clear::All,
                    cursor::Goto(1, 1))?;
        self.stdout.flush()?;
        
        for l in self.levelset.levels() {
            if let Ok(ref level) = l {
                match LevelState::new(level) {
                    Ok(mut ls) => {
                        let gr = TermGame::create(self.stdout, &mut ls).start()?;
                        match gr {
                            GameResult::Solved => 
                                { display_message(self.term_width, self.term_height,
                                        self.stdout, "Level has been solved.")?; }
                            GameResult::Canceled =>
                                { display_message(self.term_width,  self.term_height,
                                        self.stdout, "Level has been canceled.")?; }
                            GameResult::Quit => { 
                                    display_message(self.term_width, self.term_height,
                                        self.stdout, "Quit.")?;
                                    break;
                                }
                        }
                    },
                    Err(err) => {
                        display_message(self.term_width, self.term_height,
                                    self.stdout, format!("Level '{}' have errors: {}",
                                    level.name(), err).as_str())?;
                    }
                }
            }
        }
        
        write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1))?;
        Ok(())
    }
}

/// The game in terminal mode. Structure contains level state and some terminal utilities.
pub struct TermGame<'a, W: Write> {
    state: &'a mut LevelState<'a>,
    stdout: &'a mut W,
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

impl<'a, W: Write> TermGame<'a, W> {
    /// Create terminal game.
    pub fn create(stdout: &'a mut W, ls: &'a mut LevelState<'a>) -> TermGame<'a, W> {
        let (width, height) = terminal_size().unwrap();
        TermGame{ state: ls, stdout, term_width: width as usize,
                term_height: height as usize,
                empty_line: vec![b' '; width as usize] }
    }
    
    /// Get level state.
    pub fn state(&'a self) -> &'a LevelState<'a> {
        self.state
    }
    
    fn print_field(&mut self, f: Field) -> io::Result<()> {
        let fmt_str: String = match f {
            Empty => " ".to_string(),
            Wall => "░".to_string(),
            Player => "o".to_string(),
            Pack => "▒".to_string(),
            Target => format!("{} {}", Bg(Yellow), Bg(Black)),
            PlayerOnTarget => format!("{}o{}", Bg(Yellow), Bg(Black)),
            PackOnTarget => format!("{}▒{}", Bg(Yellow), Bg(Black)),
        };
        self.stdout.write(fmt_str.as_bytes())?;
        Ok(())
    }
    
    // cx, cy - position of level to display at center of the display.
    fn display_level(&mut self, cx: usize, cy: usize) -> io::Result<()> {
        write!(self.stdout, "{}{}", cursor::Goto(1, 1), Bg(Black))?;
        let levelw = self.state.level.width();
        let levelh = self.state.level.height();
        // display dimensions
        let dispw = self.term_width;
        let disph = self.term_height-1;
        let (sdx, slx, fdw) = determine_display_and_level_position(levelw, dispw, cx);
        let (sdy, sly, fdh) = determine_display_and_level_position(levelh, disph, cy);
        
        // fill empties
        for _ in 0..sdy {
            self.stdout.write(self.empty_line.as_slice())?;
        }
        for dy in sdy..sdy+fdh {
            self.stdout.write(&self.empty_line.as_slice()[0..sdx])?;
            for dx in sdx..sdx+fdw {
                self.print_field(self.state.area()[(dy-sdy+sly)*levelw + slx + dx - sdx])?;
            }
            self.stdout.write(&self.empty_line.as_slice()[sdx+fdw..dispw])?;
        }
        for _ in sdy+fdh..disph {
            self.stdout.write(self.empty_line.as_slice())?;
        }
        // display status bar
        self.display_statusbar()
    }
    
    fn display_statusbar(&mut self) -> io::Result<()> {
        // display status bar
        write!(self.stdout, "{}{:<10}  Moves: {:>7}  Pushes: {:>7}",
                cursor::Goto(1, (self.term_height-1+1) as u16),
                self.state.level().name(),
                self.state.moves().len(), self.state.pushes_count())?;
        self.stdout.flush()?;
        Ok(())
    }
    
    fn display_move_fast(&mut self, player_x: usize, player_y: usize,
                    dir: Direction) -> io::Result<()> {
        let levelw = self.state.level.width();
        let levelh = self.state.level.height();
        let dispw = self.term_width;
        let disph = self.term_height-1;
        let scx = (dispw>>1)-(levelw>>1);
        let scy = (disph>>1)-(levelh>>1);
        match dir {
            Left|PushLeft|Right|PushRight => {
                write!(self.stdout, "{}", cursor::Goto((scx+player_x-1+1) as u16,
                    (scy+player_y+1) as u16))?;
                self.print_field(self.state.area()[levelw*player_y + player_x-1])?;
                self.print_field(self.state.area()[levelw*player_y + player_x])?;
                self.print_field(self.state.area()[levelw*player_y + player_x+1])?;
            }
            Up|PushUp|Down|PushDown => {
                write!(self.stdout, "{}", cursor::Goto((scx+player_x+1) as u16,
                    (scy+player_y-1+1) as u16))?;
                self.print_field(self.state.area()[levelw*(player_y-1) + player_x])?;
                write!(self.stdout, "{}", cursor::Goto((scx+player_x+1) as u16,
                    (scy+player_y+1) as u16))?;
                self.print_field(self.state.area()[levelw*(player_y) + player_x])?;
                write!(self.stdout, "{}", cursor::Goto((scx+player_x+1) as u16,
                    (scy+player_y+1+1) as u16))?;
                self.print_field(self.state.area()[levelw*(player_y+1) + player_x])?;
            }
            _ => {}
        };
        self.display_statusbar()
    }
    
    fn display_game(&mut self) -> io::Result<()> {
        self.display_level(self.state.player_x, self.state.player_y)
    }
    
    fn display_change(&mut self, player_x: usize, player_y: usize,
                        dir: Direction) -> io::Result<()> {
        let levelw = self.state.level.width();
        let levelh = self.state.level.height();
        let dispw = self.term_width;
        let disph = self.term_height-1;
        if levelw < dispw && levelh < disph {
            self.display_move_fast(player_x, player_y, dir)
        } else {
            self.display_game()
        }
    }
    
    fn make_move(&mut self, d: Direction) -> io::Result<bool> {
        let (mv, _) = self.state.make_move(d);
        if mv { self.display_change(self.state.player_x, self.state.player_y,
                *self.state.moves().last().unwrap())?; }
        Ok(mv)
    }
    
    fn undo_move(&mut self) -> io::Result<bool> {
        let old_player_x = self.state.player_x;
        let old_player_y = self.state.player_y;
        if let Some(l) = self.state.moves().last() {
            let last_dir = *l;
            self.state.undo_move();
            self.display_change(old_player_x, old_player_y, last_dir)?;
            Ok(true)
        } else { Ok(false) }
    }
    
    /// Start game in terminal.
    pub fn start(&mut self) -> io::Result<GameResult> {
        write!(self.stdout, "{}{}{}{}", Bg(Black), Fg(White), clear::All,
                    cursor::Goto(1, 1))?;
        self.stdout.flush()?;
        
        self.state.reset();
        self.display_game()?;
        
        if !self.state.is_done() {
            for e in std::io::stdin().keys() {
                match e? {
                    Key::F(1) | Key::Char('?') => {
                        display_message(self.term_width, self.term_height, self.stdout,
                                "Keys in game:\n\
                                 Left, Right, Up, Down - move player.\n\
                                 Backspace - undo move.\n\
                                 Escape - cancel current level.\n\
                                 Q - Quit game.\n\
                                 F1, ? - display help.")?;
                        self.display_game()?;
                        }
                    Key::Left => { self.make_move(Left)?; }
                    Key::Right => { self.make_move(Right)?; }
                    Key::Up => { self.make_move(Up)?; }
                    Key::Down => { self.make_move(Down)?; }
                    Key::Backspace => { self.undo_move()?; }
                    Key::Esc => { return Ok(GameResult::Canceled); }
                    Key::Char('q') => { return Ok(GameResult::Quit); }
                    _ => {},
                };
                if self.state.is_done() { break; }
            }
        }
        Ok(GameResult::Solved)
    }
}
