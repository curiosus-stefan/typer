extern crate rusttype;
extern crate unicode_normalization;

use std::char;
use self::rusttype::*;
use self::unicode_normalization::UnicodeNormalization;
use super::*;


use units::ColorRGBA;

pub struct TextRenderer<'a> {
	pub background: ColorRGBA,
	pub width: usize,
	pub height: usize,
	pub break_word: bool,
	pub padding: (usize, usize, usize, usize),

	lines: Vec<(Vec<(ScaledGlyph<'a>, Chunk)>, f32, f32)>,
	current_line: Vec<(ScaledGlyph<'a>, Chunk)>,
	advance_height: f32,
	line_width: f32,
}



impl <'a> TextRenderer<'a> {
	pub fn new () -> Self {
		Self {
			background: [255,255,255,255],
			width: 0,
			height: 0,
			break_word: false,
			padding: (0, 0, 0, 0),

			lines: Vec::new(),
			current_line: Vec::new(),
			advance_height: 0.0,
			line_width: 0.0,
		}
	}


	fn find_font(name: &Option<String>, fonts: &'a[Font<'a>]) -> &'a Font<'a> {
		match name {
			None => {&fonts[0]}
			Some(font_name) => {
				for font in fonts {
					for (data, _, _) in font.font_name_strings() {
						let res: String = data
							.iter()
							.map(|e| *e as char)
							.collect();
						// println!("{}", res);
						if res == *font_name {return font;}
					}
				}
				&fonts[0]
			}
		}
	}


	fn nwe_line(&mut self, line_width: f32, advance_height: f32) {
		self.lines.push((self.current_line.clone(), line_width, advance_height));
		self.current_line = Vec::new();
		self.line_width = 0.0;
		self.advance_height = 0.0;
	}


	pub fn render(&mut self, chunks: Vec<Chunk>, fonts: &'a[Font<'a>], dpi_factor: f32) -> ImgBuffer {

		// calc lines
		let mut font = TextRenderer::find_font(&chunks[0].font, fonts);
		let mut current_font_name = chunks[0].font.clone();
		let mut v_metrics;
		let mut scale = Scale::uniform(0.0);
		let mut word_width = 0.0;
		let mut current_word: Vec<(ScaledGlyph, Chunk)> = Vec::new();

		for chunk in &chunks {

			if !eq_font(&current_font_name, &chunk.font) {
				font = TextRenderer::find_font(&chunk.font, fonts);
				current_font_name = chunk.font.clone();
			}

			for letter in chunk.string.nfc() {
				if let Some(font_size) = chunk.font_size {
					scale = Scale::uniform(font_size as f32 * dpi_factor);
					v_metrics = font.v_metrics(scale);
					self.advance_height = self.advance_height.max(v_metrics.ascent - v_metrics.descent + v_metrics.line_gap);
					// println!("{}", self.advance_height);
				}

				let base_glyph = font.glyph(letter);
				let mut glyph = base_glyph.scaled(scale);

				let is_break = LINE_BREAK
					.iter()
					.find(|e| **e == letter)
					.is_some();
				if is_break {
					// new line
					let w = self.line_width;
					let h = self.advance_height;
					self.nwe_line(w, h);
				}
				let h_metrics = glyph.h_metrics();
				self.line_width += h_metrics.advance_width;

				if self.width != 0 {
					if self.break_word {
						let is_break = CAN_LINE_BREAK
							.iter()
							.find(|e| **e == letter)
							.is_some();

						if self.line_width > self.width as f32 && is_break {
							self.current_line.append(&mut current_word);
							current_word = Vec::new();
							word_width = 0.0;

							// new line
							let w = self.line_width - h_metrics.advance_width;
							let h = self.advance_height;
							self.nwe_line(w, h);
						} else if self.line_width > self.width as f32 {
							// new line
							let w = self.line_width - h_metrics.advance_width - word_width;
							let h = self.advance_height;
							self.nwe_line(w, h);

							current_word.push((glyph, chunk.duplicate()));
							word_width+=h_metrics.advance_width;
						} else if is_break {
							self.current_line.append(&mut current_word);
							self.current_line.push((glyph, chunk.duplicate()));
							current_word = Vec::new();
							word_width = 0.0;
						} else {
							current_word.push((glyph, chunk.duplicate()));
							word_width += h_metrics.advance_width;
						}
					} else {
						if self.line_width > self.width as f32 {
							// new line
							let w = self.line_width - h_metrics.advance_width;
							let h = self.advance_height;
							self.nwe_line(w, h);
						}
						self.current_line.push((glyph, chunk.duplicate()));
					}
				} else {
					self.current_line.push((glyph, chunk.duplicate()));
				}
			}
		}

		let w = self.line_width;
		let h = self.advance_height;
		self.nwe_line(w, h);

		for (_, w,h) in self.lines.iter() {
			println!("---- {}x{}", w.ceil() as usize, h.ceil() as usize);
		}
		println!("lines: {}", self.lines.len());

		let mut caret = point(0.0, 0.0);
		let mut img_width = self.width;
		let mut img_height = self.height;

		if img_width == 0 {
			let width: f32 = self.lines.iter().map(|(_,w,_)| -> &f32 {w} ).sum();
			img_width =  width.ceil() as usize;
		}

		if img_height == 0 {
			let width: f32 = self.lines.iter().map(|(_,_,h)| -> &f32 {h} ).sum();
			img_height =  width.ceil() as usize;
		}

		println!("img_width:{}, img_height:{}", img_width, img_height);

		let mut buffer = ImgBuffer::new(img_width, img_height, &self.background);
		let mut last_glyph_id = None;
		let mut color = [0,0,0,255];
		let mut bg = self.background;

		for (line, width, height) in self.lines.iter_mut() {
			println!("---- {}", height);

			last_glyph_id = None;
			caret.y += *height;
			caret.x = 0.0;


			for (scaled_glyph, chunk) in line.drain(..) {
				if !eq_font(&current_font_name, &chunk.font) {
					font = TextRenderer::find_font(&chunk.font, fonts);
					current_font_name = chunk.font.clone();
				}

				if let Some(id) = last_glyph_id {
					caret.x += font.pair_kerning(scale, id, scaled_glyph.id());
				}
				let mut glyph = scaled_glyph.positioned(caret);

				if let Some(c_color) = chunk.color {
					color = c_color;
				}

				if let Some(bounding_box) = glyph.pixel_bounding_box() {
					glyph.draw(|x, y, v| {
						let x = (bounding_box.min.x+(x as i32)) as usize;
						let y = (bounding_box.min.y+(y as i32)) as usize;

						buffer.put_pixel_alpha_blend(x, y, &color, v);
					});
				}

				last_glyph_id = Some(glyph.id());
				caret.x += glyph.unpositioned().h_metrics().advance_width;
			}

		}

		buffer
	}

}



fn eq_font<'a>(a: &Option<String>, b: &Option<String>) -> bool {
	match (a, b) {
		(Some(na), Some(nb)) => {na==nb}
		(Some(na), None) => {false}
		(None, Some(na),) => {false}
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
