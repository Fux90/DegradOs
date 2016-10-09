ENTRY(start)	/* entry point, il bootloader saltera qui dopo aver effettuao il load del kernel */

SECTIONS {
	. = 1M;		/* load address della prima sezione a 1MiB, posto conveniente per effettuare load del kernel */
	
	/* Due sezioni per l'eseguibile: boot e text */
	.boot :
	{
		/* Fa in modo che l'header multiboot sia all'inizio */
		/* Necessario perchè GRUB si aspetta il multiboot header molto presto nel file */
		KEEP(*(.multibootheader))
	}
	
	.text :
	{
		/* Questa sezione contiene tutte le sezioni chiamate .text */
		*(.text)
	}
}
