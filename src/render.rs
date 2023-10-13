use ggez::{Context, glam::Vec2, graphics::{Color, self, Canvas, TextFragment, Rect, Text, Drawable}};


pub const TEXT_COLOR:Color = Color::new(0.1, 0.1, 0.1, 1.0);
pub const WINDOW_BG:Color = Color::new(1.0, 1.0, 0.95, 1.0);
pub const TEXT_BG_COLOR:Color = Color::new(0.8, 0.8, 0.95, 0.5);
pub const LIGHT_TEXT_COLOR:Color = Color::new(0.7, 0.7, 0.7, 1.0);


/// shrinks a rectangle by v inwards on each side. each side decreases in length by 2v.
pub fn shrink(rect: Rect, v: f32) -> Rect {
    Rect { x: rect.x + v, y: rect.y + v, w: rect.w - 2.0 * v, h: rect.h - 2.0 * v }
}

/// cuts rectangle in half, measured from the bottom. returns: (TOP RECTANGLE, BOTTOM RECTANGLE)
pub fn cut_bottom(rect: Rect, height: f32) -> (Rect, Rect) {
    (
        Rect {x: rect.x, y: rect.y, w: rect.w, h: rect.h - height},
        Rect {x: rect.x, y: rect.y + rect.h - height, w: rect.w, h: height}
    )
}

/// cuts rectangle in half, measured from the top. returns: (TOP RECTANGLE, BOTTOM RECTANGLE)

pub fn cut_top(rect: Rect, height: f32) -> (Rect, Rect) {
    (
        Rect {x: rect.x, y: rect.y, w: rect.w, h: height},
        Rect {x: rect.x, y: rect.y + height, w: rect.w, h: rect.h - height}
    )
}
/// cuts rectangle in half, measured from the left. returns: (LEFT RECTANGLE, RIGHT RECTANGLE)
pub fn cut_left(rect: Rect, width: f32) -> (Rect, Rect) {
    (
        Rect {x: rect.x, y: rect.y, w: width, h: rect.h},
        Rect {x: rect.x + width, y: rect.y, w: rect.w - width, h: rect.h}
    )
}


/// render words clipped inside a rectangle, on top of a background with rounded corners
pub fn render_words_in_rect(ctx: &mut Context, canvas: &mut Canvas, words: &Vec<String>, rect: Rect, font: &str, font_size: f32, cross_out: &str, color: Color) {

    let mut x = rect.x;
    let mut y = rect.y;
    let prev = canvas.scissor_rect();
    canvas.draw(&graphics::Mesh::new_rectangle(
        ctx, 
            graphics::DrawMode::fill(), rect, TEXT_BG_COLOR).unwrap(), Vec2::new(0.0, 0.0)
    );
    canvas.set_scissor_rect(rect).unwrap();

    let word_width: f32 = font_size * 6.0;
    let word_height: f32 = font_size * 0.75;

    let num_columns = ((rect.w / word_width).floor() as i32).max(1);

    let texts: Vec<(Text, &String)> = words.iter().map(move |w| {
        let mut text = Text::new(TextFragment::new(w.clone()).color(color));
        text.set_font(font);
        text.set_scale(font_size);

        (text, w)
    }).collect();

    let column_width = rect.w / num_columns as f32;

    let cross_out_lower = cross_out.to_lowercase();


    for (text, word) in texts {
        let end_y = y + word_height;

        if end_y > rect.y + rect.h {
            y = rect.y;
            x += column_width;
        }

        canvas.draw(&text, Vec2::new(x, y));

        if word.to_lowercase().starts_with(&cross_out_lower) && cross_out_lower.len() > 0 {
            let positions = text.glyph_positions(ctx).unwrap();
            let dimensions = text.dimensions(ctx).unwrap();
            
            let cross_out_len = if cross_out_lower.len() == word.len() {
                dimensions.w
            } else {
                positions[cross_out_lower.len()].x
            };
            // cross out line
            canvas.draw(
                &graphics::Mesh::new_line(
                    ctx,
                    &[
                        Vec2::new(0.0, dimensions.h * 0.5),
                        Vec2::new(cross_out_len, dimensions.h * 0.5)
                    ],
                    5.0,
                    Color::GREEN
                ).unwrap(),
                Vec2::new(x, y)
            );
        }

        y += word_height;
    }

    canvas.set_scissor_rect(prev).unwrap();
}

pub fn center_text_in_rect(ctx: &mut Context, canvas: &mut Canvas, text: &Text, rect: Rect) {
    let prev = canvas.scissor_rect();
    canvas.set_scissor_rect(rect).unwrap();

    let dimensions = text.dimensions(ctx).unwrap();

    let x = (rect.w - dimensions.w) / 2.0;
    let y = (rect.h - dimensions.h) / 2.0;

    canvas.draw(text, Vec2::new(rect.x + x, rect.y + y));

    canvas.set_scissor_rect(prev).unwrap();
}
