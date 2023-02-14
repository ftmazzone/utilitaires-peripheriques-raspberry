use ecran::ecran::ecran::Wepd7In5BV2;
use image::{DynamicImage, Rgb, Rgba};
use rusttype::{point, Font, Scale};
use std::fs;

fn main() {
    // Load the font
    let font_data = &fs::read("./STIXTwoMath-Regular.ttf").unwrap();
    // This only succeeds if collection consists of one font
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

    // The font size to use
    let scale = Scale::uniform(64.0);

    // The text to render
    let text = &format!(
        "Png ! {} ⚠ ↗",
        '\u{237c}'.to_string()
    );

    // Use a dark red colour
    let colour = (150, 0, 0);

    let v_metrics = font.v_metrics(scale);

    // layout the glyphs in a line with 20 pixels padding
    let glyphs: Vec<_> = font
        .layout(text, scale, point(20.0, 20.0 + v_metrics.ascent))
        .collect();

    // work out the layout size
    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };

    let couleur_pixel_565 = convertir_rgb_888_en_reg_565(colour);

    // Create a new rgba image with some padding
    let mut image =
        DynamicImage::new_rgb16(Wepd7In5BV2::largeur() as u32, Wepd7In5BV2::hauteur() as u32)
            .to_rgb16();

    // Loop through the glyphs in the text, positing each one on a line
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            // Draw the glyph into the image per-pixel by using the draw closure
            glyph.draw(|x, y, v| {
                let pixel;
                if v < 0.5 {
                    pixel = [0, 0, 0];
                } else {
                    pixel = [couleur_pixel_565, 0, 0]
                }

                image.put_pixel(
                    // Offset the position by the glyph bounding box
                    x + bounding_box.min.x as u32,
                    y + bounding_box.min.y as u32,
                    // Turn the coverage into an alpha value
                    Rgb(pixel),
                )
            });
        }
    }

    // Save the image to a png file
    image.save("image_example.png").unwrap();
    println!("Generated: image_example.png");
}

fn convertir_rgb_888_en_reg_565(couleur: (u8, u8, u8)) -> u16 {
    let rgb_565 = (((couleur.0 & 0b11111000) as u16) << 8) 
        + ((couleur.1 & 0b11111100) << 3)  as u16
        + (couleur.2 >> 3) as u16;
    rgb_565
}



