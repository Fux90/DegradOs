ENTRY(start)	/* entry point, il bootloader saltera qui dopo aver effettuao il load del kernel */

SECTIONS {
	. = 1M;		/* load address della prima sezione a 1MiB, posto conveniente per effettuare load del kernel */
	
	/* Due sezioni per l'eseguibile: boot e text */
	.rodata :
	{
		/* Fa in modo che l'header multiboot sia all'inizio */
		/* Necessario perchè GRUB si aspetta il multiboot header molto presto nel file */
		KEEP(*(.multibootheader))
		*(.rodata .rodata.*)
		/* Riallinea alle pagine le sezioni, rendendole multiple della dimensione di pagina */
		/* . significa 'Current Location Counter', cioè l'indirizzo virtuale corrente */
		. = ALIGN(4K);
	}
	
	.text :
	{
		/* Questa sezione contiene tutte le sezioni chiamate .text */
		*(.text .text.*)
		. = ALIGN(4K);
	}
	
	.data :
	{
		*(.data .data.*)
		. = ALIGN(4K);
	}

	.bss :
	{
		*(.bss .bss.*)
		. = ALIGN(4K);
	}

	.got :
	{
		*(.got)
		. = ALIGN(4K);
	}

	.got.plt :
	{
		*(.got.plt)
		. = ALIGN(4K);
	}
  
	.data.rel.ro : ALIGN(4K) {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }
    
    .gcc_except_table : ALIGN(4K) {
		*(.gcc_except_table)
		. = ALIGN(4K);
	}
}
