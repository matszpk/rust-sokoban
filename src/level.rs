// level.rs - main library of sokoban
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

use crate::defs::*;

use Field::*;
use Direction::*;
use CheckError::*;
use ParseError::*;

/// Level in game. Name is optional name - can be empty. Width and height determines
/// dimensions of the level. An area is fields of level ordered from top to bottom and
/// from left to right.
#[derive(PartialEq,Eq,Debug)]
pub struct Level {
    pub(crate) name: String,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) area: Vec<Field>,
}

impl Level {
    /// Get name of the level.
    pub fn name(&self) -> &String {
        &self.name
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
    
    /// Create empty level
    pub fn empty() -> Level {
        Level{ name: String::new(), width: 0, height: 0, area: vec![] }
    }
    
    // Create level from area data.
    pub fn new(name: &str, width: usize, height: usize, area: Vec<Field>)
                    -> Result<Level, ParseError> {
        if area.len() == width*height {
            Ok(Level{ name: String::from(name), width, height, area })
        } else {
            Err(WrongSize(width, height))
        }
    }
    
    // Parse level from string.
    pub fn from_str(name: &str, width: usize, height: usize, astr: &str)
                    -> Result<Level, ParseError> {
        if astr.len() != width*height {
            return Err(WrongSize(width, height));
        }
        let mut chrs = astr.chars();
        let chrs2 = chrs.clone();
        if let Some(pp) = chrs.position(is_not_field) {
            return Err(WrongField(pp%width, pp/width));
        }
        let area: Vec<Field> = chrs2.map(char_to_field).collect();
        Ok(Level{ name: String::from(name), width, height, area: area })
    }
    
    fn check_level_by_fill(&self, px: usize, py: usize, errors: &mut CheckErrors) {
        #[derive(Debug)]
        struct StackItem{ x: usize, y: usize, d: Direction }
        // find player
        let mut filled = vec![false; self.width*self.height];
        let mut stk = vec![StackItem{x: px, y: py, d:Left}];
        
        let mut touch_frames = false;
        
        while stk.len() != 0 {
            if let Some(it) = stk.last_mut() {
                if self.area[it.y*self.width + it.x] == Wall ||
                    (filled[it.y*self.width + it.x] && it.d==Left) {
                    stk.pop();  // if wall or already filled
                } else {
                    // fill this field
                    filled[it.y*self.width + it.x] = true;
                    // get next position
                    let next_pos = match it.d {
                        Left => {
                            it.d = Right;
                            if it.x > 0 {
                                Some((it.x-1, it.y))
                            } else {
                                touch_frames = true;
                                None
                            }
                        },
                        Right => {
                            it.d = Down;
                            if it.x+1 < self.width {
                                Some((it.x+1, it.y))
                            } else {
                                touch_frames = true;
                                None
                            }
                        }
                        Down => {
                            it.d = Up;
                            if it.y > 0 {
                                Some((it.x, it.y-1))
                            } else {
                                touch_frames = true;
                                None
                            }
                        }
                        Up => {
                            it.d = NoDirection;
                            if it.y+1 < self.height {
                                Some((it.x, it.y+1))
                            } else {
                                touch_frames = true;
                                None
                            }
                        }
                        _ => { None }
                    };
                    if let Some((x,y)) = next_pos {
                        stk.push(StackItem{x,y,d:Left}); // push next step
                    } else if it.d == NoDirection {
                        stk.pop();  // all is filled
                    }
                }
            }
        }
        
        if touch_frames {
            errors.push(LevelOpen);
        }
        // check availability
        self.area.iter().enumerate().for_each(|(i,x)| {
            if *x == Pack && !filled[i] {
                errors.push(PackNotAvailable(i % self.width, i / self.width))
            }
        });
        self.area.iter().enumerate().for_each(|(i,x)| {
            if *x == Target && !filled[i] {
                errors.push(TargetNotAvailable(i % self.width, i / self.width))
            }
        });
    }
    
    /// Check level.
    pub fn check(&self) -> Result<(), CheckErrors> {
        let mut errors = CheckErrors::new();
        let players_num = self.area.iter().filter(|x| x.is_player()).count();
        match players_num {
            0 => errors.push(NoPlayer),
            1 => {}
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
        
        if let Some(pp) = self.area.iter().position(|x| x.is_player()) {
            let x = pp % self.width;
            let y = pp / self.width;
            self.check_level_by_fill(x, y, &mut errors);
        }
        // find locks
        for iy in 0..self.height-1 {
            for ix in 0..self.width-1 {
                let field_ul = self.area[iy*self.width + ix];
                let field_ur = self.area[iy*self.width + ix+1];
                let field_dl = self.area[(iy+1)*self.width + ix];
                let field_dr = self.area[(iy+1)*self.width + ix+1];
                if (field_ul.is_pack() || field_ul==Wall)  &&
                    (field_ur.is_pack() || field_ur==Wall) &&
                    (field_dl.is_pack() || field_dl==Wall) &&
                    (field_dr.is_pack() || field_dr==Wall) {
                    let mut packs = 0;
                    if field_ul.is_pack() { packs+=1; }
                    if field_ur.is_pack() { packs+=1; }
                    if field_dl.is_pack() { packs+=1; }
                    if field_dr.is_pack() { packs+=1; }
                    let mut packs_on_target = 0;
                    if field_ul == PackOnTarget { packs_on_target+=1; }
                    if field_ur == PackOnTarget { packs_on_target+=1; }
                    if field_dl == PackOnTarget { packs_on_target+=1; }
                    if field_dr == PackOnTarget { packs_on_target+=1; }
                    // only if not all packs in target
                    if packs_on_target != packs {
                        errors.push(Locked2x2Block(ix, iy));
                    }
                }
            }
        }
        for iy in 1..self.height-1 {
            for ix in 1..self.width-1 {
                let field_u = self.area[(iy-1)*self.width + ix];
                let field_l = self.area[iy*self.width + ix-1];
                let field = self.area[iy*self.width + ix];
                let field_r = self.area[iy*self.width + ix+1];
                let field_d = self.area[(iy+1)*self.width + ix];
                if field == Pack {
                    if (field_u == Wall && (field_l == Wall || field_r == Wall)) ||
                        (field_d == Wall && (field_l == Wall || field_r == Wall)) ||
                        (field_l == Wall && (field_u == Wall || field_d == Wall)) ||
                        (field_r == Wall && (field_u == Wall || field_d == Wall)) {
                        errors.push(LockedPackApartWalls(ix, iy));
                    }
                }
            }
        }
        
        if errors.len() != 0 {
            Err(errors)
        } else { Ok(()) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_level_from_str() {
        let levela = Level::new("blable", 5, 3, vec![
            Wall, Wall, Wall, Wall, Wall,
            Wall, Target, Pack, Player, Wall,
            Wall, Wall, Wall, Wall, Wall]);
        assert!(levela.is_ok());
        let levelb = Level::from_str("blable", 5, 3,
            "#####\
             #.$@#\
             #####");
        assert_eq!(levela, levelb);
        
        let levela = Level::new("git", 8, 6, vec![
            Empty, Wall, Wall, Wall, Wall, Wall, Wall, Empty,
            Wall, Empty, Empty, Empty, Empty, Empty, Empty, Wall,
            Wall, Player, Empty, Empty, Target, Target, Target, Wall,
            Wall, Empty, Empty, Empty, Pack, Pack, Pack, Wall,
            Wall, Empty, Empty, Empty, Empty, Empty, Empty, Wall,
            Empty, Wall, Wall, Wall, Wall, Wall, Wall, Empty]);
        assert!(levela.is_ok());
        let levelb = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  ...#\
             #   $$$#\
             #      # \
              ###### ");
        assert_eq!(levela, levelb);
        
        let levela = Level::new("git", 8, 6, vec![
            Empty, Wall, Wall, Wall, Wall, Wall, Wall, Empty,
            Wall, Empty, Empty, Empty, Empty, Empty, Empty, Wall,
            Wall, Empty, Empty, Empty, PlayerOnTarget, Target, PackOnTarget, Wall,
            Wall, Empty, Empty, Empty, Pack, Pack, Empty, Wall,
            Wall, Empty, Empty, Empty, Empty, Empty, Empty, Wall,
            Empty, Wall, Wall, Wall, Wall, Wall, Wall, Empty]);
        assert!(levela.is_ok());
        let levelb = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #   +.*#\
             #   $$ #\
             #      # \
              ###### ");
        assert_eq!(levela, levelb);
        
        let levelb = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #   +.*#\
             #   $$ #\
             #  x   # \
              ###### ");
        assert_eq!(Err(WrongField(3,4)), levelb);
        let levelb = Level::from_str("git", 8, 7,
            " ###### \
             #      #\
             #   +.*#\
             #   $$ #\
             #      # \
              ###### ");
        assert_eq!(Err(WrongSize(8,7)), levelb);
    }
    
    #[test]
    fn test_check() {
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        assert_eq!(Ok(()), level.check());
        
        let level = Level::from_str("git", 11, 6,
            " ######    \
             #      ### \
             #@  ...#**#\
             #   $$$### \
             #      #    \
              ######    ").unwrap();
        assert_eq!(Ok(()), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  .*.#\
             #   $ $#\
             #      # \
              ###### ").unwrap();
        assert_eq!(Ok(()), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ### ## \
             #      #\
             #@  ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(LevelOpen);
        assert_eq!(Err(errors), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #   ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(NoPlayer);
        assert_eq!(Err(errors), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  +..#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(TooManyPlayers);
        assert_eq!(Err(errors), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #  @   #\
             #@  ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(TooManyPlayers);
        assert_eq!(Err(errors), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  .. #\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(TooFewTargets(3));
        assert_eq!(Err(errors), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #     .#\
             #@  ...#\
             #   $$ #\
             #      # \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(TooFewPacks(4));
        assert_eq!(Err(errors), level.check());
        
        // availability
        let level = Level::from_str("git", 11, 6,
            " ######### \
             #      #..#\
             #@  ...#$$#\
             #   $$$### \
             #      #    \
              ######    ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(PackNotAvailable(8, 2));
        errors.push(PackNotAvailable(9, 2));
        errors.push(TargetNotAvailable(8, 1));
        errors.push(TargetNotAvailable(9, 1));
        errors.push(Locked2x2Block(7, 2));
        errors.push(Locked2x2Block(8, 2));
        errors.push(LockedPackApartWalls(8, 2));
        errors.push(LockedPackApartWalls(9, 2));
        assert_eq!(Err(errors), level.check());
        
        // locks
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #   ...#\
             #@  $$.#\
             #   $$ #\
             #      # \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(Locked2x2Block(4, 2));
        assert_eq!(Err(errors), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  **.#\
             #   *$ #\
             #      # \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(Locked2x2Block(4, 2));
        assert_eq!(Err(errors), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  ** #\
             #   ** #\
             #   $ .# \
              ###### ").unwrap();
        assert_eq!(Ok(()), level.check());
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #$  ..*#\
             #@    .#\
             #      #\
             #$    $# \
              ###### ").unwrap();
        let mut errors = CheckErrors::new();
        errors.push(LockedPackApartWalls(1, 1));
        errors.push(LockedPackApartWalls(1, 4));
        errors.push(LockedPackApartWalls(6, 4));
        assert_eq!(Err(errors), level.check());
        
        // some random level
        let level = Level::from_str("git", 10, 8,
            " ####     \
             ##  ##### \
             #  $  $ # \
             # $*..* ##\
             #  *$$.  #\
             #@ *.*.  #\
             ####   ###   \
                #####  ").unwrap();
        assert_eq!(Ok(()), level.check());
        
        // some original level
        let level = Level::from_str("git", 20, 16,
            "####################\
             #..#    #          #\
             #.$  $  #$$  $## $##\
             #.$#  ###  ## ##   #\
             #  # $ #  $$   $   #\
             # ###  # #  #$  ####\
             #  ## # $   #@ #   #\
             # $    $  ##.##  $ #\
             #  # $# $# $     ###\
             #  #  #  #   ###   #\
             #  ######## #      #\
             #           #  #.#.#\
             ##$########$#   ...#\
             #    .*  #    ##.#.#\
             # .*...*   $  .....#\
             ####################").unwrap();
        assert_eq!(Ok(()), level.check());
    }
}
