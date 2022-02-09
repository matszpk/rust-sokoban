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
use std::io;
use std::io::{Read,BufRead,BufReader,Seek};
use std::fs::File;
use std::path::Path;
use std::fmt;
use int_enum::IntEnum;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event as XmlEvent;

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

#[derive(Debug,PartialEq)]
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

use Field::*;
use Direction::*;
use CheckError::*;

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

#[derive(PartialEq)]
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

impl Error for CheckErrors {
}

#[derive(PartialEq,Debug)]
/// Error caused while parsing or creating level.
pub enum ParseError {
    /// If empty lines.
    EmptyLines,
    /// If wrong field.
    WrongField(usize, usize),
    /// If wrong size.
    WrongSize(usize, usize),
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

use ParseError::*;

/// Level in game. Name is optional name - can be empty. Width and height determines
/// dimensions of the level. An area is fields of level ordered from top to bottom and
/// from left to right.
#[derive(PartialEq,Debug)]
pub struct Level {
    name: String,
    width: usize,
    height: usize,
    area: Vec<Field>,
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
    
    /// Parse level from lines.
    pub fn from_lines<B>(reader: &io::Lines::<B>) ->
                    Result<Level, ParseError> {
        Err(EmptyLines)
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


/// LevelState is state game in given a level. A level state contains changed
/// an area of a level after moves. Initially an area is copied from level.
#[derive(PartialEq,Debug,Clone)]
pub struct LevelState<'a> {
    level: &'a Level,
    player_x: usize,
    player_y: usize,
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

/// Level parse errors - contains errors and level name
#[derive(PartialEq)]
pub struct LevelParseError {
    number: usize,
    name: String,
    error: ParseError,
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

pub type LevelResult = Result<Level, LevelParseError>;

fn level_result_set_name(lr: &mut LevelResult, name: &String) {
    match lr {
        Ok(l) => l.name = name.clone(),
        Err(e) => e.name = name.clone(),
    }
}

/// Level set. Contains levels and name of the level set.
#[derive(PartialEq, Debug)]
pub struct LevelSet {
    name: String,
    levels: Vec<LevelResult>,
}

#[derive(PartialEq,Debug)]
pub enum XmlParseError {
    BadStructure,
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

use XmlParseError::*;

impl LevelSet {
    /// Get name of levelset.
    pub fn name(&self) -> &String {
        &self.name
    }
    /// Get levels.
    pub fn levels(&self) -> &Vec<LevelResult> {
        &self.levels
    }
    
    /// Return true if level set has errors.
    pub fn has_errors(&self) -> bool {
        self.levels.iter().find(|lr| lr.is_err()).is_some()
    }
    
    /// Read levelset from string.
    pub fn from_str(str: &str) -> Result<LevelSet, Box<dyn Error>> {
        Self::from_reader(&mut io::Cursor::new(str.as_bytes()))
    }
    /// Read levelset from file.
    pub fn from_file<P: AsRef<Path>>(path: P) ->
                    Result<LevelSet, Box<dyn Error>> {
        let f = File::open(path)?;
        Self::from_reader(&mut BufReader::new(f))
    }
    /// Read levelset from reader.
    pub fn from_reader<B: BufRead + Read + Seek>(reader: &mut B) ->
                    Result<LevelSet, Box<dyn Error>> {
        let mut first_bytes = [0;5];
        let readed = reader.read(&mut first_bytes)?;
        reader.seek(io::SeekFrom::Start(0))?;
        if readed == 5 && (&first_bytes == b"<?xml") {
            // if xml
            Self::read_from_xml(reader)
        } else {
            // if text
            Self::read_from_text(reader)
        }
    }
    
    fn read_from_text<B: BufRead + Read + Seek>(reader: &mut B) ->
                    Result<LevelSet, Box<dyn Error>> {
        let mut lines = reader.lines();
        
        let mut lset = LevelSet{ name: String::new(), levels: vec![] };
        if let Some(rl) = lines.next() {
            let l = rl?; // handle error
            if l.starts_with(";") {
                lset.name = l[1..].trim().to_string();
            }
        }
        // skip comments and spaces
        let mut first_empty_line = false;
        let mut lev_lines = lines.skip_while(|rl| {
            if let Ok(l) = rl {
                if l.starts_with(";") { return true; }
                else if l.len()!=0 {
                    if let Some(c) = l.chars().next() {
                        // skip some text
                        if c.is_alphanumeric() { return true; }
                    }
                } else if !first_empty_line && l.trim().len() == 0 {
                    first_empty_line = true;
                    return true
                }
            }
            false
        }).filter(|rl| {
            if let Ok(l) = rl {
                l.trim().len() != 0
            } else { false }
        });
        
        // parse levels
        let mut level_name_first = false;
        let mut level_name = String::new();
        let mut l;
        if let Some(rl) = lev_lines.next() {
            l = rl?; // handle error and get line
            'a: loop {
                if l.starts_with(";") {
                    // comments
                    level_name = l[1..].trim().to_string();
                    if lset.levels.len() == 0 {
                        level_name_first = true;
                    }
                    if !level_name_first {
                        if let Some(level_result) = lset.levels.last_mut() {
                            level_result_set_name(level_result, &level_name);
                        }
                    }
                    loop {
                        if let Some(rl) = lev_lines.next() {
                            l = rl?;
                            // skip other comments
                            if !l.starts_with(";") { break; }
                        } else { break 'a; }
                    }
                } else {
                    // level area
                    let mut level = Level::empty();
                    let mut error = None;
                    let mut level_lines = vec![];
                    
                    level.name = level_name.clone();
                    let mut end = false;
                    loop {
                        if l.starts_with(";") { break; }
                        level.width = level.width.max(l.len());
                        if let Some(pp) = l.chars().position(is_not_field) {
                            // generate error
                            error = Some(LevelParseError{
                                number: lset.levels.len(), name: level_name.clone(),
                                error: WrongField(pp, level_lines.len()-1) })
                        }
                        level_lines.push(l.trim_end().to_string());
                        if let Some(rl) = lev_lines.next() {
                            l = rl?;
                        } else {
                            end = true;
                            break; }
                    }
                    
                    if error == None {
                        level.height = level_lines.len();
                        // construct level
                        level.area = vec![Empty; level.width*level.height];
                        for y in 0..level_lines.len() {
                            level_lines[y].chars().enumerate().for_each(|(x,c)| {
                                level.area[y*level.width + x] = char_to_field(c);
                            });
                        }
                        lset.levels.push(Ok(level));
                    } else {
                        lset.levels.push(Err(error.unwrap()));
                    }
                    
                    if end { break; }
                }
            }
        }
        
        // parse levels
        Ok(lset)
    }
    
    fn read_from_xml<B: BufRead + Read + Seek>(reader: &mut B) ->
                    Result<LevelSet, Box<dyn Error>> {
        let mut lset = LevelSet{ name: String::new(), levels: vec![] };
        
        let mut reader = XmlReader::from_reader(reader);
        let mut buf = Vec::new();
        let mut in_levels = false;
        let mut in_level_collection = false;
        let mut in_level_line = false;
        let mut in_title = false;
        
        loop {
            let mut in_level = false;
            let mut level_id: Option<String> = None;
            let (mut level_width, mut level_height) = (0 as usize, 0 as usize);
            
            let res_event = reader.read_event(&mut buf);
            
            match res_event {
                Ok(XmlEvent::Start(ref e)) => {
                    match e.name() {
                        b"SokobanLevels" => {
                            if in_levels {
                                return Err(Box::new(BadStructure));
                            }
                            in_levels = true;
                        }
                        b"Title" => {
                            if in_level_collection {
                                return Err(Box::new(BadStructure));
                            }
                            in_title = true;
                        }
                        b"LevelCollection" => {
                            if !in_levels {
                                return Err(Box::new(BadStructure));
                            }
                            in_level_collection = true;
                        }
                        b"Level" => {
                            if !in_level_collection {
                                return Err(Box::new(BadStructure));
                            }
                            for ra in e.attributes() {
                                if let Ok(attr) = ra {
                                    match attr.key {
                                        b"Id" => {
                                            level_id = Some(
                                                attr.unescape_and_decode_value(&reader)?);
                                        },
                                        b"Width" => {
                                            level_width = attr.
                                                unescape_and_decode_value(&reader)?.parse()?;
                                        },
                                        b"Height" => {
                                            level_height = attr.
                                                unescape_and_decode_value(&reader)?.parse()?;
                                        },
                                        _ => {},
                                    }
                                }
                            }
                            in_level = true;
                        }
                        _ => {}
                    }
                }
                Ok(XmlEvent::End(ref e)) => {
                    match e.name() {
                        b"SokobanLevels" => { in_levels = false; }
                        b"Title" => { in_title = false; }
                        b"LevelCollection" => { in_level_collection = false; }
                        _ => {}
                    }
                }
                Ok(XmlEvent::Text(e)) => {
                    if in_title {
                        lset.name = e.unescape_and_decode(&reader)?;
                        in_title = false;
                    }
                }
                Err(e) => { return Err(Box::new(e)); }
                Ok(XmlEvent::Eof) => break,
                _ => {}
            }
            
            if in_level {
                let mut level = Level::empty();
                if let Some(lid) = level_id {
                    level.name = lid.clone();
                }
                level.width = level_width;
                level.height = level_height;
                
                let mut level_lines = vec![];
                
                loop {
                    match reader.read_event(&mut buf) {
                        Ok(XmlEvent::Start(ref e)) => {
                            match e.name() {
                                b"L" => {
                                    in_level_line = true;
                                }
                                _ => {}
                            }
                        }
                        Ok(XmlEvent::End(ref e)) => {
                            match e.name() {
                                b"Level" => { break; }
                                b"L" => { in_level_line = false; }
                                _ => {}
                            }
                        }
                        Err(e) => { return Err(Box::new(e)); }
                        Ok(XmlEvent::Text(e)) => {
                            if in_level_line {
                                if level.height != 0 && level_lines.len() == level.height {
                                    break; // do not fetch next lines
                                }
                                
                                // if in_level_line
                                let l = e.unescape_and_decode(&reader)?;
                                if l.len() > level.width {
                                    level_lines.push(l.trim_end()[..level.width].to_string());
                                } else {
                                    level_lines.push(l.trim_end().to_string());
                                }
                            }
                        }
                        Ok(XmlEvent::Eof) => break,
                        _ => {}
                    }
                }
                
                if level.height == 0 {
                    level.height = level_lines.len();
                }
                if level.width == 0 { // find max width
                    level.width = level_lines.iter().fold(0, |mx, x| mx.max(x.len()));
                }
                
                // parse level
                let mut error = None;
                level.area = vec![Empty; level.width*level.height];
                for y in 0..level_lines.len() {
                    if let Some(pp) = level_lines[y].chars().position(is_not_field) {
                        // if error found
                        error = Some(LevelParseError{
                                number: lset.levels.len(), name: level.name.clone(),
                                error: WrongField(pp, level_lines.len()-1) });
                        break;
                    }
                    level_lines[y].chars().enumerate().for_each(|(x,c)| {
                                level.area[y*level.width + x] = char_to_field(c);
                            });
                }
                // final push: error or level.
                if let Some(e) = error {
                    lset.levels.push(Err(e));
                } else {
                    lset.levels.push(Ok(level));
                }
            }
        }
        Ok(LevelSet{name: "".to_string(), levels: vec![] })
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
    
    #[test]
    fn test_read_from_text() {
        let input_str = r##"; Microban IV

; Copyright: David W Skinner
; E-Mail: sasquatch@bentonrea.com
; Web Site: http://users.bentonrea.com/~sasquatch/sokoban/
;
; Microban IV (102 puzzles, August 2010) This set includes a series of alphabet
; puzzles.

   #####
####@  #
#  $*. #
#     ##
#  #####
####
; first

      #####
   ####   #
####  $*. #
#  $*.   ##
# @   #####
#  ####
####
; second

########
#  #   #
# $$*. #
# .  . #
# .*$$@#
#   #  #
########
; third
"##;
        let lsr = LevelSet::from_str(input_str).unwrap();
        let exp_lsr = LevelSet{ name: "Microban IV".to_string(),
            levels: vec![
                Ok(Level::from_str("first", 8, 6,
                    "   #####\
                     ####@  #\
                     #  $*. #\
                     #     ##\
                     #  #####\
                     ####    ").unwrap()),
                Ok(Level::from_str("second", 11, 7,
                    "      #####   \
                        ####   #\
                     ####  $*. #\
                     #  $*.   ##\
                     # @   #####\
                     #  ####    \
                     ####       ").unwrap()),
                Ok(Level::from_str("third", 8, 7,
                    "########\
                     #  #   #\
                     # $$*. #\
                     # .  . #\
                     # .*$$@#\
                     #   #  #\
                     ########").unwrap()),
            ] };
        assert_eq!(exp_lsr, lsr);
        
        let input_str = r##"; Microban IV

; Copyright: David W Skinner
; E-Mail: sasquatch@bentonrea.com
; Web Site: http://users.bentonrea.com/~sasquatch/sokoban/
;
; Microban IV (102 puzzles, August 2010) This set includes a series of alphabet
; puzzles.

; first
   #####
####@  #
#  $*. #
#     ##
#  #####
####

; second
      #####
   ####   #
####  $*. #
#  $*.   ##
# @   #####
#  ####
####

; third
########
#  #   #
# $$*. #
# .  . #
# .*$$@#
#   #  #
########
"##;
        let lsr = LevelSet::from_str(input_str).unwrap();
        assert_eq!(exp_lsr, lsr);

let input_str = r##"; Microban IV

; Copyright: David W Skinner
; E-Mail: sasquatch@bentonrea.com
Web Site: http://users.bentonrea.com/~sasquatch/sokoban/
;
Microban IV (102 puzzles, August 2010) This set includes a series of alphabet
; puzzles.

; first
   #####
####@  #
#  $*. #
#     ##
#  #####
####

; second
      #####
   ####   #
####  $*. #
#  $*.   ##
# @   #####
#  ####
####

; third
########
#  #   #
# $$*. #
# .  . #
# .*$$@#
#   #  #
########


"##;
        let lsr = LevelSet::from_str(input_str).unwrap();
        assert_eq!(exp_lsr, lsr);
    }
}

pub fn sokhello() {
    let mut errors = CheckErrors::new();
    errors.push(CheckError::LockedPackApartWalls(4, 5));
    errors.push(CheckError::Locked2x2Block(7, 7));
    println!("SokHello! {}x", errors)
}
