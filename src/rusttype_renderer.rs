extern crate rusttype;
extern crate unicode_normalization;

use std::path::PathBuf;
use std::char;
use self::rusttype::{ScaledGlyph, PositionedGlyph, Glyph, GlyphId, GlyphIter, Scale};
use self::unicode_normalization::UnicodeNormalization;
use super::*;


// use std::cmp::Ordering;

use units::ColorRGBA;

pub struct TextRenderer {
// pub struct TextRenderer<'a> {
	pub break_word: bool,
	pub padding: (usize, usize, usize, usize),
	// pub fonts: &'a[(String, Font<'a>)],

	// lines: Vec<Line<'a>>,
	// current_line: Vec<(ScaledGlyph<'a>, Chunk, char)>,
	line_height: f32,
	line_width: f32,
	descent: f32,
}


pub fn is_line_break(c: char) -> bool {
	LINE_BREAK
		.iter()
		.find(|e| **e == c)
		.is_some()
}

pub fn is_can_line_break(c: char) -> bool {
	CAN_LINE_BREAK
		.iter()
		.find(|e| **e == c)
		.is_some()
}


// struct Line<'a> {
// 	glyphs: Vec<(ScaledGlyph<'a>, Chunk, char)>,
// 	text_align: TextAlignHorizontal,
// 	descent: f32,
// 	height: f32,
// 	chars_width: f32,
// 	x: f32,
// 	y: f32,
// }
// struct Line {
// struct Line<'a> {
// 	pub width: f32,
// 	pub descent: f32,
// 	pub height: f32,
// 	pub chars_width: f32,
// 	pub glyphs: Vec<(ScaledGlyph<'a>, &'a FormatChunk, char)>,
// 	// text_align: TextAlignHorizontal,
// 	// x: f32,
// 	// y: f32,
// }

impl TextRenderer {
// impl <'a> TextRenderer<'a> {
	pub fn new () -> Self {
		Self {
			break_word: false,
			padding: (0, 0, 0, 0),
			// fonts,
			// lines: Vec::new(),
			// current_line: Vec::new(),
			line_height: 0.0,
			line_width: 0.0,
			descent: 0.0,
		}
	}


	pub fn load_fonts <'a>(data: Vec<(String, PathBuf)>) -> Vec<(String, Font<'a>)> {
		let mut result = Vec::with_capacity(data.len());
		for (name, path) in data {
			let mut f = File::open(path).expect("wrong font path");
			let mut buffer = Vec::new();
			f.read_to_end(&mut buffer).expect("cant read from font file");
			let font = Font::from_bytes(buffer).expect("Error constructing Font");
			result.push((name, font));
		}
		result
	}


	fn find_font<'a>(&self, name: &Option<String>, fonts: &'a[(String, Font<'a>)] ) -> &'a Font<'a> {
		match name {
			None => {&fonts[0].1}
			Some(font_name) => {
				let res = fonts
					.iter()
					.find(|(name, _)| name == font_name );
				if let Some(font) = res {
					return &font.1;
				}
				&fonts[0].1
			}
		}
	}


	// fn nwe_line(&mut self, chars_width: f32) {

	// 	let text_align = if self.current_line.len() == 0 {
	// 		TextAlignHorizontal::Left
	// 	} else {
	// 		match self.current_line[0].1.text_align {
	// 			Some(align) => align,
	// 			_ => TextAlignHorizontal::Left,
	// 		}
	// 	};

	// 	let mut white_space = false;
	// 	{
	// 		if self.current_line.len() != 0 {
	// 			let (_, _, c) = self.current_line.last().unwrap();
	// 			white_space = is_can_line_break(*c);
	// 		}
	// 	}
	// 	if white_space {
	// 		self.current_line.pop();
	// 	}

	// 	self.lines.push(Line {
	// 		glyphs: self.current_line.clone(),
	// 		chars_width: chars_width,
	// 		height: self.line_height,
	// 		descent: self.descent,
	// 		text_align,
	// 		x: 0.0,
	// 		y: 0.0,
	// 	});
	// 	self.current_line = Vec::new();
	// 	self.line_width = 0.0;
	// 	self.line_height = 0.0;
	// 	self.descent = 0.0
	// }


	fn traverse <F: FnMut(&FormatChunks, &FormatChunk) > (&self, chunk: &FormatChunk, cb: &mut F) {
		// println!("{:?}", chunk);
		// let mut font = f;
		// if chunk.font != *f_name {
		// 	font = self.find_font(f_name);
		// }

		for chunk_i in &chunk.chunks {
			match chunk_i {
				// FormatChunks::String(s) => {
				// 	// for symbol in s.chars() {

				// 	// 	// block.add_symbol(chunk, symbol, font);
				// 	// }
				// }
				FormatChunks::Chunk(chunk_e) => {
					self.traverse(chunk_e, cb);
				}
				_=>{}
			}
			cb(chunk_i, chunk);
		}

	}
	// fn traverse<F:Fn(&str, &FormatChunk, &mut Line)> (chunk: &FormatChunk, cb: &mut F, block: &mut RenderBlock) {
	// 	// println!("{:?}", chunk);
	// 	for chunk_i in &chunk.chunks {
	// 		match chunk_i {
	// 			FormatChunks::String(s) => {
	// 				cb(s, chunk, block);
	// 			}
	// 			FormatChunks::Chunk(chunk_e) => {
	// 				Self::traverse(chunk_e, cb, block);
	// 			}
	// 		}
	// 	}

	// }


	pub fn render<'a>(&mut self, format_blocks: Vec<FormatBlock>, dpi_factor: f32, fonts: &'a[(String, Font<'a>)]) {

		let mut layout = Layout {
			blocks: Vec::with_capacity(format_blocks.len()),
		};

		let mut current_font_name = Some("".to_string());
		let mut font = self.find_font(&current_font_name, fonts);
		let mut scale = Scale::uniform(0.0);
		let mut line_width = 0.0;
		// let mut prev_glyph_id = None;

        for block in format_blocks {
			println!("------------------BLOCK-------------------");

			let mut render_block = RenderBlock::new();
			// let mut line = Line::n/ew();

			for (chunk, str_data) in block.chunk.iter() {
				if chunk.font != current_font_name {
					font = self.find_font(&current_font_name, fonts);
					current_font_name = chunk.font.clone();
				}

				scale = Scale::uniform(chunk.font_size as f32 * dpi_factor);
				let v_metrics = font.v_metrics(scale);

				for symbol in str_data.chars() {
					let line_break_symbol = is_line_break(symbol);
					if line_break_symbol {
						render_block.add_line();
						// prev_glyph_id = None;
						continue;
					}

					let line = render_block.get_line();

					line.height = line.height.max( (v_metrics.line_gap + v_metrics.ascent) * chunk.line_height );
					line.descent = line.descent.min(v_metrics.descent);

					let base_glyph = font.glyph(symbol);
					let mut glyph = base_glyph.scaled(scale);
					let h_metrics = glyph.h_metrics();
					let symbol_width = h_metrics.advance_width;

					// if let Some(id) = prev_glyph_id {
					// 	symbol_width += font.pair_kerning(scale, id, glyph.id());
					// }
					// prev_glyph_id = Some(glyph.id());


					if block.width == 0.0 {
						line.glyphs.push((glyph, chunk, symbol));
						continue;
					} else {
						
					}





				}


				print!("{}", str_data);
			}


			// self.traverse(&block.chunk, &mut |chunk, parent_chunk| {


			// 	match chunk {
			// 		FormatChunks::String(str_data) => {
			// 			print!("{}", str_data);

			// 			for symbol in str_data.chars() {

			// 				scale = Scale::uniform(parent_chunk.font_size as f32 * dpi_factor);
			// 				let v_metrics = font.v_metrics(scale);

			// 				let line_break_symbol = is_line_break(symbol);
			// 				if line_break_symbol {
			// 					render_block.add_line();
			// 				}
			// 				let line = render_block.get_line();

			// 				line.height = line.height.max( (v_metrics.line_gap + v_metrics.ascent) * parent_chunk.line_height );
			// 				line.descent = line.descent.min(v_metrics.descent);

			// 				if line_break_symbol {return;}

			// 				let base_glyph = font.glyph(symbol);
			// 				let mut glyph = base_glyph.scaled(scale);

			// 				if block.width == 0.0 {
			// 					// line.glyphs.push((glyph, parent_chunk, symbol));
			// 					return;
			// 				}



			// 				// let h_metrics = glyph.h_metrics();
			// 				// line_width += h_metrics.advance_width;
			// 				// // TODO: pair symbols font kerning

			// 				// if line_width > block.width {

			// 				// }




			// 			}
			// 		}
			// 		FormatChunks::Chunk(_) => {}
			// 	}
			// 	println!("---");
			// });






			// render_block.lines.push(line);
			layout.blocks.push((block, render_block));
        }





		// // calc lines
		// let mut v_metrics;
		// let mut scale = Scale::uniform(0.0);
		// let mut word_width = 0.0;
		// let mut current_word: Vec<(ScaledGlyph, Chunk, char)> = Vec::new();

		// for chunk in &chunks {

		// 	if !eq_font(&current_font_name, &chunk.font) {
		// 		font = TextRenderer::find_font(&chunk.font, fonts);
		// 		current_font_name = chunk.font.clone();
		// 	}

		// 	for letter in chunk.string.nfc() {
		// 		if let Some(font_size) = chunk.font_size {
		// 			scale = Scale::uniform(font_size as f32 * dpi_factor);
		// 			v_metrics = font.v_metrics(scale);
		// 			if let Some(mul) = chunk.line_height {
		// 				self.line_height = self.line_height.max((v_metrics.line_gap + v_metrics.ascent) * mul);
		// 				self.descent = self.descent.min(v_metrics.descent);
		// 			}
		// 		}

		// 		let is_break = is_line_break(letter);

		// 		let base_glyph = font.glyph(letter);
		// 		let mut glyph = base_glyph.scaled(scale);

		// 		let h_metrics = glyph.h_metrics();
		// 		self.line_width += h_metrics.advance_width;

		// 		if self.width != 0 {
		// 			if self.break_word {

		// 				if is_break {
		// 					self.current_line.append(&mut current_word);
		// 					let w = self.line_width - h_metrics.advance_width;
		// 					self.nwe_line(w);
		// 					current_word = Vec::new();
		// 					word_width = 0.0;
		// 				} else if self.line_width > self.width as f32 {
		// 					if is_can_line_break(letter) {
		// 						self.current_line.append(&mut current_word);
		// 						current_word = Vec::new();
		// 						word_width = 0.0;

		// 						let w = self.line_width - h_metrics.advance_width;
		// 						self.nwe_line(w);
		// 					} else {
		// 						current_word.push((glyph, chunk.duplicate(), letter));
		// 						word_width += h_metrics.advance_width;

		// 						let w = self.line_width - word_width;
		// 						self.nwe_line(w);

		// 						self.current_line.append(&mut current_word);
		// 						self.line_width += word_width;
		// 						current_word = Vec::new();
		// 						word_width = 0.0;
		// 					}
		// 				} else {
		// 					if is_can_line_break(letter) {
		// 						self.current_line.append(&mut current_word);
		// 						current_word = Vec::new();
		// 						word_width = 0.0;
		// 						self.current_line.push((glyph, chunk.duplicate(), letter));
		// 					} else {
		// 						current_word.push((glyph, chunk.duplicate(), letter));
		// 						word_width+=h_metrics.advance_width;
		// 					}
		// 				}
		// 			} else {
		// 				if is_break {
		// 					let w = self.line_width - h_metrics.advance_width;
		// 					self.nwe_line(w);
		// 				} else if self.line_width > self.width as f32 {
		// 					let w = self.line_width - h_metrics.advance_width;
		// 					self.nwe_line(w);

		// 					if !is_can_line_break(letter) {
		// 						self.current_line.push((glyph, chunk.duplicate(), letter));
		// 						self.line_width += h_metrics.advance_width;
		// 					}
		// 				} else {
		// 					self.current_line.push((glyph, chunk.duplicate(), letter));
		// 				}
		// 			}
		// 		} else if is_break {
		// 			let w = self.line_width - h_metrics.advance_width;
		// 			self.nwe_line(w);
		// 			// self.current_line.push((glyph, chunk.duplicate()));
		// 		} else {
		// 			self.current_line.push((glyph, chunk.duplicate(), letter));
		// 		}
		// 	}
		// }

		// let w = self.line_width;
		// self.nwe_line(w);

		// for Line{chars_width, height, ..} in self.lines.iter() {
		// 	println!("---- {}x{}", chars_width.ceil() as usize, height.ceil() as usize);
		// }
		// println!("lines: {}", self.lines.len());

		// let mut caret = point(0.0, 0.0);
		// let mut img_width = self.width;
		// let mut img_height = self.height;

		// if img_width == 0 {
		// 	let mut l_width: f32 = 0.0;
		// 	self.lines
		// 		.iter()
		// 		.for_each(|Line{chars_width, ..}| if *chars_width > l_width { l_width = *chars_width });
		// 	img_width =  l_width.ceil() as usize;
		// }

		// if img_height == 0 {
		// 	let height: f32 = self.lines.iter().map(|Line{height, ..}| -> &f32 {height} ).sum();
		// 	let last = self.lines.last().unwrap();
		// 	img_height =  ( height - last.descent ).ceil() as usize;
		// }

		// println!("img_width:{}, img_height:{}", img_width, img_height);

		// let mut buffer = ImgBuffer::new(img_width, img_height, &self.background);
		// let mut last_glyph_id = None;
		// let mut color = [0,0,0,255];
		// let mut justify = 0.0;

		// for Line {glyphs, height, text_align, chars_width, ..} in self.lines.iter_mut() {

		// 	last_glyph_id = None;
		// 	caret.y += *height;
		// 	justify = 0.0;

		// 	match text_align {
		// 		TextAlignHorizontal::Right => {
		// 			caret.x = (img_width as f32) - *chars_width;
		// 		}
		// 		TextAlignHorizontal::Center => {
		// 			caret.x = ((img_width as f32) - *chars_width) / 2.0;
		// 		}
		// 		TextAlignHorizontal::Left => {
		// 			caret.x = 0.0;
		// 		}
		// 		TextAlignHorizontal::Justify => {
		// 			caret.x = 0.0;
		// 			let w = (img_width as f32) - *chars_width;
		// 			let spases: f32 = glyphs
		// 				.iter()
		// 				.map(|(_, _, c)| if is_can_line_break(*c) {1.0} else {0.0})
		// 				.sum();
		// 			justify = w / spases;
		// 		}
		// 	}

		// 	for (scaled_glyph, chunk, letter, ..) in glyphs.drain(..) {
		// 		if !eq_font(&current_font_name, &chunk.font) {
		// 			font = TextRenderer::find_font(&chunk.font, fonts);
		// 			current_font_name = chunk.font.clone();
		// 		}

		// 		if let Some(id) = last_glyph_id {
		// 			caret.x += font.pair_kerning(scale, id, scaled_glyph.id());
		// 		}

		// 		if *text_align == TextAlignHorizontal::Justify {
		// 			if is_can_line_break(letter) {
		// 				caret.x += justify;
		// 			}
		// 		}

		// 		let mut glyph = scaled_glyph.positioned(caret);

		// 		if let Some(c_color) = chunk.color {
		// 			color = c_color;
		// 		}

		// 		if let Some(bounding_box) = glyph.pixel_bounding_box() {
		// 			glyph.draw(|x, y, v| {
		// 				let x = (bounding_box.min.x+(x as i32)) as usize;
		// 				let y = (bounding_box.min.y+(y as i32)) as usize;

		// 				buffer.put_pixel_alpha_blend(x, y, &color, v);
		// 			});
		// 		}

		// 		last_glyph_id = Some(glyph.id());
		// 		caret.x += glyph.unpositioned().h_metrics().advance_width;
		// 	}

		// }

		// buffer
	}

}



fn eq_font<'a>(a: &Option<String>, b: &Option<String>) -> bool {
	match (a, b) {
		(Some(na), Some(nb)) => {na == nb}
		(Some(_), None) => {false}
		(None, Some(_),) => {false}
		_ => false
	}
}

// http://www.fileformat.info/info/unicode/category/Zs/list.htm
// Unicode Characters in the 'Separator, Space' Category
// https://en.wikipedia.org/wiki/Whitespace_character
// http://www.unicode.org/Public/UNIDATA/UnicodeData.txt


static LINE_BREAK: &[char] = &['↵', '', '', '\n', '', ' ', ' '];

const CAN_LINE_BREAK: &[char] = &[
	' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '　'
	// 0x0020, as char ,	// SPACE;Zs;0;WS;;;;;N;;;;;
	// 0x1680, as char ,	// OGHAM SPACE MARK;Zs;0;WS;;;;;N;;;;;
	// 0x2000, as char ,	// EN QUAD;Zs;0;WS;2002;;;;N;;;;;
	// 0x2001, as char ,	// EM QUAD;Zs;0;WS;2003;;;;N;;;;;
	// 0x2002, as char ,	// EN SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x2003, as char ,	// EM SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x2004, as char ,	// THREE-PER-EM SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x2005, as char ,	// FOUR-PER-EM SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x2006, as char ,	// SIX-PER-EM SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x2008, as char ,	// PUNCTUATION SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x2009, as char ,	// THIN SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x200A, as char ,	// HAIR SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x205F, as char ,	// MEDIUM MATHEMATICAL SPACE;Zs;0;WS;<compat> 0020;;;;N;;;;;
	// 0x3000, as char ,	// IDEOGRAPHIC SPACE;Zs;0;WS;<wide> 0020;;;;N;;;;;
];