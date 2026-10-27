use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SvgPlot {
    pub svg_xml: String,
    pub width: f64,
    pub height: f64,
}

pub struct MatplotlibPlotter;

impl MatplotlibPlotter {
    /// Renders a vector line chart in Mercury design styling
    pub fn line_chart(title: &str, x_label: &str, y_label: &str, x_vals: &[f64], y_vals: &[f64]) -> SvgPlot {
        let width = 500.0;
        let height = 300.0;
        let margin_left = 60.0;
        let margin_bottom = 40.0;
        let margin_top = 40.0;
        let margin_right = 30.0;

        let plot_w = width - margin_left - margin_right;
        let plot_h = height - margin_top - margin_bottom;

        let min_x = x_vals.iter().copied().reduce(f64::min).unwrap_or(0.0);
        let max_x = x_vals.iter().copied().reduce(f64::max).unwrap_or(1.0);
        let min_y = y_vals.iter().copied().reduce(f64::min).unwrap_or(0.0);
        let max_y = y_vals.iter().copied().reduce(f64::max).unwrap_or(1.0);

        let range_x = if max_x > min_x { max_x - min_x } else { 1.0 };
        let range_y = if max_y > min_y { max_y - min_y } else { 1.0 };

        let mut svg = format!(
            r##"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" style="background:#ffffff; font-family:Inter, sans-serif;">"##,
            width, height, width, height
        );

        // Title
        svg.push_str(&format!(
            r##"<text x="{}" y="24" font-size="14" font-weight="600" fill="#171721" text-anchor="middle">{}</text>"##,
            width * 0.5, title
        ));

        // Background gridlines
        for i in 0..=4 {
            let gy = margin_top + plot_h * (i as f64 / 4.0);
            svg.push_str(&format!(
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#f1f3f5" stroke-width="1"/>"##,
                margin_left, gy, margin_left + plot_w, gy
            ));
        }

        // Axes
        svg.push_str(&format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#ced4da" stroke-width="1.5"/>"##,
            margin_left, margin_top + plot_h, margin_left + plot_w, margin_top + plot_h
        ));
        svg.push_str(&format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#ced4da" stroke-width="1.5"/>"##,
            margin_left, margin_top, margin_left, margin_top + plot_h
        ));

        // Data points to polyline path
        if !x_vals.is_empty() && x_vals.len() == y_vals.len() {
            let mut path_d = String::new();
            for (i, (&x, &y)) in x_vals.iter().zip(y_vals.iter()).enumerate() {
                let px = margin_left + ((x - min_x) / range_x) * plot_w;
                let py = margin_top + plot_h - ((y - min_y) / range_y) * plot_h;

                if i == 0 {
                    path_d.push_str(&format!("M {:.1} {:.1}", px, py));
                } else {
                    path_d.push_str(&format!(" L {:.1} {:.1}", px, py));
                }
            }

            // Stroke line
            svg.push_str(&format!(
                r##"<path d="{}" fill="none" stroke="#5266eb" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>"##,
                path_d
            ));

            // Circle dots
            for (&x, &y) in x_vals.iter().zip(y_vals.iter()) {
                let px = margin_left + ((x - min_x) / range_x) * plot_w;
                let py = margin_top + plot_h - ((y - min_y) / range_y) * plot_h;
                svg.push_str(&format!(
                    r##"<circle cx="{:.1}" cy="{:.1}" r="4" fill="#ffffff" stroke="#5266eb" stroke-width="2"/>"##,
                    px, py
                ));
            }
        }

        // Axis labels
        svg.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="11" fill="#868e96" text-anchor="middle">{}</text>"##,
            margin_left + plot_w * 0.5, height - 10.0, x_label
        ));
        svg.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="11" fill="#868e96" text-anchor="middle" transform="rotate(-90 {} {})">{}</text>"##,
            20.0, margin_top + plot_h * 0.5, 20.0, margin_top + plot_h * 0.5, y_label
        ));

        svg.push_str("</svg>");

        SvgPlot {
            svg_xml: svg,
            width,
            height,
        }
    }

    /// Renders a bar chart in Mercury Emerald styling
    pub fn bar_chart(title: &str, categories: &[String], values: &[f64]) -> SvgPlot {
        let width = 500.0;
        let height = 300.0;
        let margin_left = 60.0;
        let margin_bottom = 50.0;
        let margin_top = 40.0;
        let margin_right = 30.0;

        let plot_w = width - margin_left - margin_right;
        let plot_h = height - margin_top - margin_bottom;

        let max_val = values.iter().copied().reduce(f64::max).unwrap_or(1.0).max(1.0);

        let mut svg = format!(
            r##"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg" style="background:#ffffff; font-family:Inter, sans-serif;">"##,
            width, height, width, height
        );

        svg.push_str(&format!(
            r##"<text x="{}" y="24" font-size="14" font-weight="600" fill="#171721" text-anchor="middle">{}</text>"##,
            width * 0.5, title
        ));

        if !categories.is_empty() {
            let bar_slot_w = plot_w / categories.len() as f64;
            let bar_w = bar_slot_w * 0.65;

            for (i, (cat, &val)) in categories.iter().zip(values.iter()).enumerate() {
                let bar_h = (val / max_val) * plot_h;
                let bx = margin_left + i as f64 * bar_slot_w + (bar_slot_w - bar_w) * 0.5;
                let by = margin_top + plot_h - bar_h;

                svg.push_str(&format!(
                    r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="4" fill="#0CA678"/>"##,
                    bx, by, bar_w, bar_h
                ));

                // Value above bar
                svg.push_str(&format!(
                    r##"<text x="{:.1}" y="{:.1}" font-size="10" font-weight="500" fill="#495057" text-anchor="middle">{:.0}</text>"##,
                    bx + bar_w * 0.5, by - 6.0, val
                ));

                // Category label below
                svg.push_str(&format!(
                    r##"<text x="{:.1}" y="{:.1}" font-size="11" fill="#495057" text-anchor="middle">{}</text>"##,
                    bx + bar_w * 0.5, margin_top + plot_h + 20.0, cat
                ));
            }
        }

        // Bottom baseline
        svg.push_str(&format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#ced4da" stroke-width="1.5"/>"##,
            margin_left, margin_top + plot_h, margin_left + plot_w, margin_top + plot_h
        ));

        svg.push_str("</svg>");

        SvgPlot {
            svg_xml: svg,
            width,
            height,
        }
    }
}
