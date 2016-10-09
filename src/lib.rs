#![feature(lang_items)]
#![feature(const_fn, unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;

#[macro_use]
mod vga_buffer;

#[no_mangle]
pub extern fn rust_main() 
{
	//ATTENZIONE: stack piccolo e nessuna pagina di controllo

	let systemName = "DegradOS";
	let bitMode = 64;
	
	vga_buffer::clear_screen();	
	
	//print_DegradOS();
	
	blubbering(systemName, bitMode);
	vga_buffer::print_centered(systemName);
	
	loop{}
}

pub fn blubbering(systemName: &str, bitMode: u8) {
	println!("Hy, I'm {} {}bit!", systemName, bitMode);
	println!("Are you sure of your OS choice??");
	println!("Hey! I have those greenish characters like in The Matrix!!");
	println!("Cool!!");
	println!("");
	println!("Call me Neo.");
}

/* VECCHIO MODO
pub fn print_DegradOS()
{
	let systemWelcome = b"Degrado OS!!";
	let color_byte = 0x1e;
	
	let mut systemWelcome_colored = [color_byte; 24];
	for (i, char_byte) in systemWelcome.into_iter().enumerate() {
		systemWelcome_colored[i*2] = *char_byte;
	}
	
	// Scrive il messaggio al centro del VGA text buffer
	let buffer_ptr = (0xb8000 + 1988) as *mut _;
	unsafe { *buffer_ptr = systemWelcome_colored };
}
*/

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang =  "panic_fmt"] extern fn panic_fmt() -> !{loop{}}

// Fake function. Ricompileremo libcore with panic="abort".
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> !
{
	loop { }
}

