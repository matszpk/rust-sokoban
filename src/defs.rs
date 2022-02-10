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

use std::error::Error;
use std::fmt;
use int_enum::IntEnum;

/// Type represents direction of the move.
#[repr(u8)]
#[derive(PartialEq,Eq,Debug,Clone,Copy,IntEnum)]
pub enum Direction {
    /// Move left.
    Left = 0,
    /// Move right.
    Right = 1,
    /// Move up.
    Up = 2,
    /// Move down.
    Down = 3,
    /// Move and push left.
    PushLeft = 4,
    /// Move and push right.
    PushRight = 5,
    /// Move and push up.
    PushUp = 6,
    /// Move and push down.
    PushDown = 7,
    // No direction.
    NoDirection = 8,
}

/// Type represents field in level area.
#[repr(u8)]
#[derive(PartialEq,Eq,Debug,Clone,Copy,IntEnum)]
pub enum Field {
    /// Empty field.
    Empty = 0,
    /// Wall.
    Wall = 1,
    /// Box to move to target.
    Pack = 2,
    /// Player.
    Player = 3,
    /// Empty target.
    Target = 4,
    /// Box on target.
    PackOnTarget = 5,
    /// Player on target.
    PlayerOnTarget = 6,
}

#[derive(Debug,PartialEq,Eq)]
/// Check level error.
pub enum CheckError {
    /// No player.
    NoPlayer,
    /// Too many players.
    TooManyPlayers,
    /// No packs and targets.
    NoPacksAndTargets,
    /// If level open (no closing walls).
    LevelOpen,
    /// If too few packs - number of required packs.
    TooFewPacks(usize),
    /// If too few targets - number of required targets.
    TooFewTargets(usize),
    /// If pack is not available for player - place of pack.
    PackNotAvailable(usize, usize),
    /// If target not available for player - place of target.
    TargetNotAvailable(usize, usize),
    /// If pack locked apart wall - place of pack.
    LockedPackApartWalls(usize, usize),
    /// If walls and packs creates 2x2 block - place of 2x2 block.
    Locked2x2Block(usize, usize),
}

#[derive(PartialEq,Eq,Debug)]
/// Error caused while parsing or creating level.
pub enum ParseError {
    /// If empty lines.
    EmptyLines,
    /// If wrong field.
    WrongField(usize, usize),
    /// If wrong size.
    WrongSize(usize, usize),
}

/// Parse error concerned XML structure.
#[derive(PartialEq,Eq,Debug)]
pub enum XmlParseError {
    /// If bad structure of XML content.
    BadStructure,
}

use Field::*;
use CheckError::*;
use ParseError::*;
use XmlParseError::*;

impl Field {
    /// Return true if is player in this field.
    pub fn is_player(self) -> bool {
        self == Player || self == PlayerOnTarget
    }
    /// Return true if is pack in this field.
    pub fn is_pack(self) -> bool {
        self == Pack || self == PackOnTarget
    }
    /// Return true if is target in this field.
    pub fn is_target(self) -> bool {
        self == Target || self == PackOnTarget || self == PlayerOnTarget
    }
    /// Set player in this field even if this field contains other object.
    pub fn set_player(&mut self) {
        match *self {
            Target|PackOnTarget => *self = PlayerOnTarget,
            _ => *self = Player,
        }
    }
    /// Unset player in this field.
    pub fn unset_player(&mut self) {
        match *self {
            Player => *self = Empty,
            PlayerOnTarget => *self = Target,
            _ => panic!("Invalid field"),
        }
    }
    /// Set pack in this field even if this field contains other object.
    pub fn set_pack(&mut self) {
        match *self {
            Target|PlayerOnTarget => *self = PackOnTarget,
            _ => *self = Pack,
        }
    }
    /// Unset pack in this field.
    pub fn unset_pack(&mut self) {
        match *self {
            Pack => *self = Empty,
            PackOnTarget => *self = Target,
            _ => panic!("Invalid field"),
        }
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoPlayer => write!(f, "No player"),
            TooManyPlayers => write!(f, "Too many players"),
            NoPacksAndTargets => write!(f, "No packs and targets"),
            LevelOpen => write!(f, "Level open"),
            TooFewPacks(x) => write!(f, "Too few packs - required {}", x),
            TooFewTargets(x) => write!(f, "Too few targets - required {}", x),
            PackNotAvailable(x, y) => write!(f, "Pack {}x{} not available", x, y),
            TargetNotAvailable(x, y) => write!(f, "Target {}x{} not available", x, y),
            LockedPackApartWalls(x, y) =>
                write!(f, "Locked pack {}x{} apart walls", x, y),
            Locked2x2Block(x, y) => write!(f, "Locked 2x2 block {}x{}", x, y),
        }
    }
}

impl Error for CheckError {
}

#[derive(PartialEq,Eq)]
/// Type contains all check errors.
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

impl fmt::Debug for CheckErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self as &dyn fmt::Display).fmt(f)
    }
}

impl CheckErrors {
    pub(crate) fn new() -> CheckErrors {
        CheckErrors(Vec::new())
    }
    pub(crate) fn push(&mut self, e: CheckError) {
        self.0.push(e)
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl Error for CheckErrors {
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmptyLines => write!(f, "Empty lines"),
            WrongField(x, y) => write!(f, "Wrong field {}x{}", x, y),
            WrongSize(x, y) => write!(f, "Wrong size {}x{}", x, y),
        }
    }
}

impl Error for ParseError {
}

/// Level parse errors - contains errors and level name
#[derive(PartialEq,Eq)]
pub struct LevelParseError {
    pub(crate) number: usize,
    pub(crate) name: String,
    pub(crate) error: ParseError,
}

impl fmt::Display for LevelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nr: {}, Name: {}, Error: {}", self.number, self.name, self.error)
    }
}

impl fmt::Debug for LevelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self as &dyn fmt::Display).fmt(f)
    }
}

impl Error for LevelParseError {
}

impl fmt::Display for XmlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadStructure => writeln!(f, "Bad structure of XML"),
        }
    }
}

impl Error for XmlParseError {
}

pub(crate) fn char_to_field(x: char) -> Field {
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

pub(crate) fn is_not_field(x: char) -> bool {
    x!=' ' && x!='#' && x!='@' && x!='+' && x!='.' && x!='$' && x!='*'
}
