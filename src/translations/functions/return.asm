@5
D=A
@LCL
A=M-D
D=M
@R14
M=D

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
@R14
A=M
0;JMP
