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

use std::io as io;

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
    NoPacksAndTarget,
    /// If level open (no closing walls) - place where level is open.
    LevelOpen{x: u32, y: u32},
    /// If too few packs - number of required packs.
    TooFewPacks(u32),
    /// If too few targets - number of required targets.
    TooFewTargets(u32),
    /// If pack is not available for player - place of pack.
    PackNotAvailable{x: u32, y: u32},
    /// If target not available for player - place of target.
    TargetNotAvailable{x: u32, y: u32},
    /// If pack locked apart wall - place of pack.
    LockedPackApartWall{x: u32, y: u32},
    /// If 4 packs creates 2x2 box - place of 2x2 box.
    Locked4Packs{x: u32, y: u32},
}

pub type CheckErrors = Vec<CheckError>;

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
                    -> Level<'a> {
        let area: Vec<Field> = astr.chars().map(|x| {
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
        Level{ name, width, height, area: area, moves: vec!() }
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

pub fn sokhello() {
    println!("SokHello!")
}
