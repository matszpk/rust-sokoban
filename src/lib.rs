// lib.rs - main library of sokoban
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
use std::fmt;

/// Type represents direction of the move.
pub enum Direction {
    /// Move left.
    Left,
    /// Move right.
    Right,
    /// Move up.
    Up,
    /// Move down.
    Down,
    /// Move and push left.
    PushLeft,
    /// Move and push right.
    PushRight,
    /// Move and push up.
    PushUp,
    /// Move and push down.
    PushDown,
}

/// Type represents field in level area.
pub enum Field {
    /// Empty field.
    Empty,
    /// Wall.
    Wall,
    /// Box to move to target.
    Pack,
    /// Player.
    Player,
    /// Empty target.
    Target,
    /// Box in target.
    PackInTarget,
    /// Player in target.
    PlayerInTarget,
}

// Check level error.
pub enum CheckError {
    /// No player.
    NoPlayer,
    /// No packs and targets.
    NoPacksAndTargets,
    /// If level open (no closing walls) - place where level is open.
    LevelOpen(u32, u32),
    /// If too few packs - number of required packs.
    TooFewPacks(u32),
    /// If too few targets - number of required targets.
    TooFewTargets(u32),
    /// If pack is not available for player - place of pack.
    PackNotAvailable(u32, u32),
    /// If target not available for player - place of target.
    TargetNotAvailable(u32, u32),
    /// If pack locked apart wall - place of pack.
    LockedPackApartWall(u32, u32),
    /// If 4 packs creates 2x2 box - place of 2x2 box.
    Locked4Packs(u32, u32),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::NoPlayer => write!(f, "No player"),
            CheckError::NoPacksAndTargets => write!(f, "No packs and targets"),
            CheckError::LevelOpen(x, y) => write!(f, "Level open in {}x{}", x, y),
            CheckError::TooFewPacks(x) => write!(f, "Too few packs - required {}", x),
            CheckError::TooFewTargets(x) => write!(f, "Too few targets - required {}", x),
            CheckError::PackNotAvailable(x, y) =>
                write!(f, "Pack {}x{} not available", x, y),
            CheckError::TargetNotAvailable(x, y) =>
                write!(f, "Target {}x{} not available", x, y),
            CheckError::LockedPackApartWall(x, y) =>
                write!(f, "Locked pack {}x{} apart wall", x, y),
            CheckError::Locked4Packs(x, y) =>
                write!(f, "Locked 4 packs {}x{}", x, y),
        }
    }
}

pub struct CheckErrors(Vec<CheckError>);

impl fmt::Display for CheckErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().fold(Ok(()), |r,x| r.and(write!(f, "{}. ", x)))
    }
}

impl CheckErrors {
    fn new() -> CheckErrors {
        CheckErrors(Vec::new())
    }
    fn push(& mut self, e: CheckError) {
        self.0.push(e)
    }
}

pub enum ParseError {
    EmptyLines,
    WrongField{x: u32, y: u32},
}

/// Level in game. Name is optional name - can be empty. Width and height determines
/// dimensions of the level. An area is fields of level ordered from top to bottom and
/// from left to right.
pub struct Level<'a> {
    name: &'a str,
    width: u32,
    height: u32,
    area: Vec<Field>,
    moves: Vec<Direction>,
}

impl<'a> Level<'a> {
    /// Get name of the level.
    pub fn name(&self) -> &'a str {
        self.name
    }
    /// Get width of the level.
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Get height of the level.
    pub fn height(&self) -> u32 {
        self.height
    }
    /// Get an area of the level.
    pub fn area(&self) -> &Vec<Field> {
        &self.area
    }
    
    pub fn new(name: &'a str, width: u32, height: u32, area: Vec<Field>) -> Level<'a> {
        Level{ name, width, height, area, moves: vec!() }
    }
    
    pub fn from_string(name: &'a str, width: u32, height: u32, astr: &str)
                    -> Result<Level<'a>, ParseError> {
        let mut chrs = astr.chars().take(width as usize*height as usize);
        let chrs2 = chrs.clone();
        if let Some(p) = chrs.position(|x|
                x!=' ' && x!='#' && x!='@' && x!='+' && x!='.' && x!='$' && x!='*' ) {
            let pp = p as u32;
            return Err(ParseError::WrongField{ x: pp%width, y: pp/width });
        }
        let area: Vec<Field> = chrs2.map(|x| {
            match x {
                ' ' => Field::Empty,
                '#' => Field::Wall,
                '@' => Field::Player,
                '+' => Field::PlayerInTarget,
                '.' => Field::Target,
                '$' => Field::Pack,
                '*' => Field::PackInTarget,
                _ => Field::Empty,
            }
        }).collect();
        Ok(Level{ name, width, height, area: area, moves: vec!() })
    }
    
    /// Parse level from lines.
    pub fn from_file(name: &'a str, reader: &dyn io::Read) ->
                    Result<Level<'a>, ParseError> {
        Err(ParseError::EmptyLines)
    }
    
    /// Check level.
    pub fn check(&self) -> Result<(), CheckErrors> {
        Ok(())
    }
    
    /// Reset level state to original state - undo all moves.
    pub fn reset(&mut self) {
    }
    
    /// Check level is done.
    pub fn is_done(&self) -> bool {
        false
    }
    
    /// Make move if possible. Return 2 booleans.
    /// The first boolean indicates that move has been done.
    /// The second boolean indicates that move push pack.
    pub fn make_move(&mut self, dir: Direction) -> (bool, bool) {
        (true, true)
    }
    
    /// Undo move. Return true if move undone.
    pub fn undo_move(&mut self) -> bool {
        false
    }
    
    /// Get all moves.
    pub fn moves(&self) -> &Vec<Direction> {
        &self.moves
    }
}

#[cfg(test)]
mod test {
}

pub fn sokhello() {
    let mut errors = CheckErrors::new();
    errors.push(CheckError::LockedPackApartWall(4, 5));
    errors.push(CheckError::Locked4Packs(7, 7));
    println!("SokHello! {}", errors)
}
