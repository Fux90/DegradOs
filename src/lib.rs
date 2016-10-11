#![feature(lang_items)]
#![feature(const_fn, unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
#[macro_use]
extern crate bitflags;

#[macro_use]
mod vga_buffer;
mod memory;

// PAGE FLAGS.

// ---

#[no_mangle]
pub extern fn rust_main(multiboot_info_pointer: usize) 
{
	//ATTENZIONE: stack piccolo e nessuna pagina di controllo

	let system_name = "DegradOS";
	let bit_mode = 64;
	
	vga_buffer::clear_screen();	
	
	//print_DegradOS();
	blubbering(system_name, bit_mode);
	
	let boot_info = unsafe { multiboot2::load(multiboot_info_pointer) };
	print_info(multiboot_info_pointer, boot_info);
	
	// Test frame allocation.
	frame_allocation_test(multiboot_info_pointer, boot_info);
	
	vga_buffer::print_centered(system_name);
	
	
	loop{}
}

pub fn blubbering(system_name: &str, bit_mode: u8) {
	println!("Hy, I'm {} {}bit!", system_name, bit_mode);
	println!("Are you sure of your OS choice??");
	println!("Hey! I have those greenish characters like in The Matrix!!");
	println!("Cool!!");
	println!("");
	println!("Call me Neo.");
}

pub fn print_info(	multiboot_info_pointer: usize,
					boot_info: &multiboot2::BootInformation) 
{
	print_multiboot_info(boot_info);
	//print_kernel_sections(boot_info);
	print_kernel_start_end(boot_info);
	print_multiboot_start_end(multiboot_info_pointer, boot_info);
}

pub fn print_multiboot_info(boot_info: &multiboot2::BootInformation) {
	let memory_map_tag = boot_info.memory_map_tag().expect("Memory tag required");
	
	println!("---");
	println! ("Memory areas (Aaaah!! I'm naked!!):");
	for area in memory_map_tag.memory_areas() {
		println!(	"    Start: 0x{:x}, Length: 0x{:x}",
					area.base_addr, 
					area.length);
	}
}

pub fn print_kernel_sections(boot_info: &multiboot2::BootInformation) {	
	let elf_sections_tag = boot_info.elf_sections_tag()
    .expect("Elf-sections tag required");
	println!("---");
	println!("Kernel sections:");
	for section in elf_sections_tag.sections() {
		println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
			section.addr, section.size, section.flags);
	}
}

pub fn print_kernel_start_end(boot_info: &multiboot2::BootInformation) {
	let elf_sections_tag = 	boot_info.elf_sections_tag()
							.expect("Elf-sections tag required");
    
	let kernel_start = 	elf_sections_tag.sections().map(|s| s.addr)
						.min().unwrap();
			
	let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size)
					 .max().unwrap();
			
	println! ("");
	println! ("Kernel start: {}", kernel_start);
	println! ("Kernel end: {}", kernel_end);
}

pub fn print_multiboot_start_end(multiboot_information_pointer: usize,
								 boot_info: &multiboot2::BootInformation) {
	let multiboot_start = multiboot_information_pointer;
	let multiboot_end = multiboot_start + (boot_info.total_size as usize);
	
	println! ("");
	println! ("Multiboot start: {}", multiboot_start);
	println! ("Multiboot end: {}", multiboot_end);
}

pub fn frame_allocation_test(multiboot_information_pointer: usize,
							 boot_info: &multiboot2::BootInformation) {
	let memory_map_tag = boot_info.memory_map_tag().expect("Memory tag required");
	
	let elf_sections_tag = 	boot_info.elf_sections_tag()
							.expect("Elf-sections tag required");
    
	let kernel_start = 	elf_sections_tag.sections().map(|s| s.addr)
						.min().unwrap();
			
	let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size)
					 .max().unwrap();
					 
	let multiboot_start = multiboot_information_pointer;
	let multiboot_end = multiboot_start + (boot_info.total_size as usize);
	
	let mut frame_allocator = memory::AreaFrameAllocator::new(
		kernel_start as usize,
		kernel_end as usize,
		multiboot_start,
		multiboot_end,
		memory_map_tag.memory_areas()
	);
	
	// Testing.
	memory::test_paging(&mut frame_allocator);
	// ---
	
	for i in 0.. {
        use memory::FrameAllocator;
        if let None = frame_allocator.allocate_frame() {
            println!("allocated {} frames", i);
            break;
        }
    }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang =  "panic_fmt"] 
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> !
{
	vga_buffer::clear_screen();
	
	println! ("=========================");
	println! ("Nooooooo!! Kernel panic!!");
	println! ("(Aha! If it had been blue, it would have worked)");
	println! ("");
	println! ("-------------------------");
	println!("PANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
	println! ("=========================");
	
	loop{}
}

// Fake function. Ricompileremo libcore with panic="abort".
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> !
{
	loop { }
}

