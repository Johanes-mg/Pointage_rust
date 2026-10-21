use egui::{Color32, FontId, Rounding, Stroke, TextureFilter, TextureOptions, TextureWrapMode};
use std::collections::HashMap;
use std::sync::Mutex;

pub const PRIMAIRE: Color32 = Color32::from_rgb(21, 43, 84);
pub const PRIMAIRE_CLAIR: Color32 = Color32::from_rgb(38, 66, 120);
pub const ACCENT: Color32 = Color32::from_rgb(0, 122, 204);

pub const FOND: Color32 = Color32::from_rgb(244, 246, 249);
pub const FOND_CARTE: Color32 = Color32::from_rgb(255, 255, 255);
pub const FOND_BILAN: Color32 = Color32::from_rgb(235, 242, 250);
pub const BORDURE: Color32 = Color32::from_rgb(222, 226, 232);

pub const TEXTE: Color32 = Color32::from_rgb(30, 35, 45);
pub const TEXTE_SECONDAIRE: Color32 = Color32::from_rgb(110, 118, 130);
pub const SUCCES: Color32 = Color32::from_rgb(34, 148, 84);
pub const DANGER: Color32 = Color32::from_rgb(200, 55, 55);
pub const WARNING: Color32 = Color32::from_rgb(220, 145, 20);

pub const SELECTION_JOUR: Color32 = Color32::from_rgb(220, 235, 252);
pub const FOND_POINTAGE: Color32 = Color32::from_rgb(230, 248, 236);

pub const JOUR_BIEN_BG: Color32 = Color32::from_rgb(200, 240, 210);
pub const JOUR_MOYEN_BG: Color32 = Color32::from_rgb(255, 240, 200);
pub const JOUR_MAUVAIS_BG: Color32 = Color32::from_rgb(255, 220, 220);

pub const FERIE_DEMI_BG: Color32 = Color32::from_rgb(255, 236, 214);

pub const SELECTION_TEXTE: Color32 = Color32::from_rgb(210, 214, 220);

pub fn r_sm() -> Rounding {
    Rounding::same(6.0)
}
pub fn r_md() -> Rounding {
    Rounding::same(10.0)
}
pub fn r_lg() -> Rounding {
    Rounding::same(14.0)
}

pub fn configurer_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = FOND;
    visuals.window_fill = FOND_CARTE;
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.faint_bg_color = FOND;

    visuals.widgets.noninteractive.bg_fill = FOND_CARTE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDURE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXTE);
    visuals.widgets.noninteractive.rounding = r_md();

    visuals.widgets.inactive.bg_fill = Color32::WHITE;
    visuals.widgets.inactive.weak_bg_fill = FOND;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDURE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXTE);
    visuals.widgets.inactive.rounding = r_md();

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(245, 248, 252);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(240, 244, 248);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXTE);
    visuals.widgets.hovered.rounding = r_md();

    visuals.widgets.active.bg_fill = Color32::WHITE;
    visuals.widgets.active.weak_bg_fill = Color32::WHITE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXTE);
    visuals.widgets.active.rounding = r_md();

    visuals.selection.bg_fill = SELECTION_TEXTE;
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);

    visuals.window_rounding = r_lg();
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
        color: Color32::from_black_alpha(30),
    };
    visuals.popup_shadow = visuals.window_shadow;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 10.0);
    style.spacing.interact_size = egui::vec2(38.0, 34.0);
    style.spacing.window_margin = egui::Margin::same(18.0);
    style.text_styles = [
        (egui::TextStyle::Heading, FontId::proportional(22.0)),
        (egui::TextStyle::Body, FontId::proportional(13.5)),
        (egui::TextStyle::Button, FontId::proportional(13.5)),
        (egui::TextStyle::Small, FontId::proportional(11.5)),
        (egui::TextStyle::Monospace, FontId::monospace(12.5)),
    ]
    .into();
    ctx.set_style(style);
}

static IMAGES: Mutex<Option<HashMap<String, egui::TextureHandle>>> = Mutex::new(None);

fn chemins_possibles(nom: &str) -> Vec<String> {
    let mut chemins = vec![format!("Images/{}", nom)];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            chemins.push(
                dir.join("Images")
                    .join(nom)
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }

    chemins.push(format!("{}/Images/{}", env!("CARGO_MANIFEST_DIR"), nom));
    chemins
}

pub fn image(ctx: &egui::Context, nom: &str) -> Option<egui::TextureHandle> {
    {
        let cache = IMAGES.lock().unwrap();
        if let Some(map) = cache.as_ref() {
            if let Some(tex) = map.get(nom) {
                return Some(tex.clone());
            }
        }
    }

    for chemin in chemins_possibles(nom) {
        if let Ok(bytes) = std::fs::read(&chemin) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let taille = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(taille, &pixels);
                let options = TextureOptions {
                    magnification: TextureFilter::Linear,
                    minification: TextureFilter::Linear,
                    wrap_mode: TextureWrapMode::ClampToEdge,
                };
                let texture = ctx.load_texture(nom, color_image, options);

                let mut cache = IMAGES.lock().unwrap();
                let map = cache.get_or_insert_with(HashMap::new);
                map.insert(nom.to_string(), texture.clone());
                return Some(texture);
            }
        }
    }

    None
}

pub fn image_ui(
    ui: &mut egui::Ui,
    nom: &str,
    largeur: f32,
    hauteur: f32,
) -> egui::Response {
    let ctx = ui.ctx().clone();
    if let Some(tex) = image(&ctx, nom) {
        let natif = tex.size_vec2();
        let ratio = (largeur / natif.x).min(hauteur / natif.y);
        let taille = if ratio >= 1.0 {
            egui::vec2(largeur, hauteur)
        } else {
            egui::vec2(natif.x * ratio, natif.y * ratio)
        };
        ui.add(egui::Image::new(&tex).fit_to_exact_size(taille))
    } else {
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(largeur, hauteur), egui::Sense::hover());
        ui.painter().rect_filled(rect, 3.0, BORDURE);
        resp
    }
}

pub fn carte<R>(ui: &mut egui::Ui, contenu: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::none()
        .fill(FOND_CARTE)
        .rounding(r_md())
        .stroke(Stroke::new(1.0_f32, BORDURE))
        .inner_margin(egui::Margin::symmetric(20.0, 16.0))
        .show(ui, contenu)
        .inner
}

pub fn bouton_secondaire(ui: &mut egui::Ui, texte: &str, largeur: f32) -> bool {
    let b = egui::Button::new(egui::RichText::new(texte).color(TEXTE).size(13.0))
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0_f32, BORDURE))
        .rounding(r_md())
        .min_size(egui::vec2(largeur, 40.0));
    ui.add(b).clicked()
}

pub fn bouton_image_texte(
    ui: &mut egui::Ui,
    fichier_image: &str,
    texte: &str,
    largeur: f32,
    hauteur: f32,
) -> bool {
    let ctx = ui.ctx().clone();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(largeur, hauteur), egui::Sense::click());

    let fond = if resp.hovered() {
        Color32::from_rgb(245, 248, 252)
    } else {
        Color32::WHITE
    };
    let bordure = if resp.hovered() { ACCENT } else { BORDURE };

    let painter = ui.painter();
    painter.rect_filled(rect, r_md(), fond);
    painter.rect_stroke(rect, r_md(), Stroke::new(1.0_f32, bordure));

    let mut x = rect.left() + 14.0;
    if let Some(tex) = image(&ctx, fichier_image) {
        let img_size = egui::vec2(20.0, 20.0);
        let img_rect = egui::Rect::from_min_size(
            egui::pos2(x, rect.center().y - img_size.y / 2.0),
            img_size,
        );
        painter.image(
            tex.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        x += img_size.x + 10.0;
    }

    painter.text(
        egui::pos2(x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        texte,
        FontId::proportional(13.0),
        TEXTE,
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

#[allow(dead_code)]
pub fn bouton_image_centree(
    ui: &mut egui::Ui,
    fichier_image: &str,
    largeur: f32,
    hauteur: f32,
    largeur_image: f32,
    hauteur_image: f32,
) -> bool {
    let ctx = ui.ctx().clone();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(largeur, hauteur), egui::Sense::click());

    let fond = if resp.hovered() {
        Color32::from_rgb(245, 248, 252)
    } else {
        Color32::WHITE
    };
    let bordure = if resp.hovered() { ACCENT } else { BORDURE };

    let painter = ui.painter();
    painter.rect_filled(rect, r_md(), fond);
    painter.rect_stroke(rect, r_md(), Stroke::new(1.0_f32, bordure));

    if let Some(tex) = image(&ctx, fichier_image) {
        let img_rect = egui::Rect::from_center_size(
            rect.center(),
            egui::vec2(largeur_image, hauteur_image),
        );
        painter.image(
            tex.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn bouton_image_seule(
    ui: &mut egui::Ui,
    fichier_image: &str,
    taille: f32,
    taille_icone: f32,
) -> bool {
    let ctx = ui.ctx().clone();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(taille, taille), egui::Sense::click());

    let fond = if resp.hovered() {
        Color32::from_rgb(245, 248, 252)
    } else {
        Color32::WHITE
    };
    let bordure = if resp.hovered() { ACCENT } else { BORDURE };

    let painter = ui.painter();
    painter.rect_filled(rect, r_md(), fond);
    painter.rect_stroke(rect, r_md(), Stroke::new(1.0_f32, bordure));

    if let Some(tex) = image(&ctx, fichier_image) {
        let img_rect =
            egui::Rect::from_center_size(rect.center(), egui::vec2(taille_icone, taille_icone));
        painter.image(
            tex.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn badge(ui: &mut egui::Ui, texte: &str, fond: Color32, couleur: Color32) {
    egui::Frame::none()
        .fill(fond)
        .rounding(r_sm())
        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(texte)
                    .color(couleur)
                    .size(11.5)
                    .strong(),
            );
        });
}

pub fn separateur(ui: &mut egui::Ui) {
    let largeur = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(largeur, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, BORDURE);
}

pub fn message_inline(ui: &mut egui::Ui, texte: &str, succes: bool) {
    let (fond, couleur) = if succes {
        (Color32::from_rgb(209, 250, 229), SUCCES)
    } else {
        (Color32::from_rgb(254, 226, 226), DANGER)
    };
    egui::Frame::none()
        .fill(fond)
        .rounding(r_sm())
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(texte).color(couleur).size(12.5));
        });
}

pub fn charger_icone_fenetre(nom: &str) -> Option<egui::IconData> {
    for chemin in chemins_possibles(nom) {
        if let Ok(bytes) = std::fs::read(&chemin) {
            if let Some(icon) = decoder_ico(&bytes) {
                return Some(icon);
            }
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let taille = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                return Some(egui::IconData {
                    rgba: pixels,
                    width: taille[0] as u32,
                    height: taille[1] as u32,
                });
            }
        }
    }
    None
}

fn decoder_ico(bytes: &[u8]) -> Option<egui::IconData> {
    if bytes.len() < 6 {
        return None;
    }
    if bytes[0..2] != [0x00, 0x00] || bytes[2..4] != [0x01, 0x00] {
        return None;
    }
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    if count == 0 {
        return None;
    }
    let entries_offset = 6;
    let entries_size = 16;
    if bytes.len() < entries_offset + entries_size * count {
        return None;
    }

    let tailles_preferees: [u32; 4] = [32, 48, 64, 256];
    let mut candidats: Vec<(u32, usize, usize)> = Vec::new();

    for i in 0..count {
        let base = entries_offset + i * entries_size;
        let w = if bytes[base] == 0 { 256 } else { bytes[base] as u32 };
        let h = if bytes[base + 1] == 0 { 256 } else { bytes[base + 1] as u32 };
        if w != h {
            continue;
        }
        let taille_data = u32::from_le_bytes([
            bytes[base + 8],
            bytes[base + 9],
            bytes[base + 10],
            bytes[base + 11],
        ]) as usize;
        let offset_data = u32::from_le_bytes([
            bytes[base + 12],
            bytes[base + 13],
            bytes[base + 14],
            bytes[base + 15],
        ]) as usize;
        let fin = match offset_data.checked_add(taille_data) {
            Some(f) => f,
            None => continue,
        };
        if fin > bytes.len() {
            continue;
        }
        candidats.push((w, offset_data, taille_data));
    }

    if candidats.is_empty() {
        return None;
    }

    let choix = tailles_preferees
        .iter()
        .find_map(|&t| candidats.iter().find(|(w, _, _)| *w == t))
        .or_else(|| candidats.iter().max_by_key(|(w, _, _)| *w))?;

    let (w, offset, len) = *choix;
    let data = bytes.get(offset..offset + len)?;

    if data.len() >= 8 && data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        let img = image::load_from_memory(data).ok()?;
        let rgba = img.to_rgba8();
        let (iw, ih) = rgba.dimensions();
        return Some(egui::IconData {
            rgba: rgba.into_raw(),
            width: iw,
            height: ih,
        });
    }

    decoder_bmp_32(data, w)
}

fn decoder_bmp_32(data: &[u8], w: u32) -> Option<egui::IconData> {
    if data.len() < 40 {
        return None;
    }
    let header_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if header_size < 40 || header_size > data.len() {
        return None;
    }
    let bpp = u16::from_le_bytes([data[14], data[15]]);
    if bpp != 32 {
        return None;
    }

    let h = w;
    let pixels_offset = header_size;
    let pixels_size = (w as usize) * (h as usize) * 4;
    let masque_offset = pixels_offset + pixels_size;
    let masque_size = (w as usize) * (h as usize) / 8;

    if data.len() < masque_offset + masque_size {
        return None;
    }

    let pixels = &data[pixels_offset..pixels_offset + pixels_size];
    let masque = &data[masque_offset..masque_offset + masque_size];

    let mut rgba = Vec::with_capacity(pixels_size);

    for row in (0..h).rev() {
        for col in 0..w {
            let idx = ((row * w + col) * 4) as usize;
            let b = pixels[idx];
            let g = pixels[idx + 1];
            let r = pixels[idx + 2];
            let a = pixels[idx + 3];

            let bit_idx = ((row * w + col) / 8) as usize;
            let bit_pos = 7 - ((row * w + col) % 8);
            let bit_masque = (masque[bit_idx] >> bit_pos) & 1;

            let alpha_final = if a == 0 && bit_masque == 0 {
                if r != 0 || g != 0 || b != 0 {
                    255
                } else {
                    0
                }
            } else {
                a
            };

            rgba.extend_from_slice(&[r, g, b, alpha_final]);
        }
    }

    Some(egui::IconData {
        rgba,
        width: w,
        height: h,
    })
}
