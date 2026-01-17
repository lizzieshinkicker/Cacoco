use super::{RenderContext, get_alignment_anchor_offset};
use crate::constants::{DEFAULT_GLYPH_H, DEFAULT_GLYPH_W};
use crate::models::sbardef::*;
use eframe::egui;

/// Renders a numeric player statistic.
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
        NumberType::Ammo => ctx.state.inventory.get_ammo(def.param),
        NumberType::AmmoSelected => {
            let idx = ctx
                .state
                .inventory
                .get_selected_ammo_type(ctx.state.selected_weapon_slot);
            ctx.state.inventory.get_ammo(idx)
        }
        NumberType::MaxAmmo => ctx.state.inventory.get_max_ammo(def.param),
        NumberType::AmmoWeapon => ctx
            .state
            .inventory
            .get_weapon_ammo_type(def.param)
            .map_or(0, |idx| ctx.state.inventory.get_ammo(idx)),
        NumberType::MaxAmmoWeapon => ctx
            .state
            .inventory
            .get_weapon_ammo_type(def.param)
            .map_or(0, |idx| ctx.state.inventory.get_max_ammo(idx)),
        NumberType::Kills => ctx.state.player.kills,
        NumberType::Items => ctx.state.player.items,
        NumberType::Secrets => ctx.state.player.secrets,
        NumberType::MaxKills => ctx.state.player.max_kills,
        NumberType::MaxItems => ctx.state.player.max_items,
        NumberType::MaxSecrets => ctx.state.player.max_secrets,
        NumberType::KillsPercent => ctx
            .state
            .get_stat_percent(ctx.state.player.kills, ctx.state.player.max_kills),
        NumberType::ItemsPercent => ctx
            .state
            .get_stat_percent(ctx.state.player.items, ctx.state.player.max_items),
        NumberType::SecretsPercent => ctx
            .state
            .get_stat_percent(ctx.state.player.secrets, ctx.state.player.max_secrets),
        NumberType::PowerupDuration => ctx
            .state
            .player
            .powerup_durations
            .get(&def.param)
            .cloned()
            .unwrap_or(0.0)
            .ceil() as i32,
    };

    if def.maxlength > 0 {
        let clean_len = def.maxlength.clamp(0, 9) as u32;
        let max_val = 10_i32.saturating_pow(clean_len) - 1;
        let min_val = -10_i32.saturating_pow(clean_len.saturating_sub(1)) + 1;

        if val > max_val {
            val = max_val;
        }
        if val < min_val {
            val = min_val;
        }
    }

    let text = if is_percent {
        format!("{}%", val)
    } else {
        format!("{}", val)
    };
    draw_text_line(
        ctx,
        &text,
        &def.font,
        pos,
        def.common.alignment,
        true,
        alpha,
    );
}

/// Renders a dynamic alphanumeric string.
pub(super) fn draw_string(ctx: &RenderContext, def: &StringDef, pos: egui::Pos2, alpha: f32) {
    let text = match def.type_ {
        0 => def
            .data
            .as_deref()
            .unwrap_or("Having Fun with Cacoco!")
            .to_string(),
        1 => "Entryway".to_string(),
        2 => "MAP01".to_string(),
        3 => "Sandy Petersen".to_string(),
        _ => String::new(),
    };
    draw_text_line(
        ctx,
        &text,
        &def.font,
        pos,
        def.common.alignment,
        false,
        alpha,
    );
}

/// High-performance text rendering logic.
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

    let (base_sc_x, _) = ctx.get_native_scale_factor();
    let scale_adjustment = 1.0 / base_sc_x;

    let scaled_size = layout.size * scale_adjustment;
    let self_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, scaled_size);
    let off = get_alignment_anchor_offset(align, self_rect);

    let mut cur_x = pos.x + off.x;
    let start_y = pos.y + off.y;

    let tint = egui::Color32::from_white_alpha((255.0 * alpha) as u8);
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

    for glyph in layout.glyphs {
        if let Some(tex) = glyph.texture {
            let y_adj = glyph.y_offset * scale_adjustment;
            let char_pos = if ctx.is_native {
                egui::pos2(cur_x, start_y + y_adj)
            } else {
                egui::pos2(cur_x.floor(), (start_y + y_adj).floor())
            };

            let s_pos = ctx.to_screen(char_pos);
            let (sc_x, sc_y) = ctx.get_render_scale();

            let s_size = egui::vec2(
                glyph.tex_w * sc_x * scale_adjustment,
                glyph.h * sc_y * scale_adjustment,
            );
            ctx.painter
                .image(tex.id(), egui::Rect::from_min_size(s_pos, s_size), uv, tint);
        }
        cur_x += glyph.advance * scale_adjustment;
    }
}

/// Calculates the virtual width of a rendered string.
pub fn measure_text_line(ctx: &RenderContext, txt: &str, font: &str, is_num: bool) -> f32 {
    layout_text_line(ctx, txt, font, is_num).map_or(0.0, |l| l.size.x)
}

/// Calculates the full virtual size (width and height) of a rendered string.
pub fn measure_text_size(ctx: &RenderContext, txt: &str, font: &str, is_num: bool) -> egui::Vec2 {
    layout_text_line(ctx, txt, font, is_num).map_or(egui::Vec2::ZERO, |l| l.size)
}

fn layout_text_line<'a>(
    ctx: &'a RenderContext,
    text: &str,
    font: &str,
    is_num: bool,
) -> Option<TextLayout<'a>> {
    let (stem, font_type) = if is_num {
        ctx.file
            .data
            .number_fonts
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(font))
            .map(|f| (f.stem.clone(), f.type_))
    } else {
        ctx.file
            .data
            .hud_fonts
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(font))
            .map(|f| (f.stem.clone(), f.type_))
    }?;

    let mut glyphs = Vec::new();
    let mut total_w = 0.0;
    let mut max_h = 0.0;
    let stem_upper = stem.to_uppercase();

    let mut mono_width = 0.0;
    if font_type == 0 {
        let zero_id = ctx.assets.resolve_patch_id(&stem, '0', is_num);
        mono_width = ctx
            .assets
            .textures
            .get(&zero_id)
            .map(|t| t.size_vec2().x)
            .unwrap_or(DEFAULT_GLYPH_W);
    } else if font_type == 1 {
        let chars = if is_num { "0123456789" } else { "ABCDEFGHJM" };
        for c in chars.chars() {
            let id = ctx.assets.resolve_patch_id(&stem, c, is_num);
            if let Some(tex) = ctx.assets.textures.get(&id) {
                mono_width = mono_width.max(tex.size_vec2().x);
            }
        }
        if mono_width == 0.0 {
            mono_width = DEFAULT_GLYPH_W;
        }
    }

    for c in text.chars() {
        if c == ' ' {
            let w = if font_type == 2 { 4.0 } else { mono_width };
            glyphs.push(Glyph {
                texture: None,
                tex_w: 0.0,
                advance: w,
                h: 0.0,
                y_offset: 0.0,
            });
            total_w += w;
            continue;
        }

        let id = ctx.assets.resolve_patch_id(&stem, c, is_num);

        if let Some(tex) = ctx.assets.textures.get(&id) {
            let sz = tex.size_vec2();
            let mut y_offset = 0.0;

            let mut advance = if font_type == 2 { sz.x } else { mono_width };

            if stem_upper == "STT" && c == '1' && font_type == 2 {
                advance += 2.0;
            }

            if stem_upper == "STCFN" {
                match c {
                    '.' | ',' => y_offset = 4.0,
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
            if (sz.y + y_offset) > max_h {
                max_h = sz.y + y_offset;
            }
        } else {
            let w = if font_type == 2 {
                DEFAULT_GLYPH_W
            } else {
                mono_width
            };
            glyphs.push(Glyph {
                texture: None,
                tex_w: 0.0,
                advance: w,
                h: 0.0,
                y_offset: 0.0,
            });
            total_w += w;
        }
    }

    if max_h == 0.0 {
        max_h = DEFAULT_GLYPH_H;
    }
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
