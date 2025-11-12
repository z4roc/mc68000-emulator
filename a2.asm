            ORG     $0800
N:	    DC.L    8         
RESULT:     DS.L    1
VALUE:	    DS.L    1        

	ORG $1000
recursion_pow2
	TST.L D1
	BNE .recursive
	
	MOVE.L #1, D0
	RTS
	
.recursive
	MOVE.L D0, VALUE
	SUBI.L #1, D1
	BSR recursion_pow2
	
	MOVE.L VALUE, D2
	MULS D2, D0
	RTS
	
START:
	MOVE.L #2, D0 ; 2^N
	MOVE.L N, D1
	BSR recursion_pow2
	
	MOVE.L D0, RESULT
	MOVE.L #9, D1
	TRAP #15
	END START