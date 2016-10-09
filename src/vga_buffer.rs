use core::ptr::Unique;
use core::fmt::Write;
use volatile::Volatile;
use spin::Mutex;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;


// TEMPORANEO.
pub fn print_something() {
    let mut writer = Writer {
        column_position: 15,
        row_position: 12,
        color_code: ColorCode::new(Color::Yellow, Color::Blue),
        buffer: unsafe { Unique::new(0xb8000 as *mut _) },
    };

    writer.write_byte(b'H');
    writer.write_byte(b'y');
    
    write!(writer, "\nThe numbers are {} and {}", 42, 1.0/3.0);
}
// FINE TEMPORANEO.


// MACROS.
macro_rules! print {
    ($($arg:tt)*) => ({
            use core::fmt::Write;
            let mut writer = $crate::vga_buffer::WRITER.lock();
            writer.write_fmt(format_args!($($arg)*)).unwrap();
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
// ---


pub fn clear_screen() {
    for _ in 0..BUFFER_HEIGHT {
        println!("");
    }
}

pub fn print_centered(s: &str) {
	let len = s.len() + 4;
    let mut writer = Writer {
        column_position: 40 - (len / 2),
        row_position: 11,
        color_code: ColorCode::new(Color::Yellow, Color::Blue),
        buffer: unsafe { Unique::new(0xb8000 as *mut _) },
    };

	let sep = "*";
	write_separator(&mut writer, sep, len);
	write_string_in_separator(&mut writer, s, len, sep);
    write_separator(&mut writer, sep, len);
}

pub fn write_string_in_separator(writer: &mut Writer, s: &str, s_len: usize, sep: &str) {
	write! (writer, "{} {} {}", sep, s, sep);
	writer.down_one_row();
	writer.cursor_backward(s_len);
}

pub fn write_separator(writer: &mut Writer, sep: &str, len: usize) {
	writer.repeat_n_times(sep, len);
	writer.down_one_row();
	writer.cursor_backward(len);
}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
	column_position: 0,
	row_position: BUFFER_HEIGHT - 1,
	color_code: ColorCode::new(Color::LightGreen, Color::Black),
	buffer: unsafe { Unique::new(0xb8000 as *mut _) },
});


#[allow(dead_code)]
#[repr(u8)]
pub enum Color
{
	Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

// COLOR CODE
#[derive(Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
	const fn new(foreground: Color, background: Color) -> ColorCode {
		ColorCode((background as u8) << 4 | (foreground as u8))
	}
}
// ---


#[derive(Clone, Copy)]
#[repr(C)]	// Garantisce che i campi siano nell'ordine specificato
struct ScreenChar {
	ascii_character: u8,
	color_code: ColorCode,
}


struct Buffer {
	chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


// WRITER
pub struct Writer {
	column_position: usize,
	row_position: usize,
	color_code: ColorCode,
	buffer: Unique<Buffer>,
}

impl Writer {
	pub fn write_byte(&mut self, byte: u8) {
		match byte {
			b'\n' => self.new_line(),
			byte => {
				if self.column_position >= BUFFER_WIDTH {
					self.new_line();
				}
				
				let row = self.row_position;//BUFFER_HEIGHT - 1;
				let col = self.column_position;
				
				let color_code = self.color_code;
				self.buffer().chars[row][col].write(ScreenChar {
					ascii_character: byte,
					color_code: color_code,
				});
				
				self.column_position += 1;
			}
		}
	}

	pub fn write_string(&mut self, s: &str) {
		for byte in s.bytes() {
			self.write_byte(byte);
		}
	}
	
	fn buffer(&mut self) -> &mut  Buffer {
		unsafe { self.buffer.get_mut() }
	}
	
	/*
	fn new_line(&mut self) {
		for	row in 0..(BUFFER_HEIGHT-1) {
			let buffer = self.buffer();
			buffer.chars[row] = buffer.chars[row + 1];
		}
		
		self.clear_row(BUFFER_HEIGHT - 1);
		self.column_position = 0;
	}
	*/
	fn new_line(&mut self) {
		for row in 1..BUFFER_HEIGHT {
			for col in 0..BUFFER_WIDTH {
				let buffer = self.buffer();
				let character = buffer.chars[row][col].read();
				buffer.chars[row - 1][col].write(character);
			}
		}
		self.clear_row(BUFFER_HEIGHT-1);
		self.column_position = 0;
}

	fn down_one_row(&mut self) {
		self.row_position += 1;
		if (self.row_position == BUFFER_HEIGHT) {
			self.row_position -= 1;
		}
	}
	
	fn cursor_forward(&mut self, pos: usize) {
		self.column_position += pos;
		if (self.column_position >= BUFFER_WIDTH) {
			self.column_position = BUFFER_WIDTH - 1;
		}
	}
	
	fn cursor_backward(&mut self, pos: usize) {
		if (self.column_position < pos) {
			self.column_position = 0;
		}
		else {
			self.column_position -= pos;
		}
	}
	
	fn repeat_n_times(&mut self, s: &str, n: usize) { 
		for _ in 0..n {
			self.write_string(s);
		}
	}

	/*
	fn clear_row(&mut self, row: usize) { 
		let blank = ScreenChar {
			ascii_character: b' ',
			color_code: self.color_code,
		};
		
		self.buffer().chars[row] = [blank; BUFFER_WIDTH];
	}
	*/
	fn clear_row(&mut self, row: usize) {
		let blank = ScreenChar {
			ascii_character: b' ',
			color_code: self.color_code,
		};
		for col in 0..BUFFER_WIDTH {
			self.buffer().chars[row][col].write(blank);
		}
	}
}

impl ::core::fmt::Write for Writer {
	fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
		for byte in s.bytes() {
			self.write_byte(byte);
		}
		
		Ok(())
	}
}
// ---
