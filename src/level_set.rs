// level_set.rs - main library of sokoban
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
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event as XmlEvent;

use crate::defs::*;

use crate::Level;
use Field::*;
use ParseError::*;
use XmlParseError::*;

/// Level result - contains level or parse error.
pub type LevelResult = Result<Level, LevelParseError>;

fn level_result_set_name(lr: &mut LevelResult, name: &String) {
    match lr {
        Ok(l) => l.name = name.clone(),
        Err(e) => e.name = name.clone(),
    }
}

/// Level set. Contains levels and name of the level set.
#[derive(PartialEq,Eq, Debug)]
pub struct LevelSet {
    name: String,
    levels: Vec<LevelResult>,
}

impl LevelSet {
    /// Get name of levelset.
    pub fn name(&self) -> &String {
        &self.name
    }
    /// Get levels.
    pub fn levels(&self) -> &Vec<LevelResult> {
        &self.levels
    }
    
    /// Returns true if level set has errors.
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
                                error: WrongField(pp, level_lines.len()) })
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
                                if level.width != 0 && l.len() > level.width {
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
                    level.width = level_lines.iter().map(|x| x.len()).max().
                            unwrap_or_default();
                }
                
                // parse level
                let mut error = None;
                level.area = vec![Empty; level.width*level.height];
                for y in 0..level_lines.len() {
                    if let Some(pp) = level_lines[y].chars().position(is_not_field) {
                        // if error found
                        error = Some(LevelParseError{
                                number: lset.levels.len(), name: level.name.clone(),
                                error: WrongField(pp, y) });
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
        Ok(lset)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
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
#  $*.  b##
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
        let exp_lsr = LevelSet{ name: "Microban IV".to_string(),
            levels: vec![
                Ok(Level::from_str("first", 8, 6,
                    "   #####\
                     ####@  #\
                     #  $*. #\
                     #     ##\
                     #  #####\
                     ####    ").unwrap()),
                Err(LevelParseError{ number: 1, name: "second".to_string(),
                        error: WrongField(8, 3) }),
                Ok(Level::from_str("third", 8, 7,
                    "########\
                     #  #   #\
                     # $$*. #\
                     # .  . #\
                     # .*$$@#\
                     #   #  #\
                     ########").unwrap()),
            ] };

        let lsr = LevelSet::from_str(input_str).unwrap();
        assert_eq!(exp_lsr, lsr);
    }
    
    #[test]
    fn test_read_from_xml() {
        let input_str = r##"<?xml version="1.0" encoding="utf-8"?>
<SokobanLevels xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="SokobanLev.xsd">
  <Title>Microban</Title>
  <Email>sasquatch@bentonrea.com</Email>
  <Url>http://users.bentonrea.com/~sasquatch/sokoban/</Url>
  <LevelCollection Copyright="David W Skinner" MaxWidth="30" MaxHeight="17">
    <Level Id="funny" Width="6" Height="7">
      <L>####</L>
      <L># .#</L>
      <L>#  ###</L>
      <L>#*@  #</L>
      <L>#  $ #</L>
      <L>#  ###</L>
      <L>####</L>
    </Level>
    <Level Id="blocky" Width="6" Height="7">
      <L>######</L>
      <L>#    #</L>
      <L># #@ #</L>
      <L># $* #</L>
      <L># .* #</L>
      <L>#    #</L>
      <L>######</L>
    </Level>
    <Level Id="harder" Width="9" Height="6">
      <L>  ####</L>
      <L>###  ####</L>
      <L>#     $ #</L>
      <L># #  #$ #</L>
      <L># . .#@ #</L>
      <L>#########</L>
    </Level>
  </LevelCollection>
</SokobanLevels>"##;
        
            let lsr = LevelSet::from_str(input_str).unwrap();
            let exp_lsr = LevelSet{ name: "Microban".to_string(),
            levels: vec![
                Ok(Level::from_str("funny", 6, 7,
                    "####  \
                     # .#  \
                     #  ###\
                     #*@  #\
                     #  $ #\
                     #  ###\
                     ####  ").unwrap()),
                Ok(Level::from_str("blocky", 6, 7,
                    "######\
                     #    #\
                     # #@ #\
                     # $* #\
                     # .* #\
                     #    #\
                     ######").unwrap()),
                Ok(Level::from_str("harder", 9, 6,
                    "  ####   \
                     ###  ####\
                     #     $ #\
                     # #  #$ #\
                     # . .#@ #\
                     #########").unwrap()),
            ] };
            assert_eq!(exp_lsr, lsr);
            
            let input_str = r##"<?xml version="1.0" encoding="utf-8"?>
<SokobanLevels xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="SokobanLev.xsd">
  <Title>Microban</Title>
  <Email>sasquatch@bentonrea.com</Email>
  <Url>http://users.bentonrea.com/~sasquatch/sokoban/</Url>
  <LevelCollection Copyright="David W Skinner" MaxWidth="30" MaxHeight="17">
    <Level Id="funny">
      <L>####</L>
      <L># .#</L>
      <L>#  ###</L>
      <L>#*@  #</L>
      <L>#  $ #</L>
      <L>#  ###</L>
      <L>####</L>
    </Level>
    <Level Id="blocky">
      <L>######</L>
      <L>#    #</L>
      <L># #@ #</L>
      <L># $* #</L>
      <L># .* #</L>
      <L>#    #</L>
      <L>######</L>
    </Level>
    <Level Id="harder">
      <L>  ####</L>
      <L>###  ####</L>
      <L>#     $ #</L>
      <L># #  #$ #</L>
      <L># . .#@ #</L>
      <L>#########</L>
    </Level>
  </LevelCollection>
</SokobanLevels>"##;
            
            let lsr = LevelSet::from_str(input_str).unwrap();
            assert_eq!(exp_lsr, lsr);
            
            let input_str = r##"<?xml version="1.0" encoding="utf-8"?>
<SokobanLevels xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="SokobanLev.xsd">
  <Title>Microban</Title>
  <Email>sasquatch@bentonrea.com</Email>
  <Url>http://users.bentonrea.com/~sasquatch/sokoban/</Url>
  <LevelCollection Copyright="David W Skinner" MaxWidth="30" MaxHeight="17">
    <Level Id="funny">
      <L>####</L>
      <L># .#</L>
      <L>#  ###</L>
      <L>#*@  #</L>
      <L>#  $ #</L>
      <L>#  ###</L>
      <L>####</L>
    </Level>
    <Level Id="blocky">
      <L>######</L>
      <L>#    #</L>
      <L># #@ #</L>
      <L># $* #</L>
      <L># .* #</L>
      <L>#    #</L>
      <L>######</L>
    </Level>
    <Level Id="harder">
      <L>  ####</L>
      <L>###  ####</L>
      <L># b   $ #</L>
      <L># #  #$ #</L>
      <L># . .#@ #</L>
      <L>#########</L>
    </Level>
  </LevelCollection>
</SokobanLevels>"##;
            
            let lsr = LevelSet::from_str(input_str).unwrap();
            let exp_lsr = LevelSet{ name: "Microban".to_string(),
            levels: vec![
                Ok(Level::from_str("funny", 6, 7,
                    "####  \
                     # .#  \
                     #  ###\
                     #*@  #\
                     #  $ #\
                     #  ###\
                     ####  ").unwrap()),
                Ok(Level::from_str("blocky", 6, 7,
                    "######\
                     #    #\
                     # #@ #\
                     # $* #\
                     # .* #\
                     #    #\
                     ######").unwrap()),
                Err(LevelParseError{ number: 2, name: "harder".to_string(),
                    error: WrongField(2, 2) }),
            ] };
            assert_eq!(exp_lsr, lsr);
    }
}
