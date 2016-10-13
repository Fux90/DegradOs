extern crate x86;

pub use self::entry::*;
pub use self::mapper::Mapper;

use self::temporary_page::TemporaryPage;
use self::table::{Table, Level4};

use core::ptr::Unique;
use core::ops::{Deref, DerefMut};

use memory::{PAGE_SIZE, Frame, FrameAllocator};

use multiboot2::BootInformation;


mod entry;
mod table;
mod temporary_page;
mod mapper;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;


// TESTING.

pub fn test_paging<A>(allocator: &mut A)
	where A: FrameAllocator
{
	let mut page_table = unsafe { ActivePageTable::new() };
	
	// Indirizzo 0 mappato.
	println!("Some = {:?}", page_table.translate(0));
	// Seconda entry di P1.
	println!("Some = {:?}", page_table.translate(4096));
	// Seconda entry di P2.
	println!("Some = {:?}", page_table.translate(512 * 4096));
	// 300esima entry di P2
	println!("Some = {:?}", page_table.translate(300 * 512 * 4096));
	// Seconda entry di P3
	println!("None = {:?}", page_table.translate(512 * 512 * 4096));
	// Ultimo byte mappato.
	println!("Some = {:?}", page_table.translate(512 * 512 * 4096 - 1));
	
	test_map_to_and_unmap(allocator, page_table);
	
	println!("");
}

fn test_map_to_and_unmap<A>(allocator: &mut A, mut page_table: ActivePageTable)
	where A: FrameAllocator
{
	// Map.
	println! ("\nTesting map.");
	
	let addr = 42 * 512 * 512 * 4096; // 42esima entry di p3
	let page = Page::containing_address(addr);
	let frame = allocator.allocate_frame().expect("No more frames.");
	
	println! ("None = {:?}, map to {:?}",
				page_table.translate(addr),
				frame);
				
	page_table.map_to(page, frame, EntryFlags::empty(), allocator);
	
	println! ("Some = {:?}.", page_table.translate(addr));
	println! ("Next free frame: {:?}", allocator.allocate_frame());
	
	// Unmap.
	println! ("\nTesting unmap.");
	
	println!("{:#x}", unsafe { *(Page::containing_address(addr).start_address() as *const u64) });
	page_table.unmap(Page::containing_address(addr), allocator);
	
	println! ("None = {:?}.", page_table.translate(addr));
}

// ---


// PAGE.

#[derive(Debug, Clone, Copy)]
pub struct Page {
	number: usize,
}

impl Page {
	pub fn containing_address(address: VirtualAddress) -> Page {
		assert!(address < 0x0000_8000_0000_0000 ||
			address >= 0xffff_8000_0000_0000,
			"invalid address: 0x{:x}", address);
		Page { number: address / PAGE_SIZE }
	}
	
	fn start_address(&self) -> usize {
		self.number * PAGE_SIZE
	}
	
	fn p4_index(&self) -> usize {
		(self.number >> 27) & 0o777
	}
	
	fn p3_index(&self) -> usize {
		(self.number >> 18) & 0o777
	}
	
	fn p2_index(&self) -> usize {
		(self.number >> 9) & 0o777
	}
	
	fn p1_index(&self) -> usize {
		(self.number >> 0) & 0o777
	}
}
// ---


// ACTIVE PAGE.

pub struct ActivePageTable {
	mapper: Mapper,
}

impl Deref for ActivePageTable {
	type Target = Mapper;
	
	fn deref(&self) -> &Mapper {
		&self.mapper
	}
}

impl DerefMut for ActivePageTable {
	fn deref_mut(&mut self) -> &mut Mapper {
		&mut self.mapper
	}
}

impl ActivePageTable {
	unsafe fn new() -> ActivePageTable {
		ActivePageTable {
			mapper: Mapper::new()
		}
	}
	
	pub fn with<F>( &mut self,
					table: &mut InactivePageTable,
					temporary_page: &mut temporary_page::TemporaryPage,
					f: F)
		 where F: FnOnce(&mut Mapper)
	{
		 use self::x86::{controlregs, tlb};
		 let flush_tlb = || unsafe { tlb::flush_all() };
	 
		{
			let backup = Frame::containing_address(
			unsafe { controlregs::cr3() } as usize);

			// Mappa temporary_page sulla tabella p4 corrente.
			let p4_table = temporary_page.map_table_frame(backup.clone(), self);

			// Overwrite del mapping ricorsivo.
			self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITEABLE);
			flush_tlb();

			f(self);

			self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITEABLE);
			flush_tlb();
		}
		 
		temporary_page.unmap(self);
	}
	 
	pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use self::x86::controlregs;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(unsafe { controlregs::cr3() } as usize),
        };
        unsafe {
            controlregs::cr3_write(new_table.p4_frame.start_address() as u64);
        }
        old_table
    }
}

// ---

// INACTIVE PAGE

pub struct InactivePageTable {
	p4_frame: Frame,
}

impl InactivePageTable {
	pub fn new(frame: Frame,
				active_table: &mut ActivePageTable,
				temporary_page: &mut TemporaryPage) 
		-> InactivePageTable 
	{
		{
			let table = temporary_page.map_table_frame( frame.clone(),
														active_table);
														
			table.zero();
			table[511].set(frame.clone(), PRESENT | WRITEABLE);
		}
		temporary_page.unmap(active_table);
		
		InactivePageTable {p4_frame: frame }
	}
}

// ---

// KERNEL REMAPPING.

pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation)
	where A: FrameAllocator
{
	let mut temporary_page = TemporaryPage::new( Page{ number: 0xcafebabe }, 
												 allocator);
												 
	let mut active_table = unsafe { ActivePageTable::new() };
	let mut new_table = {
		let frame = allocator.allocate_frame().expect("No more frames.");
		InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
	};
	
	active_table.with(&mut new_table, &mut temporary_page, |mapper| {
		let elf_sections_tag = boot_info.elf_sections_tag()
										.expect("Memory map tag required.");
										
		for section in elf_sections_tag.sections() {
			use self::entry::WRITEABLE;
			
			if !section.is_allocated() {
				// La sezione non è caricata in memoria.
				continue; 
			}
			
			assert! (section.addr as usize % PAGE_SIZE == 0, "Le sezioni devono essere allineate alle pagine.");
			
			println! (	"Mapping section at addre: {:#x}, size: {:#x}",
						section.addr,
						section.size);
						
			let flags = WRITEABLE;
			
			let start_frame = Frame::containing_address(section.start_address());
			let end_frame = Frame::containing_address(section.end_address() - 1);
			for frame in Frame::range_inclusive(start_frame, end_frame) {
				mapper.identity_map(frame, flags, allocator);
			}
		}
	});
}

// ---
