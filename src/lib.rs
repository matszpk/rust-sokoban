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
#[derive(PartialEq,Debug)]
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
#[derive(PartialEq,Debug)]
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
    /// Box on target.
    PackOnTarget,
    /// Player on target.
    PlayerOnTarget,
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

use Field::*;
use Direction::*;
use CheckError::*;

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoPlayer => write!(f, "No player"),
            NoPacksAndTargets => write!(f, "No packs and targets"),
            LevelOpen(x, y) => write!(f, "Level open in {}x{}", x, y),
            TooFewPacks(x) => write!(f, "Too few packs - required {}", x),
            TooFewTargets(x) => write!(f, "Too few targets - required {}", x),
            PackNotAvailable(x, y) => write!(f, "Pack {}x{} not available", x, y),
            TargetNotAvailable(x, y) => write!(f, "Target {}x{} not available", x, y),
            LockedPackApartWall(x, y) => write!(f, "Locked pack {}x{} apart wall", x, y),
            Locked4Packs(x, y) => write!(f, "Locked 4 packs {}x{}", x, y),
        }
    }
}

pub struct CheckErrors(Vec<CheckError>);

impl fmt::Display for CheckErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().take(self.0.len()-1).fold(Ok(()),
                |r,x| r.and(write!(f, "{}. ", x)))?;
        if let Some(x) = self.0.last() {
            write!(f, "{}.", x)
        } else { Ok(()) }
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

#[derive(PartialEq,Debug)]
pub enum ParseError {
    EmptyLines,
    WrongField(u32, u32),
    WrongSize(u32, u32),
}

use ParseError::*;

/// Level in game. Name is optional name - can be empty. Width and height determines
/// dimensions of the level. An area is fields of level ordered from top to bottom and
/// from left to right.
#[derive(PartialEq,Debug)]
pub struct Level<'a> {
    name: &'a str,
    width: u32,
    height: u32,
    area: Vec<Field>,
    moves: Vec<Direction>,
}

fn char_to_field(x: char) -> Field {
    match x {
        ' ' => Empty,
        '#' => Wall,
        '@' => Player,
        '+' => PlayerOnTarget,
        '.' => Target,
        '$' => Pack,
        '*' => PackOnTarget,
        _ => Empty,
    }
}

fn is_not_field(x: char) -> bool {
    x!=' ' && x!='#' && x!='@' && x!='+' && x!='.' && x!='$' && x!='*'
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
        if astr.len() != (width as usize)*(height as usize) {
            return Err(WrongSize(width, height));
        }
        let mut chrs = astr.chars();
        let chrs2 = chrs.clone();
        if let Some(p) = chrs.position(is_not_field) {
            let pp = p as u32;
            return Err(WrongField(pp%width, pp/width));
        }
        let area: Vec<Field> = chrs2.map(char_to_field).collect();
        Ok(Level{ name, width, height, area: area, moves: vec!() })
    }
    
    /// Parse level from lines.
    pub fn from_lines<B>(reader: &io::Lines::<B>) ->
                    Result<Level<'a>, ParseError> {
        Err(EmptyLines)
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
    use super::*;
    
    #[test]
    fn test_level_from_string() {
        let levela = Level::new("blable", 5, 3, vec![
            Wall, Wall, Wall, Wall, Wall,
            Wall, Target, Pack, Player, Wall,
            Wall, Wall, Wall, Wall, Wall]);
        let levelb = Level::from_string("blable", 5, 3, "######.$@######");
        assert_eq!(Ok(levela), levelb);
        
        let levela = Level::new("git", 8, 6, vec![
            Empty, Wall, Wall, Wall, Wall, Wall, Wall, Empty,
            Wall, Empty, Empty, Empty, Empty, Empty, Empty, Wall,
            Wall, Player, Empty, Empty, Target, Target, Target, Wall,
            Wall, Empty, Empty, Empty, Pack, Pack, Pack, Wall,
            Wall, Empty, Empty, Empty, Empty, Empty, Empty, Wall,
            Empty, Wall, Wall, Wall, Wall, Wall, Wall, Empty]);
        let levelb = Level::from_string("git", 8, 6,
            " ###### \
             #      #\
             #@  ...#\
             #   $$$#\
             #      # \
              ###### ");
        assert_eq!(Ok(levela), levelb);
        
        let levela = Level::new("git", 8, 6, vec![
            Empty, Wall, Wall, Wall, Wall, Wall, Wall, Empty,
            Wall, Empty, Empty, Empty, Empty, Empty, Empty, Wall,
            Wall, Empty, Empty, Empty, PlayerOnTarget, Target, PackOnTarget, Wall,
            Wall, Empty, Empty, Empty, Pack, Pack, Empty, Wall,
            Wall, Empty, Empty, Empty, Empty, Empty, Empty, Wall,
            Empty, Wall, Wall, Wall, Wall, Wall, Wall, Empty]);
        let levelb = Level::from_string("git", 8, 6,
            " ###### \
             #      #\
             #   +.*#\
             #   $$ #\
             #      # \
              ###### ");
        assert_eq!(Ok(levela), levelb);
        
        let levelb = Level::from_string("git", 8, 6,
            " ###### \
             #      #\
             #   +.*#\
             #   $$ #\
             #  x   # \
              ###### ");
        assert_eq!(Err(WrongField(3,4)), levelb);
        let levelb = Level::from_string("git", 8, 7,
            " ###### \
             #      #\
             #   +.*#\
             #   $$ #\
             #      # \
              ###### ");
        assert_eq!(Err(WrongSize(8,7)), levelb);
    }
}

pub fn sokhello() {
    let mut errors = CheckErrors::new();
    errors.push(CheckError::LockedPackApartWall(4, 5));
    errors.push(CheckError::Locked4Packs(7, 7));
    println!("SokHello! {}x", errors)
}
