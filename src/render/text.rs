use crate::constants::{DEFAULT_GLYPH_W, DEFAULT_GLYPH_H};
use crate::model::*;
use eframe::egui;
use super::{get_alignment_anchor_offset, RenderContext};

pub(super) fn draw_number(
    ctx: &RenderContext,
    def: &NumberDef,
    pos: egui::Pos2,
    is_percent: bool,
    alpha: f32,
) {
    let mut val = match def.type_ {
        NumberType::Health => ctx.state.player.health,
        NumberType::Armor => ctx.state.player.armor,
        NumberType::Frags => 0,
        NumberType::Ammo => ctx.state.get_ammo(def.param),
        NumberType::AmmoSelected => {
            let idx = ctx.state.get_selected_ammo_type();
            ctx.state.get_ammo(idx)
        }
        NumberType::MaxAmmo => ctx.state.get_max_ammo(def.param),
        NumberType::AmmoWeapon => ctx
            .state
            .get_weapon_ammo_type(def.param)
            .map_or(0, |idx| ctx.state.get_ammo(idx)),
        NumberType::MaxAmmoWeapon => ctx
            .state
            .get_weapon_ammo_type(def.param)
            .map_or(0, |idx| ctx.state.get_max_ammo(idx)),
    };

    if def.maxlength > 0 {
        let max_val = 10_i32.pow(def.maxlength as u32) - 1;
        let min_val = -10_i32.pow((def.maxlength - 1) as u32) + 1;
        if val > max_val { val = max_val; }
        if val < min_val { val = min_val; }
    }

    let text = if is_percent {
        format!("{}%", val)
    } else {
        format!("{}", val)
    };
    draw_text_line(ctx, &text, &def.font, pos, def.common.alignment, true, alpha);
}

pub fn draw_text_line(
    ctx: &RenderContext,
    txt: &str,
    font: &str,
    pos: egui::Pos2,
    align: Alignment,
    is_num: bool,
    alpha: f32,
) {
    let layout = match layout_text_line(ctx, txt, font, is_num) {
        Some(l) => l,
        None => {
            let color = egui::Color32::RED.linear_multiply(alpha);
            ctx.painter.text(
                ctx.to_screen(pos),
                egui::Align2::LEFT_TOP,
                format!("NO FONT: {}", font),
                egui::FontId::proportional(10.0 * ctx.proj.final_scale_y),
                color,
            );
            return;
        }
    };

    let off = get_alignment_anchor_offset(align, layout.size.x, layout.size.y);
    let mut cur_x = pos.x + off.x;
    let start_y = pos.y + off.y;

    let tint = egui::Color32::from_white_alpha((255.0 * alpha) as u8);
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

    for glyph in layout.glyphs {
        if let Some(tex) = glyph.texture {
            let char_pos = egui::pos2(cur_x, start_y + glyph.y_offset);

            let s_pos = ctx.to_screen(char_pos);
            let s_size = egui::vec2(
                glyph.tex_w * ctx.proj.final_scale_x,
                glyph.h * ctx.proj.final_scale_y,
            );
            ctx.painter.image(tex.id(), egui::Rect::from_min_size(s_pos, s_size), uv, tint);
        }
        cur_x += glyph.advance;
    }
}

pub fn measure_text_line(ctx: &RenderContext, txt: &str, font: &str, is_num: bool) -> f32 {
    layout_text_line(ctx, txt, font, is_num).map_or(0.0, |l| l.size.x)
}

fn layout_text_line<'a>(
    ctx: &'a RenderContext,
    text: &str,
    font: &str,
    is_num: bool,
) -> Option<TextLayout<'a>> {
    let stem = if is_num {
        ctx.get_number_font(font).map(|f| f.stem.clone())
    } else {
        ctx.file
            .data
            .hud_fonts
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(font))
            .map(|f| f.stem.clone())
    }?;

    let mut glyphs = Vec::new();
    let mut total_w = 0.0;
    let mut max_h = 0.0;
    let stem_upper = stem.to_uppercase();

    for c in text.chars() {
        if c == ' ' {
            glyphs.push(Glyph {
                texture: None,
                tex_w: 0.0,
                advance: 4.0,
                h: 0.0,
                y_offset: 0.0,
            });
            total_w += 4.0;
            continue;
        }

        let name = ctx.assets.resolve_patch_name(&stem, c, is_num);
        if let Some(tex) = ctx.assets.textures.get(&name) {
            let sz = tex.size_vec2();

            let mut y_offset = 0.0;
            let mut advance = sz.x;

            // Hardcode spacing offset for STT from the IWAD.
            if stem_upper == "STT" && c == '1' {
                advance += 2.0;
            }

            // Hardcode vertical offsets specifically for the IWAD HUD font characters.
            if stem_upper == "STCFN" {
                match c {
                    '.' | ','  => y_offset = 4.0,
                    '-' => y_offset = 2.0,
                    _ => {}
                }
            }

            glyphs.push(Glyph {
                texture: Some(tex),
                tex_w: sz.x,
                advance,
                h: sz.y,
                y_offset,
            });

            total_w += advance;
            if (sz.y + y_offset) > max_h { max_h = sz.y + y_offset; }
        } else {
            glyphs.push(Glyph {
                texture: None,
                tex_w: 0.0,
                advance: DEFAULT_GLYPH_W,
                h: 0.0,
                y_offset: 0.0,
            });
            total_w += DEFAULT_GLYPH_W;
        }
    }

    if max_h == 0.0 { max_h = DEFAULT_GLYPH_H; }
    Some(TextLayout {
        glyphs,
        size: egui::vec2(total_w, max_h),
    })
}

struct Glyph<'a> {
    texture: Option<&'a egui::TextureHandle>,
    tex_w: f32,
    advance: f32,
    h: f32,
    y_offset: f32,
}

struct TextLayout<'a> {
    glyphs: Vec<Glyph<'a>>,
    size: egui::Vec2,
}