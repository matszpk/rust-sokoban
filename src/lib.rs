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
use int_enum::IntEnum;

/// Type represents direction of the move.
#[repr(u8)]
#[derive(PartialEq,Debug,Clone,Copy,IntEnum)]
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
    NoDirection = 8,
}

/// Type represents field in level area.
#[repr(u8)]
#[derive(PartialEq,Debug,Clone,Copy,IntEnum)]
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

// Check level error.
pub enum CheckError {
    /// No player.
    NoPlayer,
    /// Too many players.
    TooManyPlayers,
    /// No packs and targets.
    NoPacksAndTargets,
    /// If level open (no closing walls) - place where level is open.
    LevelOpen(usize, usize),
    /// If too few packs - number of required packs.
    TooFewPacks(usize),
    /// If too few targets - number of required targets.
    TooFewTargets(usize),
    /// If pack is not available for player - place of pack.
    PackNotAvailable(usize, usize),
    /// If target not available for player - place of target.
    TargetNotAvailable(usize, usize),
    /// If pack locked apart wall - place of pack.
    LockedPackApartWall(usize, usize),
    /// If 4 packs creates 2x2 box - place of 2x2 box.
    Locked4Packs(usize, usize),
}

use Field::*;
use Direction::*;
use CheckError::*;

impl Field {
    pub fn is_player(self) -> bool {
        self == Player || self == PlayerOnTarget
    }
    pub fn is_pack(self) -> bool {
        self == Pack || self == PackOnTarget
    }
    pub fn is_target(self) -> bool {
        self == Target || self == PackOnTarget || self == PlayerOnTarget
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoPlayer => write!(f, "No player"),
            TooManyPlayers => write!(f, "Too many players"),
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
    fn push(&mut self, e: CheckError) {
        self.0.push(e)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(PartialEq,Debug)]
pub enum ParseError {
    EmptyLines,
    WrongField(usize, usize),
    WrongSize(usize, usize),
}

use ParseError::*;

/// Level in game. Name is optional name - can be empty. Width and height determines
/// dimensions of the level. An area is fields of level ordered from top to bottom and
/// from left to right.
#[derive(PartialEq,Debug)]
pub struct Level<'a> {
    name: &'a str,
    width: usize,
    height: usize,
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
    pub fn width(&self) -> usize {
        self.width
    }
    /// Get height of the level.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Get an area of the level.
    pub fn area(&self) -> &Vec<Field> {
        &self.area
    }
    
    // Create level from area data.
    pub fn new(name: &'a str, width: usize, height: usize, area: Vec<Field>) -> Level<'a> {
        Level{ name, width, height, area, moves: vec!() }
    }
    
    // Parse level from string.
    pub fn from_string(name: &'a str, width: usize, height: usize, astr: &str)
                    -> Result<Level<'a>, ParseError> {
        if astr.len() != width*height {
            return Err(WrongSize(width, height));
        }
        let mut chrs = astr.chars();
        let chrs2 = chrs.clone();
        if let Some(pp) = chrs.position(is_not_field) {
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
    
    fn check_level_by_fill(&self) -> bool {
        // find player
        if let Some(pp) = self.area.iter().position(|x| x.is_player()) {
            let x = pp % self.width;
            let y = pp / self.width;
            //
            let mut filled = vec![false; self.width*self.height];
            let mut stk = vec![(x,y,Left)];
            let mut target_count = 0;
            let mut pack_count = 0;
            while stk.len() != 0 {
                if let Some(it) = stk.last_mut() {
                    if self.area[it.1*self.width + it.0] == Wall ||
                        filled[it.1*self.width + it.0] {
                        stk.pop();  // if wall or already filled
                    } else {
                        // fill this field
                        if self.area[it.1*self.width + it.0].is_target() {
                            target_count+=1;
                        }
                        if self.area[it.1*self.width + it.0].is_pack() {
                            pack_count+=1;
                        }
                        filled[it.1*self.width + it.0] = true;
                        // get next position
                        let next_pos = match it.2 {
                            Left => {
                                it.2 = Right;
                                if it.0 > 0 {
                                    Some((it.0-1, it.1))
                                } else { None }
                            },
                            Right => {
                                it.2 = Down;
                                if it.0+1 < self.width {
                                    Some((it.0+1, it.1))
                                } else { None }
                            }
                            Down => {
                                it.2 = Up;
                                if it.1 > 0 {
                                    Some((it.0, it.1-1))
                                } else { None }
                            }
                            Up => {
                                it.2 = NoDirection;
                                if it.1+1 < self.height {
                                    Some((it.0, it.1+1))
                                } else { None }
                            }
                            _ => { None }
                        };
                        if let Some((x,y)) = next_pos {
                            stk.push((x,y,Left)); // push next step
                        } else if it.2 == NoDirection {
                            stk.pop();  // all is filled
                        }
                    }
                }
            }
        }
        false
    }
    
    /// Check level.
    pub fn check(&self) -> Result<(), CheckErrors> {
        let mut errors = CheckErrors::new();
        let players_num = self.area.iter().filter(|x| x.is_player()).count();
        match players_num {
            0 => errors.push(NoPlayer),
            _ => errors.push(TooManyPlayers),
        }
        // check number of packs and targets.
        let packs_num = self.area.iter().filter(|x| x.is_pack()).count();
        let targets_num = self.area.iter().filter(|x| x.is_target()).count();
        if packs_num < targets_num {
            errors.push(TooFewPacks(targets_num));
        } else if targets_num < packs_num {
            errors.push(TooFewTargets(packs_num));
        }
        
        // check whether level is open: by filling
        
        if errors.len() != 0 {
            Err(errors)
        } else { Ok(()) }
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
