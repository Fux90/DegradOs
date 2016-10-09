global long_mode_start

section .text
bits 64
long_mode_start:
	
	extern rust_main
	call rust_main
	
	; Begin (Set screen color)
	mov dword eax, 0x1e
	mov dword ebx, 0xb8000
	mov dword ecx, 2000 ; 80 * 25 chars - VGA Text-mode Display size
.colorOutput:
	mov byte[ebx], 0
	mov byte [ebx+1], al
	add ebx, 2
	loop .colorOutput
	; end
	
	mov dword ebx, 0xb8000
	add ebx, 1924
	; Stampa 
	; 44 65 67 72 Degr
	; 61 64 4f 53 adOS
	; 20 36 34 62 _64b
	; 69 74 21 21 it!!
	mov rax, 0x1e721e671e651e44
	mov qword [ebx], rax
	add ebx, 8
	mov rax, 0x1e531e4f1e641e61
	mov qword [ebx], rax
	mov rax, 0x1e621e341e361e20
	add ebx, 8
	mov qword [ebx], rax
	mov rax, 0x1e211e211e741e69
	add ebx, 8
	mov qword [ebx], rax
	hlt
