section .multibootheader
header_start:
	dd 0xe85250d6					; magic number, dd sta per define double 
	dd 0							; 0 -> i386
	dd header_end - header_start	; lunghezza dell'header
	; checksum
	dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))
	
	; tags multiboot opzionali
	
	; tag fine richiesto
	dw 0	; tipo, dw sta per define word
	dw 0	; flags
	dd 8	; dimensione
header_end:
