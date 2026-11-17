//! Vector-painted monochrome icons.
//!
//! egui's bundled fonts do not cover the symbol codepoints used by a media
//! player (▶ ⛶ ≡ ★ …), so every icon is drawn with the painter instead of a
//! font glyph. This keeps the interface crisp at any scale and truly
//! monochrome, independent of installed fonts.

use egui::{Color32, CornerRadius, Painter, Rect, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Play,
    Pause,
    Stop,
    Prev,
    Next,
    StepBack,
    StepFwd,
    Volume,
    Muted,
    Playlist,
    Info,
    Fullscreen,
    ExitFullscreen,
    StarFilled,
    StarOutline,
    Close,
    Up,
    Down,
}

/// Paint `icon` centered in `rect` using `color`.
pub fn paint(painter: &Painter, icon: Icon, rect: Rect, color: Color32) {
    let c = rect.center();
    let s = rect.height().min(rect.width()) * 0.5;
    let stroke = Stroke::new((s * 0.28).max(1.3), color);

    let triangle_left = |scale: f32| {
        let p = vec![
            egui::pos2(c.x + s * 0.6 * scale, c.y - s * 0.85 * scale),
            egui::pos2(c.x + s * 0.6 * scale, c.y + s * 0.85 * scale),
            egui::pos2(c.x - s * 0.8 * scale, c.y),
        ];
        painter.add(egui::Shape::convex_polygon(p, color, Stroke::NONE));
    };
    let triangle_right = |scale: f32| {
        let p = vec![
            egui::pos2(c.x - s * 0.6 * scale, c.y - s * 0.85 * scale),
            egui::pos2(c.x - s * 0.6 * scale, c.y + s * 0.85 * scale),
            egui::pos2(c.x + s * 0.8 * scale, c.y),
        ];
        painter.add(egui::Shape::convex_polygon(p, color, Stroke::NONE));
    };
    let vbar = |x: f32, h: f32| {
        painter.rect_filled(
            Rect::from_center_size(egui::pos2(x, c.y), Vec2::new((s * 0.3).max(1.6), s * h)),
            CornerRadius::same(1),
            color,
        );
    };

    match icon {
        Icon::Play => triangle_right(1.0),
        Icon::Pause => {
            let w = (s * 0.38).max(1.6);
            let h = s * 1.5;
            painter.rect_filled(
                Rect::from_center_size(egui::pos2(c.x - s * 0.45, c.y), Vec2::new(w, h)),
                CornerRadius::same(1),
                color,
            );
            painter.rect_filled(
                Rect::from_center_size(egui::pos2(c.x + s * 0.45, c.y), Vec2::new(w, h)),
                CornerRadius::same(1),
                color,
            );
        }
        Icon::Stop => {
            painter.rect_filled(
                Rect::from_center_size(c, Vec2::splat(s * 1.25)),
                CornerRadius::same(2),
                color,
            );
        }
        Icon::Prev => {
            vbar(c.x - s * 0.75, 1.6);
            triangle_left(0.85);
        }
        Icon::Next => {
            triangle_right(0.85);
            vbar(c.x + s * 0.75, 1.6);
        }
        Icon::StepBack => {
            triangle_left(0.7);
            vbar(c.x + s * 0.7, 1.4);
        }
        Icon::StepFwd => {
            vbar(c.x - s * 0.7, 1.4);
            triangle_right(0.7);
        }
        Icon::Volume | Icon::Muted => {
            let p = vec![
                egui::pos2(c.x - s * 0.9, c.y - s * 0.35),
                egui::pos2(c.x - s * 0.35, c.y - s * 0.35),
                egui::pos2(c.x + s * 0.15, c.y - s * 0.8),
                egui::pos2(c.x + s * 0.15, c.y + s * 0.8),
                egui::pos2(c.x - s * 0.35, c.y + s * 0.35),
                egui::pos2(c.x - s * 0.9, c.y + s * 0.35),
            ];
            painter.add(egui::Shape::convex_polygon(p, color, Stroke::NONE));
            if icon == Icon::Volume {
                for r in [0.55, 0.95] {
                    painter.add(egui::Shape::line(
                        vec![
                            egui::pos2(c.x + s * 0.35, c.y - s * r * 0.7),
                            egui::pos2(c.x + s * 0.55, c.y - s * r * 0.5),
                            egui::pos2(c.x + s * 0.55, c.y + s * r * 0.5),
                            egui::pos2(c.x + s * 0.35, c.y + s * r * 0.7),
                        ],
                        stroke,
                    ));
                }
            } else {
                let d = s * 0.45;
                painter.line_segment(
                    [
                        egui::pos2(c.x + s * 0.35, c.y - d),
                        egui::pos2(c.x + s * 0.95, c.y + d),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        egui::pos2(c.x + s * 0.95, c.y - d),
                        egui::pos2(c.x + s * 0.35, c.y + d),
                    ],
                    stroke,
                );
            }
        }
        Icon::Playlist => {
            for dy in [-0.7, 0.0, 0.7] {
                painter.line_segment(
                    [
                        egui::pos2(c.x - s * 0.8, c.y + s * dy),
                        egui::pos2(c.x + s * 0.8, c.y + s * dy),
                    ],
                    stroke,
                );
            }
        }
        Icon::Info => {
            painter.circle_stroke(c, s * 0.95, stroke);
            painter.circle_filled(egui::pos2(c.x, c.y - s * 0.45), (s * 0.18).max(1.0), color);
            painter.line_segment(
                [
                    egui::pos2(c.x, c.y + s * 0.05),
                    egui::pos2(c.x, c.y + s * 0.6),
                ],
                stroke,
            );
        }
        Icon::Fullscreen => {
            let (x0, y0, x1, y1) = (c.x - s * 0.9, c.y - s * 0.9, c.x + s * 0.9, c.y + s * 0.9);
            let l = s * 0.45;
            for (cx, cy, dx, dy) in [
                (x0, y0, 1.0, 1.0),
                (x1, y0, -1.0, 1.0),
                (x0, y1, 1.0, -1.0),
                (x1, y1, -1.0, -1.0),
            ] {
                painter.line_segment([egui::pos2(cx, cy), egui::pos2(cx + l * dx, cy)], stroke);
                painter.line_segment([egui::pos2(cx, cy), egui::pos2(cx, cy + l * dy)], stroke);
            }
        }
        Icon::ExitFullscreen => {
            let (x0, y0, x1, y1) = (
                c.x - s * 0.95,
                c.y - s * 0.95,
                c.x + s * 0.95,
                c.y + s * 0.95,
            );
            for (tx, ty, sx, sy) in [(x0, y0, x1, y1), (x1, y1, x0, y0)] {
                painter.line_segment([egui::pos2(tx, ty), egui::pos2(sx, sy)], stroke);
                let (ux, uy) = (sx - tx, sy - ty);
                let len = (ux * ux + uy * uy).sqrt();
                let (nx, ny) = (ux / len, uy / len);
                let a = s * 0.4;
                painter.add(egui::Shape::line(
                    vec![
                        egui::pos2(tx + nx * a, ty),
                        egui::pos2(tx, ty),
                        egui::pos2(tx, ty + ny * a),
                    ],
                    stroke,
                ));
            }
        }
        Icon::StarFilled | Icon::StarOutline => {
            let outer = s * 1.0;
            let inner = outer * 0.42;
            let mut pts = Vec::with_capacity(10);
            for i in 0..10 {
                let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                let r = if i % 2 == 0 { outer } else { inner };
                pts.push(egui::pos2(c.x + angle.cos() * r, c.y + angle.sin() * r));
            }
            if icon == Icon::StarFilled {
                painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
            } else {
                painter.add(egui::Shape::line(pts, stroke));
            }
        }
        Icon::Close => {
            let d = s * 0.7;
            painter.line_segment(
                [egui::pos2(c.x - d, c.y - d), egui::pos2(c.x + d, c.y + d)],
                stroke,
            );
            painter.line_segment(
                [egui::pos2(c.x + d, c.y - d), egui::pos2(c.x - d, c.y + d)],
                stroke,
            );
        }
        Icon::Up | Icon::Down => {
            let dir: f32 = if icon == Icon::Up { -1.0 } else { 1.0 };
            painter.line_segment(
                [
                    egui::pos2(c.x, c.y - s * 0.85 * dir),
                    egui::pos2(c.x, c.y + s * 0.85 * dir),
                ],
                stroke,
            );
            painter.add(egui::Shape::line(
                vec![
                    egui::pos2(c.x - s * 0.55, c.y + s * 0.3 * dir),
                    egui::pos2(c.x, c.y + s * 0.85 * dir),
                    egui::pos2(c.x + s * 0.55, c.y + s * 0.3 * dir),
                ],
                stroke,
            ));
        }
    }
}

/// A compact click-and-hover button that paints `icon` as vector shapes.
pub fn icon_button(ui: &mut Ui, icon: Icon, tooltip: &str) -> bool {
    let size = Vec2::new(30.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let visuals = ui.style().visuals.clone();
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(3), visuals.widgets.hovered.bg_fill);
    }
    let color = if response.hovered() {
        visuals.widgets.hovered.fg_stroke.color
    } else {
        visuals.widgets.inactive.fg_stroke.color
    };
    paint(ui.painter(), icon, rect.shrink2(Vec2::new(5.0, 4.0)), color);
    response.on_hover_text(tooltip).clicked()
}

/// Small square icon button for dense rows (playlist entries, toasts).
pub fn icon_button_small(ui: &mut Ui, icon: Icon, tooltip: &str) -> bool {
    let size = Vec2::splat(18.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let visuals = ui.style().visuals.clone();
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(3), visuals.widgets.hovered.bg_fill);
    }
    let color = if response.hovered() {
        visuals.widgets.hovered.fg_stroke.color
    } else {
        visuals.widgets.inactive.fg_stroke.color
    };
    paint(ui.painter(), icon, rect.shrink(2.5), color);
    response.on_hover_text(tooltip).clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_covers_every_icon() {
        let ctx = egui::Context::default();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::unique("test"),
        ));
        let rect = Rect::from_min_size(egui::Pos2::ZERO, Vec2::splat(20.0));
        for icon in [
            Icon::Play,
            Icon::Pause,
            Icon::Stop,
            Icon::Prev,
            Icon::Next,
            Icon::StepBack,
            Icon::StepFwd,
            Icon::Volume,
            Icon::Muted,
            Icon::Playlist,
            Icon::Info,
            Icon::Fullscreen,
            Icon::ExitFullscreen,
            Icon::StarFilled,
            Icon::StarOutline,
            Icon::Close,
            Icon::Up,
            Icon::Down,
        ] {
            paint(&painter, icon, rect, Color32::WHITE);
        }
    }
}
