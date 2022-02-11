// level_state.rs - main library of sokoban
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

use crate::Level;
use Field::*;
use Direction::*;
use CheckError::*;

/// LevelState is state game in given a level. A level state contains changed
/// an area of a level after moves. Initially an area is copied from level.
#[derive(PartialEq,Eq,Debug,Clone)]
pub struct LevelState<'a> {
    pub(crate) level: &'a Level,
    pub(crate) player_x: usize,
    pub(crate) player_y: usize,
    area: Vec<Field>,
    moves: Vec<Direction>,
}

impl<'a> LevelState<'a> {
    /// Create new level state from level.
    pub fn new(level: &'a Level) -> Result<LevelState<'a>, CheckError> {
        if let Some(pp) = level.area.iter().position(|x| x.is_player()) {
            let player_x = pp % level.width();
            let player_y = pp / level.width();
            Ok(LevelState{ level, player_x, player_y, area: level.area().clone(),
                    moves: vec!() })
        } else {
            Err(NoPlayer)
        }
    }
    
    /// Return player X position.
    pub fn player_x(&self) -> usize {
        self.player_x
    }
    /// Return player Y position.
    pub fn player_y(&self) -> usize {
        self.player_y
    }
    /// Return current area.
    pub fn area(&self) -> &Vec<Field> {
        &self.area
    }
    
    /// Reset level state to original state - undo all moves.
    pub fn reset(&mut self) {
        if let Some(pp) = self.level.area().iter().position(|x| x.is_player()) {
            self.moves = vec!();
            self.player_x = pp % self.level.width();
            self.player_y = pp / self.level.width();
            self.area.copy_from_slice(self.level.area());
        } else {
            panic!("No player!");
        }
    }
    
    /// Check whether level is done.
    pub fn is_done(&self) -> bool {
        let packs_num = self.area.iter().filter(|x| x.is_pack()).count();
        let targets_num = self.area.iter().filter(|x| x.is_target()).count();
        let packs_on_targets_num = self.area.iter().filter(
                    |x| **x == PackOnTarget).count();
        packs_num == packs_on_targets_num && targets_num == packs_on_targets_num
    }
    
    /// Make move if possible. Return 2 booleans.
    /// The first boolean indicates that move has been done.
    /// The second boolean indicates that move push pack.
    pub fn make_move(&mut self, dir: Direction) -> (bool, bool) {
        let width = self.level.width();
        let height = self.level.height();
        let this_pos = self.player_y*width + self.player_x;
        // get some setup for direction. next positions, new player position and directions.
        let (pnext_pos, pnext2_pos, new_x, new_y, dir, push_dir) = match dir {
            Left|PushLeft => {
                let pnext_pos = if self.player_x>0
                    { Some(this_pos-1) } else { None };
                let pnext2_pos = if self.player_x>1
                    { Some(this_pos-2) } else { None };
                (pnext_pos, pnext2_pos,
                self.player_x-1, self.player_y, Left, PushLeft)
            }
            Right|PushRight => {
                let pnext_pos = if self.player_x<width-1
                    { Some(this_pos+1) } else { None };
                let pnext2_pos = if self.player_x<width-2
                    { Some(this_pos+2) } else { None };
                (pnext_pos, pnext2_pos,
                self.player_x+1, self.player_y, Right, PushRight)
            }
            Up|PushUp => {
                let pnext_pos = if self.player_y>0
                    { Some(this_pos-width) } else { None };
                let pnext2_pos = if self.player_y>1
                    { Some(this_pos-2*width) } else { None };
                (pnext_pos, pnext2_pos,
                self.player_x, self.player_y-1, Up, PushUp)
            }
            Down|PushDown => {
                let pnext_pos = if self.player_y<height-1
                    { Some(this_pos+width) } else { None };
                let pnext2_pos = if self.player_y<height-2
                    { Some(this_pos+2*width) }else { None };
                (pnext_pos, pnext2_pos,
                self.player_x, self.player_y+1, Down, PushDown)
            }
            NoDirection => (None, None, 0, 0, NoDirection, NoDirection),
        };
        
        if let Some(next_pos) = pnext_pos {
            // check whether if wall
            match self.area[next_pos] {
                Empty|Target => {
                    self.area[next_pos].set_player();
                    self.area[this_pos].unset_player();
                    self.player_x = new_x;
                    self.player_y = new_y;
                    self.moves.push(dir);
                    (true, false)
                }
                Pack|PackOnTarget => {
                    if let Some(next2_pos) = pnext2_pos {
                        if self.area[next2_pos] != Wall &&
                            !self.area[next2_pos].is_pack() {
                            self.area[next2_pos].set_pack();
                            self.area[next_pos].set_player();
                            self.area[this_pos].unset_player();
                            self.player_x = new_x;
                            self.player_y = new_y;
                            self.moves.push(push_dir);
                            (true, true)
                        } else { (false, false) }
                    } else {
                        (false, false)
                    }
                }
                _ => (false, false)
            }
        } else { (false, false) }
    }
    
    /// Undo move. Return true if move undone.
    pub fn undo_move(&mut self) -> bool {
        if let Some(dir) = self.moves.pop() {
            let width = self.level.width();
            let height = self.level.height();
            let this_pos = self.player_y*width + self.player_x;
            
            let (prev_pos, pnext_pos, old_x, old_y) = match dir {
                Right|PushRight => {
                    if self.player_x==0 { panic!("Unexpected frame"); }
                    let next_pos = if dir == PushRight
                        { Some(this_pos+1) } else { None };
                    (this_pos-1, next_pos, self.player_x-1, self.player_y)
                }
                Left|PushLeft => {
                    if self.player_x>=width-1 { panic!("Unexpected frame"); }
                    let next_pos = if dir == PushLeft
                        { Some(this_pos-1) } else { None };
                    (this_pos+1, next_pos, self.player_x+1, self.player_y)
                }
                Down|PushDown => {
                    if self.player_y==0 { panic!("Unexpected frame"); }
                    let next_pos = if dir == PushDown
                        { Some(this_pos+width) } else { None };
                    (this_pos-width, next_pos, self.player_x, self.player_y-1)
                }
                Up|PushUp => {
                    if self.player_y>=height-1 { panic!("Unexpected frame"); }
                    let next_pos = if dir == PushUp
                        { Some(this_pos-width) } else { None };
                    (this_pos+width, next_pos, self.player_x, self.player_y+1)
                }
                NoDirection => {
                    panic!("Unknown direction");
                }
            };
            
            if let Some(next_pos) = pnext_pos {
                self.area[next_pos].unset_pack();
                self.area[this_pos].set_pack();
            } else {
                self.area[this_pos].unset_player();
            }
            self.area[prev_pos].set_player();
            self.player_x = old_x;
            self.player_y = old_y;
            true
        } else { false }
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
    fn test_make_move_and_undo_move() {
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             # @ ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, false), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 1, player_y: 2,
            area: Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![Left] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, false), lstate.make_move(Right));
        assert_eq!(LevelState{ level: &level,
            player_x: 3, player_y: 2,
            area: Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #  @...#\
             #   $$$#\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![Right] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, false), lstate.make_move(Up));
        assert_eq!(LevelState{ level: &level,
            player_x: 2, player_y: 1,
            area: Level::from_str("git", 8, 6,
            " ###### \
             # @    #\
             #   ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![Up] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, false), lstate.make_move(Down));
        assert_eq!(LevelState{ level: &level,
            player_x: 2, player_y: 3,
            area: Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #   ...#\
             # @ $$$#\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![Down] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        // move from target
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             # +  ..#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, false), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 1, player_y: 2,
            area: Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@.  ..#\
             #   $$$#\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![Left] },
            lstate);
        let mut lstate2 = lstate.clone();
        assert_eq!(true, lstate2.undo_move());
        assert_eq!(old_lstate, lstate2);
        // move to target
        let old_lstate = lstate.clone();
        assert_eq!((true, false), lstate.make_move(Right));
        assert_eq!(LevelState{ level: &level,
            player_x: 2, player_y: 2,
            area: Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             # +  ..#\
             #   $$$#\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![Left,Right] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        // move failures
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 1, player_y: 2,
            area: level.area().clone(),
            moves: vec![] },
            lstate);
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #   ..+#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Right));
        assert_eq!(LevelState{ level: &level,
            player_x: 6, player_y: 2,
            area: level.area().clone(),
            moves: vec![] },
            lstate);
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #  @   #\
             #   ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Up));
        assert_eq!(LevelState{ level: &level,
            player_x: 3, player_y: 1,
            area: level.area().clone(),
            moves: vec![] },
            lstate);
        
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #   ...#\
             #   $$$#\
             #  @   # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Down));
        assert_eq!(LevelState{ level: &level,
            player_x: 3, player_y: 4,
            area: level.area().clone(),
            moves: vec![] },
            lstate);
        
        // pushes
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             # ..$  #\
             #  $@$ #\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, true), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 3, player_y: 3,
            area: Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             # ..$  #\
             # $@ $ #\
             #   $  #\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![PushLeft] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, true), lstate.make_move(Right));
        assert_eq!(LevelState{ level: &level,
            player_x: 5, player_y: 3,
            area: Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             # ..$  #\
             #  $ @$#\
             #   $  #\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![PushRight] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, true), lstate.make_move(Up));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 2,
            area: Level::from_str("git", 8, 7,
            " ###### \
             # ..$  #\
             # ..@  #\
             #  $ $ #\
             #   $  #\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![PushUp] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, true), lstate.make_move(Down));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 4,
            area: Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             # ..$  #\
             #  $ $ #\
             #   @  #\
             #   $  # \
              ###### ").unwrap().area().clone(),
            moves: vec![PushDown] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        // pushes from/to target
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             #  .$  #\
             # .$@$ #\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, true), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 3, player_y: 3,
            area: Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             #  .$  #\
             # *@ $ #\
             #   $  #\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![PushLeft] },
            lstate);
        let mut lstate2 = lstate.clone();
        assert_eq!(true, lstate2.undo_move());
        assert_eq!(old_lstate, lstate2);
        
        let old_lstate = lstate.clone();
        assert_eq!((true, true), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 2, player_y: 3,
            area: Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             #  .$  #\
             #$+  $ #\
             #   $  #\
             #      # \
              ###### ").unwrap().area().clone(),
            moves: vec![PushLeft, PushLeft] },
            lstate);
        assert_eq!(true, lstate.undo_move());
        assert_eq!(old_lstate, lstate);
        
        // pushes failures
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             #...$  #\
             # $$@$ #\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             #  .$  #\
             # **@$ #\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             # ..$  #\
             # #$@$ #\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Left));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
        
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             #...$  #\
             #  $@$$#\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Right));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             # ..$  #\
             #  $@$##\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Right));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
        
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..$  #\
             #...$  #\
             #  $@$ #\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Up));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..#  #\
             # ..$  #\
             #  $@$ #\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Up));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
        
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             #...$  #\
             #  $@$ #\
             #   $  #\
             #   $  # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Down));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             # ..$  #\
             #  $@$ #\
             #   $  #\
             #   #  # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        assert_eq!((false, false), lstate.make_move(Down));
        assert_eq!(LevelState{ level: &level,
            player_x: 4, player_y: 3,
            area:level.area().clone(),
            moves: vec![] },
            lstate);
    }
    
    #[test]
    fn test_reset() {
        let level = Level::from_str("git", 8, 7,
            " ###### \
             # ..   #\
             #  .$  #\
             # .$@$ #\
             #   $  #\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        let old_lstate = lstate.clone();
        assert_eq!((true, true), lstate.make_move(Left));
        assert_eq!((true, true), lstate.make_move(Left));
        lstate.reset();
        assert_eq!(old_lstate, lstate);
    }
    
    #[test]
    fn test_is_done() {
        let level = Level::from_str("git", 8, 6,
            " ###### \
             #      #\
             #@  ...#\
             #   $$$#\
             #      # \
              ###### ").unwrap();
        let mut lstate = LevelState::new(&level).unwrap();
        for m in vec![Down, Down, Right, Right, Right,
                    Up, Down,Right, Up, Down, Right, Up] {
            assert_eq!(false, lstate.is_done());
            lstate.make_move(m);
        }
        assert_eq!(true, lstate.is_done());
    }
}
