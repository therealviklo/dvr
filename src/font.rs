use std::cmp::{max, min};

use crate::{Dvr, Texture};

pub struct Font {
	sheets: Vec<FontSheet>,
	leading: f32,
	tofu_char: char,
}

impl Font {
	pub fn new(sheets: Vec<FontSheet>, leading: f32, tofu_char: char) -> Font {
		Font {
			sheets,
			leading,
			tofu_char,
		}
	}

	pub fn draw_text(&self, dvr: &Dvr, text: &str, x: f32, y: f32, max_w: Option<f32>, h: f32, text_align: Align, vert_align: VAlign) -> Result<(), String> {
		let text = match max_w {
			Some(max_w) => &self.auto_line_split(text, max_w, h)?,
			None => text,
		};
		let vert_align_factor: f32 = match vert_align {
			VAlign::Top => -0.5,
			VAlign::Centre => 0.0,
			VAlign::Bottom => 0.5,
		};
		let num_lines = self.calculate_num_lines(text);
		let mut y_offset: f32 = match vert_align {
			VAlign::Top => 0.0,
			VAlign::Centre => 0.5 * h * (num_lines as f32 - 1.0),
			VAlign::Bottom => h * (num_lines as f32 - 1.0),
		};
		for line in text.lines() {
			let mut x_offset: f32 = match text_align {
				Align::Left => 0.0,
				Align::Centre => -0.5 * self.calculate_line_width(line, h)?,
				Align::Right => -self.calculate_line_width(line, h)?,
			};
			for c in line.chars() {
				let (c, sheet) = self.get_char_and_sheet(c, h)?;
				let charw = sheet.get_char_width(c, h)?;
				let (cx, cy) = sheet.get_pos(c);
				let (cw, ch) = sheet.get_char_cell_size()?;
				let actual_height = h / (1.0 - sheet.extra_ascent - sheet.extra_descent);
				dvr.draw(
					&sheet.texture,
					x + x_offset + 0.5 * charw,
					y + y_offset + vert_align_factor * h + (sheet.extra_ascent - sheet.extra_descent) * h,
					Some((cw * actual_height / ch, actual_height)),
					Some(((cx as f32 * cw, cy as f32 * ch), (cw, ch))),
					0.0
				)?;
				x_offset += charw;
			}
			y_offset -= h * self.leading;
		}
		Ok(())
	}

	pub fn calculate_num_lines(&self, text: &str) -> usize {
		text.lines().count()
	}

	pub fn calculate_text_width(&self, text: &str, h: f32) -> Result<f32, String> {
		let mut max_width: f32 = 0.0;
		for line in text.lines() {
			let line_width = self.calculate_line_width(line, h)?;
			max_width = max_width.max(line_width);
		}
		Ok(max_width)
	}

	fn calculate_line_width(&self, line: &str, h: f32) -> Result<f32, String> {
		let mut width: f32 = 0.0;
		for c in line.chars() {
			let (c, sheet) = self.get_char_and_sheet(c, h)?;
			let charw = sheet.get_char_width(c, h)?;
			width += charw;
		}
		Ok(width)
	}

	fn auto_line_split(&self, text: &str, max_w: f32, h: f32) -> Result<String, String> {
		let mut new_text = String::new();
		let mut first = true;
		for line in text.lines() {
			if first {
				first = false;
			} else {
				new_text.push('\n');
			}
			let mut width = 0.0;
			for c in line.chars() {
				let (c, sheet) = self.get_char_and_sheet(c, h)?;
				let charw = sheet.get_char_width(c, h)?;
				width += charw;
				if width > max_w {
					new_text.push('\n');
					width = charw;
				}
				new_text.push(c);
			}
		}
		Ok(new_text)
	}

	/// This gets the character that will be printed and the sheet where it is.
	/// Usually the character that is printed is simply c but if there is a tofu character
	/// it may be the tofu character instead.
	fn get_char_and_sheet(&self, c: char, h: f32) -> Result<(char, &FontSheet), String> {
		let sheet = self.get_sheet(c);
		match sheet {
			Ok(_) => {},
			Err(e) => match self.tofu_char {
				'\0' => return Err(e),
				_ => {
					if c == self.tofu_char {
						return Err("Tofu character does not exist".to_string());
					}
					return self.get_char_and_sheet(self.tofu_char, h);
				}
			}
		}
		let sheet = sheet?;
		return Ok((c, sheet));
	}

	fn get_sheet(&self, c: char) -> Result<&FontSheet, String> {
		for sheet in &self.sheets {
			if c >= sheet.range.0 && c <= sheet.range.1 && sheet.char_widths[c as usize - sheet.range.0 as usize] != 0.0 {
				return Ok(&sheet);
			}
		}
		Err("Character not in font".to_string())
	}
}

pub enum Align {
	Left,
	Centre,
	Right,
}

pub enum VAlign {
	Bottom,
	Centre,
	Top,
}

pub struct FontSheet {
	range: (char, char),
	texture: Texture,
	chars_per_row: usize,
	rows: usize,
	extra_ascent: f32,
	extra_descent: f32,
	char_widths: Vec<f32>,
}

impl FontSheet {
	pub fn new(range: (char, char), texture: Texture, chars_per_row: usize, rows: usize, extra_ascent: f32, extra_descent: f32, char_widths: Vec<f32>) -> Result<FontSheet, String> {
		let range = (min(range.0, range.1), max(range.0, range.1));
		if char_widths.len() != (range.1 as usize - range.0 as usize + 1) {
			return Err("The number of character widths does not match the number of characters in the range".to_string());
		}
		Ok(FontSheet {
			range,
			texture,
			chars_per_row,
			rows,
			extra_ascent,
			extra_descent,
			char_widths: char_widths.iter().map(|x| 1.0 - 2.0 * x).collect(),
		})
	}

	fn get_char_cell_size(&self) -> Result<(f32, f32), String> {
		let (w, h) = self.texture.get_size();
		Ok(((w as usize / self.chars_per_row) as f32, (h as usize / self.rows) as f32))
	}

	fn get_char_width(&self, c: char, h: f32) -> Result<f32, String> {
		let (sheet_width, sheet_height) = self.texture.get_size();
		if c < self.range.0 || c > self.range.1 {
			return Err("Character not in sheet".to_string());
		}
		let charw = self.char_widths[c as usize - self.range.0 as usize];
		let scale = h * self.rows as f32 / sheet_height as f32  / (1.0 - self.extra_ascent - self.extra_descent);
		Ok((sheet_width as usize / self.chars_per_row) as f32 * charw * scale)
	}

	fn get_pos(&self, c: char) -> (usize, usize) {
		let char_index = c as usize - self.range.0 as usize;
		(
			char_index % self.chars_per_row,
			char_index / self.chars_per_row,
		)
	}
}
