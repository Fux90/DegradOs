global start	; Esporta (render pubblica) un'etichetta.
extern long_mode_start

section .text	; Sezione text, default per codice eseguibile.
bits 32			; Specifica che le linee seguenti sono istruzioni a 32 bit.
start:
	mov esp, stack_top 	; Aggiorniamo il registro esp (stack pointer) 
						; a stack_top (NB: cresce verso il basso!!).

	mov edi, ebx		; Inserisce l'info pointeer di Multiboot in EDI
						; (primo intero o ptr passato a funzione)
	; Controlli.
	call check_multiboot
	call check_cpuid
	call check_long_mode
	
	; Paging.
	call set_up_page_tables
	call enable_paging
	
	; Abilit SSE.
	call set_up_SSE
	
	; Carica la GDT a 64 bit.
	lgdt [gdt64.pointer]
	
	; Aggiorna i selettori.
	mov ax, gdt64.data
	mov ss, ax	; Stack selector.
	mov ds, ax	; Data selector.
	mov es, ax	; Extra selector.
	
	jmp gdt64.code:long_mode_start

set_up_page_tables:
	; Mappa la prima entry di P4 alla table P3.
	mov eax, p3_table
	or eax, 0b11 ; Presente + Scrivibile.
	mov [p4_table], eax
	
	; Mappa la prima entry di P3 alla table P2.
	mov eax, p2_table
	or eax, 0b11 ; Presente + Scrivibile.
	mov [p3_table], eax
	
	; Mappa ogni entry di P2 a una huge page di 2 MiB.
	mov ecx, 0 ; Contatore.
	
.map_p2_table:
	; Mappa l'entry ecx-esima di P2 ad una huge page che comincia allo
	; indirizzo 2 MiB * ecx
	mov eax, 0x200000
	mul ecx							; Indirizzo iniziale di ogni ecx-esima pagina.
	or eax, 0b10000011				; Presente + Scrivibile + Huge.
	mov [p2_table + ecx * 8], eax	; Mappa la ecx-esima pagina.
	
	inc ecx
	cmp ecx, 512
	jne .map_p2_table				; Mappa l'entry successiva, 
									; se non ne sono state mappate 512.
	ret
	
	
enable_paging:
	; Caricare P$ nel registro CR3 (usato dalla CPU per accedere alla tab).
	mov eax, p4_table
	mov cr3, eax
	
	; Abilitare PAE-flag in cr4 (Physical Address Extension).
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax
	
	; Impostare il bit LongMode nel EFER MSR (Model Specific Register).
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr
	
	; Abilita il pagine nel registro cr0.
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax
	
	ret
	
	
; Stampa ERR a il codice di errore,
; il parametro e il codice di errore (ASCII) nel registro al.	
error:
	mov dword [0xb8000], 0x4f524f45	; Indirizzo 0xb000 è l'inizio del VGA text buffer.
	mov dword [0xb8004], 0x4f3a4f52
	mov dword [0xb8008], 0x4f204f20
	mov byte [0xb800a], al
	hlt

; -
; FUNZIONI DI CONTROLLO
; -

; Controlla che EAX contenga il MagicNumber dei Multiboot - Throws ERR 0.
check_multiboot: 
	cmp eax, 0x36d76289
	jne .no_multiboot
	ret
.no_multiboot:
	mov al, "0"
	jmp error
; -------------------------------------------------------

; Controlla se CPUID e supportata tentando il flip del bit ID (bit 21) nel
; registro FLAGS. Se possibile, CPUID e disponibile. Altrimenti Throws ERR 1.
check_cpuid:
    ; Copia FLAGS in EAX via stack.
    pushfd
    pop eax
 
    ; Copia in ECX per successivi confronti.
    mov ecx, eax
 
    ; Flip del bit ID.
    xor eax, 1 << 21
 
    ; Copia EAX in FLAGS via stack.
    push eax
    popfd
 
    ; Copia FLAGS di nuovo EAX (con il flipped bit se CPUID supportato).
    pushfd
    pop eax
 
    ; Restore FLAGS dalla vecchia versione in ECX (i.e. risistemare l'ID 
    ; bit, se mai fosse stato flippato).
    push ecx
    popfd
 
    ; Paragonare EAX e ECX. Se uguali il bit non ha subito flipping
    ; e il CPUID non è supportato.
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
	mov al, "1"
	jmp error
; -------------------------------------------------------

; Controlla se la CPU supporta la Long Mode - Throw ERR 2.
check_long_mode:
	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb .no_long_mode
	
	mov eax, 0x80000001
	cpuid
	test edx, 1 << 29	; Controlla se il bit LM è settato nel registro D.
	jz .no_long_mode	; Se non lo fosse, non ci sarebbe long mode.
	
	ret
.no_long_mode:
	mov al, "2"
	jmp error
; -------------------------------------------------------

;Controlla SSe e lo abilita. Throw a
set_up_SSE:
	mov eax, 0x1
	cpuid
	test edx, 1<<25
	jz .no_SSE
	
	; Abilita SSE se presente.
	mov eax, cr0
	and ax, 0xfffb	; Clear dell'emulazione coprocessore CR0.EM
	or ax, 0x2		; setta il monitoring del coprocessore CR0.MP
	mov cr0, eax
	mov eax, cr4
	or ax, 3 << 9	; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
    mov cr4, eax

    ret
.no_SSE:
	mov al, "a"
	jmp error
; -------------------------------------------------------

		
; STACK -------------------------------------------------
section .bss
align 4096		; Questo assicura che le page tables siano allineate.
p4_table: 		; Page-Map Level 4 (PML4).
	resb 4096
p3_table:		; Page-Directory Pointer Table (PDP).
	resb 4096
p2_table:		; Page-Directory Table (PD).
	resb 4096 
stack_bottom:
	resb 4096	; reserve byte, in particolare 4096 bit.
stack_top:
; END STACK ---------------------------------------------

; GDT (Global Descriptor Table --------------------------
section .rodata
gdt64:
	dq 0												; Zero entry.
.code: equ $ - gdt64
	dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53)	; Code segment (dq - define quad).
.data: equ $ - gdt64
	dq (1<<44) | (1<<47) | (1<<41)						; Data segment.
.pointer:
	dw $ - gdt64 - 1	; $ viene sostituito dall'indirizzo corrente (.pointer qui).
	dq gdt64
; END GDT -----------------------------------------------
