// SPDX-License-Identifier: GPL-3.0-or-later

//! Mapping between the image on screen and the complex plane, and the window
//! changes behind pan, zoom and zoom-box.
//!
//! Positions are normalised image coordinates: (0, 0) is the top-left of
//! the image, (1, 1) the bottom-right.

use fractalforge_core::Window;

/// The complex point at normalised image position `(u, v)`.
pub fn point_at(w: &Window, u: f64, v: f64) -> (f64, f64) {
    (
        w.x_min + u * (w.x_max - w.x_min),
        w.y_max - v * (w.y_max - w.y_min),
    )
}

/// Zooms by `factor` (> 1 zooms in) keeping the point under `(u, v)` fixed.
pub fn zoom_about(w: &Window, u: f64, v: f64, factor: f64) -> Window {
    let (px, py) = point_at(w, u, v);
    let width = (w.x_max - w.x_min) / factor;
    let height = (w.y_max - w.y_min) / factor;
    let x_min = px - u * width;
    let y_max = py + v * height;
    Window::new(x_min, x_min + width, y_max - height, y_max)
}

/// Moves the view so content shifts by `(du, dv)` image widths/heights
/// (dragging right shows what was to the left).
pub fn pan(w: &Window, du: f64, dv: f64) -> Window {
    let dx = du * (w.x_max - w.x_min);
    let dy = dv * (w.y_max - w.y_min);
    Window::new(w.x_min - dx, w.x_max - dx, w.y_min + dy, w.y_max + dy)
}

/// The window covering the normalised rectangle `(u0, v0)`–`(u1, v1)`.
pub fn sub_window(w: &Window, u0: f64, v0: f64, u1: f64, v1: f64) -> Window {
    let (x0, y0) = point_at(w, u0.min(u1), v0.min(v1));
    let (x1, y1) = point_at(w, u0.max(u1), v0.max(v1));
    Window::new(x0, x1, y1, y0)
}

/// Widens or narrows the window about its centre so its width/height ratio
/// equals `aspect`, keeping the height.
pub fn fit_aspect(w: &Window, aspect: f64) -> Window {
    let cx = (w.x_min + w.x_max) / 2.0;
    let half_w = (w.y_max - w.y_min) * aspect / 2.0;
    Window::new(cx - half_w, cx + half_w, w.y_min, w.y_max)
}

/// Where an image rendered for `old` appears, in normalised coordinates of
/// the view `new`: returns `(u0, v0, u1, v1)` of its corners.
pub fn placement(old: &Window, new: &Window) -> (f64, f64, f64, f64) {
    let u = |x: f64| (x - new.x_min) / (new.x_max - new.x_min);
    let v = |y: f64| (new.y_max - y) / (new.y_max - new.y_min);
    (u(old.x_min), v(old.y_max), u(old.x_max), v(old.y_min))
}

/// Magnification relative to a view two units high.
pub fn magnification(w: &Window) -> f64 {
    2.0 / (w.y_max - w.y_min)
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Window = Window::new(-2.0, 2.0, -1.0, 1.0);

    fn close(a: &Window, b: &Window) -> bool {
        let e = 1e-12;
        (a.x_min - b.x_min).abs() < e
            && (a.x_max - b.x_max).abs() < e
            && (a.y_min - b.y_min).abs() < e
            && (a.y_max - b.y_max).abs() < e
    }

    #[test]
    fn zoom_keeps_point_fixed() {
        let z = zoom_about(&W, 0.25, 0.75, 4.0);
        let (a, b) = point_at(&W, 0.25, 0.75);
        let (c, d) = point_at(&z, 0.25, 0.75);
        assert!((a - c).abs() < 1e-12 && (b - d).abs() < 1e-12);
        assert!(((z.x_max - z.x_min) - 1.0).abs() < 1e-12);
        assert!(close(&zoom_about(&z, 0.25, 0.75, 0.25), &W));
    }

    #[test]
    fn pan_and_box() {
        let p = pan(&W, 0.5, 0.0);
        assert!(close(&p, &Window::new(-4.0, 0.0, -1.0, 1.0)));
        let p = pan(&W, 0.0, 0.5); // drag down shows what was above
        assert!(close(&p, &Window::new(-2.0, 2.0, 0.0, 2.0)));
        let b = sub_window(&W, 0.75, 1.0, 0.25, 0.5);
        assert!(close(&b, &Window::new(-1.0, 1.0, -1.0, 0.0)));
    }

    #[test]
    fn aspect_and_placement() {
        let f = fit_aspect(&W, 1.0);
        assert!(close(&f, &Window::new(-1.0, 1.0, -1.0, 1.0)));
        assert_eq!(placement(&W, &W), (0.0, 0.0, 1.0, 1.0));
        let zoomed = zoom_about(&W, 0.5, 0.5, 2.0);
        let (u0, v0, u1, v1) = placement(&W, &zoomed);
        assert!((u0 + 0.5).abs() < 1e-12 && (v0 + 0.5).abs() < 1e-12);
        assert!((u1 - 1.5).abs() < 1e-12 && (v1 - 1.5).abs() < 1e-12);
        assert_eq!(magnification(&W), 1.0);
    }
}
