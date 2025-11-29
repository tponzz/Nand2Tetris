// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Fill.asm

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.

//// Replace this comment with your code.

(LOOP)
    @KBD
    D=M

    @SET_WHITE
    D; JEQ

    @SET_BLACK
    D; JNE

(SET_WHITE)
    @color
    M=0
    @INIT
    0; JMP

(SET_BLACK)
    @color
    M=-1

(INIT)
    @i
    M=0

(DRAW)
    @i
    D=M

    // 16 bits per line (1111111111111111 = -1)
    @SCREEN
    D=A+D
    @target
    M=D
    @color
    D=M
    @target
    A=M
    M=D

    // 32 lines
    @8191
    D=A
    @i
    D=D-M
    @LOOP
    D; JEQ

    @i
    M=M+1
    @DRAW
    0; JMP