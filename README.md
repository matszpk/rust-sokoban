## Sokoban

This is simple Sokoban game written in the Rust language. The game is working terminal
environment and it provides simple user interface. A game display levels with standard
semigraphic characters and it requires ANSI terminal to display levels.
A game can load the levelset from file in
the standard XSB format and in the XML format (SLC).

### Usage

The game can be easily run by entering command:

`sokoban levelsetfile` - where levelsetfile is levelset file path.

Keys while game:

* Left, Right, Up, Down - move player.
* Backspace - undo last move.
* Escape - cancel current level.
* Q - quit game.
* F1, ? - display help.
