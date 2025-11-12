	ORG 	$800
N:	DC.L	5
RESULT:	DS.L	1

	ORG	$1000
	
START:	
	MOVE.L #1, D0
	MOVE.L N, D1			; first instruction of program
	LSL.L D1, D0
	MOVE.L D0, RESULT
	
	TRAP	#15

	END	START		; last line of source