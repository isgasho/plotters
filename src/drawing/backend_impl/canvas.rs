use js_sys::JSON;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use crate::style::{Color, FontDesc};

/// The backend that is drawing on the HTML canvas
/// TODO: Support double bufferring
pub struct CanvasBackend {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

pub struct CanvasError(JsValue);

impl std::fmt::Display for CanvasError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return write!(
            fmt,
            "Canvas Error: {}",
            JSON::stringify(&self.0)
                .map(|s| Into::<String>::into(&s))
                .unwrap_or_else(|_| "Unknown".to_string())
        );
    }
}

impl std::fmt::Debug for CanvasError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return write!(
            fmt,
            "CanvasError({})",
            JSON::stringify(&self.0)
                .map(|s| Into::<String>::into(&s))
                .unwrap_or_else(|_| "Unknown".to_string())
        );
    }
}

impl std::error::Error for CanvasError {}

impl CanvasBackend {
    /// Create a new drawing backend backed with an HTML5 canvas object
    /// - `elem_id` The element id for the canvas
    /// - Return either some drawing backend has been created, or none in error case
    pub fn new(elem_id: &str) -> Option<Self> {
        let document = window()?.document()?;
        let canvas = document.get_element_by_id(elem_id)?;
        let canvas: HtmlCanvasElement = canvas.dyn_into().ok()?;
        let context: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
        Some(CanvasBackend { canvas, context })
    }
}

fn make_canvas_color<C: Color>(color: &C) -> JsValue {
    let (r, g, b) = color.rgb();
    let a = color.alpha();
    format!("rgba({},{},{},{})", r, g, b, a).into()
}

impl DrawingBackend for CanvasBackend {
    type ErrorType = CanvasError;

    fn get_size(&self) -> (u32, u32) {
        (self.canvas.width(), self.canvas.height())
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<CanvasError>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<CanvasError>> {
        Ok(())
    }

    fn draw_pixel<S: BackendStyle>(
        &mut self,
        point: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<CanvasError>> {
        self.context
            .set_fill_style(&make_canvas_color(style.as_color()));
        self.context
            .fill_rect(f64::from(point.0), f64::from(point.1), 1.0, 1.0);
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.context
            .set_stroke_style(&make_canvas_color(style.as_color()));
        self.context.begin_path();
        self.context.move_to(f64::from(from.0), f64::from(from.1));
        self.context.line_to(f64::from(to.0), f64::from(to.1));
        self.context.stroke();
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if fill {
            self.context
                .set_fill_style(&make_canvas_color(style.as_color()));
            self.context.fill_rect(
                f64::from(upper_left.0),
                f64::from(upper_left.1),
                f64::from(bottom_right.0 - upper_left.0),
                f64::from(bottom_right.1 - upper_left.1),
            );
        } else {
            self.context
                .set_stroke_style(&make_canvas_color(style.as_color()));
            self.context.stroke_rect(
                f64::from(upper_left.0),
                f64::from(upper_left.1),
                f64::from(bottom_right.0 - upper_left.0),
                f64::from(bottom_right.1 - upper_left.1),
            );
        }
        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut path = path.into_iter();
        self.context.begin_path();
        if let Some(start) = path.next() {
            self.context
                .set_stroke_style(&make_canvas_color(style.as_color()));
            self.context.move_to(f64::from(start.0), f64::from(start.1));
            for next in path {
                self.context.line_to(f64::from(next.0), f64::from(next.1));
            }
        }
        self.context.stroke();
        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if fill {
            self.context
                .set_fill_style(&make_canvas_color(style.as_color()));
        } else {
            self.context
                .set_stroke_style(&make_canvas_color(style.as_color()));
        }
        self.context.begin_path();
        self.context
            .arc(
                f64::from(center.0),
                f64::from(center.1),
                f64::from(radius),
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .map_err(|e| DrawingErrorKind::DrawingError(CanvasError(e)))?;
        if fill {
            self.context.fill();
        } else {
            self.context.stroke();
        }
        Ok(())
    }

    fn draw_text<'b, C: Color>(
        &mut self,
        text: &str,
        font: &FontDesc<'b>,
        pos: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.context.set_text_baseline("bottom");
        self.context.set_fill_style(&make_canvas_color(color));
        self.context
            .set_font(&format!("{}px {}", font.get_size(), font.get_name()));
        self.context
            .fill_text(text, f64::from(pos.0), f64::from(pos.1) + font.get_size())
            .map_err(|e| DrawingErrorKind::DrawingError(CanvasError(e)))?;
        Ok(())
    }
}
