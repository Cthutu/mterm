use std::cmp::min;

use crate::PresentInput;

//
// Implements some methods for the PresentInput structure
//

impl<'a> PresentInput<'a> {
    pub fn blit(&mut self, p: Point, dst_width: usize, dst_height: usize, image: &Image) {
        let blitops = BlitOps {
            src: BlitRect::new(0, 0, image.width, image.height),
            dst: BlitRect::new(0, 0, self.width, self.height),
            src_blit: BlitRect::new(0, 0, image.width, image.height),
            dst_blit: BlitRect::new(p.x, p.y, dst_width, dst_height),
        };
        blit(&image.fore_image, &mut self.fore_image, &blitops);
        blit(&image.back_image, &mut self.back_image, &blitops);
        blit(&image.text_image, &mut self.text_image, &blitops);
    }

    pub fn blit_screen(&mut self, image: &Image) {
        self.blit(Point::new(0, 0), self.width, self.height, image);
    }
}

//
// Point
// An X, Y coordinate
//

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

//
// Char
// This represents a single ASCII character with an associated ink and paper colour.
//

#[derive(Debug, Clone, Copy)]
pub struct Char {
    pub ch: u8,
    pub ink: u32,
    pub paper: u32,
}

impl Char {
    pub fn new(ch: u8, ink: u32, paper: u32) -> Self {
        Char { ch, ink, paper }
    }
}

//
// RogueImage
// This represents a rectangular collection of RogueChars to render sprites and screens.
//

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub fore_image: Vec<u32>,
    pub back_image: Vec<u32>,
    pub text_image: Vec<u32>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        let size = (width * height) as usize;
        Image {
            width,
            height,
            fore_image: vec![0; size],
            back_image: vec![0; size],
            text_image: vec![0; size],
        }
    }

    pub fn coords_to_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn clip(&self, p: Point, width: usize, height: usize) -> (usize, usize, usize, usize) {
        let mut x = p.x;
        let mut y = p.y;
        let mut width = width;
        let mut height = height;
        if x < 0 {
            width += x as usize;
            x = 0;
        }
        if y < 0 {
            height += y as usize;
            y = 0;
        }
        let x = x as usize;
        let y = y as usize;
        width = min(width, self.width - x);
        height = min(height, self.height - y);

        (x, y, width, height)
    }

    pub fn clear(&mut self, ink: u32, paper: u32) {
        self.draw_rect_filled(
            Point::new(0, 0),
            self.width,
            self.height,
            Char::new(b' ', ink, paper),
        );
    }

    pub fn draw_char(&mut self, p: Point, ch: Char) {
        if p.x >= 0 && p.y >= 0 {
            if let Some(i) = self.coords_to_index(p.x as usize, p.y as usize) {
                self.fore_image[i] = ch.ink;
                self.back_image[i] = ch.paper;
                self.text_image[i] = ch.ch as u32;
            }
        }
    }

    pub fn draw_string(&mut self, p: Point, text: &str, ink: u32, paper: u32) {
        let (x, y, w, _) = self.clip(p, text.len(), 1);

        if let Some(i) = self.coords_to_index(x, y) {
            let w = w as usize;
            self.fore_image[i..i + w].iter_mut().for_each(|x| *x = ink);
            self.back_image[i..i + w]
                .iter_mut()
                .for_each(|x| *x = paper);
            self.text_image[i..i + w]
                .iter_mut()
                .enumerate()
                .for_each(|(j, x)| *x = (text.as_bytes()[j]) as u32);
        }
    }

    pub fn draw_rect(&mut self, p: Point, width: usize, height: usize, ch: Char) {
        if width < 3 || height < 3 {
            self.draw_rect_filled(p, width, height, ch);
        } else {
            // Draw top
            self.draw_rect_filled(p, width, 1, ch);
            // Draw bottom
            self.draw_rect_filled(Point::new(p.x, p.y + (height as i32) - 1), width, 1, ch);
            // Draw left
            self.draw_rect_filled(Point::new(p.x, p.y + 1), 1, height - 2, ch);
            // Draw right
            self.draw_rect_filled(
                Point::new(p.x + (width as i32) - 1, p.y + 1),
                1,
                height - 2,
                ch,
            );
        }
    }

    pub fn draw_rect_filled(&mut self, p: Point, width: usize, height: usize, ch: Char) {
        // Clip the coords and size to the image
        let (x, y, width, height) = self.clip(p, width, height);

        if let Some(mut i) = self.coords_to_index(x, y) {
            let width = width as usize;
            (0..height).for_each(|_| {
                // Render a row
                self.fore_image[i..i + width]
                    .iter_mut()
                    .for_each(|x| *x = ch.ink);
                self.back_image[i..i + width]
                    .iter_mut()
                    .for_each(|x| *x = ch.paper);
                self.text_image[i..i + width]
                    .iter_mut()
                    .for_each(|x| *x = ch.ch as u32);

                i += self.width as usize;
            });
        }
    }
}

//
// Blitting
//

struct BlitRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl BlitRect {
    fn new(x: i32, y: i32, width: usize, height: usize) -> Self {
        BlitRect {
            x,
            y,
            w: width as i32,
            h: height as i32,
        }
    }
}

struct BlitOps {
    src: BlitRect,      // Full size of the source rectangle (assume x, y is always 0, 0)
    dst: BlitRect,      // Full size of the destination rectangle (assume x, y is always 0, 0)
    src_blit: BlitRect, // Rectangle to blit from within src rectangle
    dst_blit: BlitRect, // Rectangle to blit to within dst rectangle
}

fn blit<T>(src: &Vec<T>, dst: &mut Vec<T>, ops: &BlitOps)
where
    T: Copy,
{
    let mut sx = ops.src_blit.x;
    let mut sy = ops.src_blit.y;
    let mut sw = ops.src_blit.w;
    let mut sh = ops.src_blit.h;
    let mut dx = ops.dst_blit.x;
    let mut dy = ops.dst_blit.y;
    let mut dw = ops.dst_blit.w;
    let mut dh = ops.dst_blit.h;

    // If the source blit area is before the full area, we need to clip to the
    // edge of the full area.
    if sx < 0 {
        sw += dx; // Shrink source
        dw += dx; // Shrink destination
        dx -= dx; // Shift the origin to the right
        sx = 0;
    }
    // If the source blit area is over the right edge of the full area, we need
    // to clip to the right edge of the full area.
    if sx + sw > ops.src.w {
        sw = ops.src.w - sx;
    }
    // Clip the source to the destination
    let width = min(sw, dw);

    // Now do the same for the Y axis
    if sy < 0 {
        sh += dy; // Shrink source
        dh += dy; // Shrink destination
        dy -= dy; // Shift the origin to the right
        sy = 0;
    }
    if sy + sh > ops.src.h {
        sh = ops.src.h - sy;
    }
    let height = min(sh, dh);

    if width > 0 && height > 0 {
        // Now we copy source into destination
        let mut si = sy * ops.src.w + sx;
        let mut di = dy * ops.dst.w + dx;

        (0..height).for_each(|_| {
            let src_slice = &src[si as usize..(si + width) as usize];
            let dst_slice = &mut dst[di as usize..(di + width) as usize];

            dst_slice.copy_from_slice(src_slice);

            si += ops.src.w;
            di += ops.dst.w;
        });
    }
}
