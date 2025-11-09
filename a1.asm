; ============================================================
; Programm: Potenzberechnung 2^n
; ============================================================

            ORG     $0800
N_VALUE:    DC.L    8         
RESULT:     DS.L    1           

            ORG     $1000

START:      MOVE.L  #1, D0         
            MOVEA.L #N_VALUE, A0  
            MOVE.L  (A0), D1       
            CMP.L   #0, D1
            BEQ     DONE          

LOOP:       MULS    #2, D0        
            SUBQ.L  #1, D1        
            BNE     LOOP         

DONE:       MOVEA.L #RESULT, A1   
            MOVE.L  D0, (A1)       
            SIMHALT

            END     START