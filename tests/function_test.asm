@256
D=A
@SP
M=D
@global$ret.0
D=A
@SP
M=M+1
A=M-1
M=D

@LCL
D=M
@SP
M=M+1
A=M-1
M=D

@ARG
D=M
@SP
M=M+1
A=M-1
M=D

@THIS
D=M
@SP
M=M+1
A=M-1
M=D

@THAT
D=M
@SP
M=M+1
A=M-1
M=D

@5
D=A
@SP
D=M-D
@ARG
M=D

@SP
D=M
@LCL
M=D

@Sys.init
0;JMP

(global$ret.0)
// function Sys.init 0
(Sys.init)
@0
D=A
@SP
M=M+D
A=M-D

// push constant 2
@2
D=A
@SP
M=M+1
A=M-1
M=D

// push constant 3
@3
D=A
@SP
M=M+1
A=M-1
M=D

// call add_three 2
@Sys.init$ret.3
D=A
@SP
M=M+1
A=M-1
M=D

@LCL
D=M
@SP
M=M+1
A=M-1
M=D

@ARG
D=M
@SP
M=M+1
A=M-1
M=D

@THIS
D=M
@SP
M=M+1
A=M-1
M=D

@THAT
D=M
@SP
M=M+1
A=M-1
M=D

@7
D=A
@SP
D=M-D
@ARG
M=D

@SP
D=M
@LCL
M=D

@add_three
0;JMP

(Sys.init$ret.3)

// label loop
(function_test.Sys.init$loop)

// goto loop
@function_test.Sys.init$loop
0;JMP

// return
@SP
A=M-1
D=M
@ARG
A=M
M=D

@ARG
D=M
@SP
M=D+1

@LCL
D=M
@R13
M=D-1
A=M
D=M
@THAT
M=D

@R13
M=M-1
A=M
D=M
@THIS
M=D

@R13
M=M-1
A=M
D=M
@ARG
M=D

@R13
M=M-1
A=M
D=M
@LCL
M=D

@R13
A=M-1
A=M
0;JMP

// function add_three 2
(add_three)
@2
D=A
@SP
M=M+D
A=M-D
M=0
A=A+1
M=0
A=A+1

// push argument 1
@ARG
D=M
@1
A=D+A
D=M
@SP
M=M+1
A=M-1
M=D

// push argument 0
@ARG
D=M
@0
A=D+A
D=M
@SP
M=M+1
A=M-1
M=D

// add
@SP
M=M-1
A=M
D=M
A=A-1
M=M+D

// return
@SP
A=M-1
D=M
@ARG
A=M
M=D

@ARG
D=M
@SP
M=D+1

@LCL
D=M
@R13
M=D-1
A=M
D=M
@THAT
M=D

@R13
M=M-1
A=M
D=M
@THIS
M=D

@R13
M=M-1
A=M
D=M
@ARG
M=D

@R13
M=M-1
A=M
D=M
@LCL
M=D

@R13
A=M-1
A=M
0;JMP


