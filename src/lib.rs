use core::f32;
use font::FontSheet;
use js_sys::Math::random;
use input::{Event, Input};
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use crate::wasm::*;
#[cfg(target_arch = "wasm32")]
mod wasm_utils;

pub mod state;
pub mod font;
pub mod input;

use state::{LogicStatus, State, StateHandler};

struct TestState {
    a: f32,
    b: f32,
    c: f32,
    tex: Texture,
    font: font::Font,
    inp: Input,
    s: String,
}

impl TestState {
    pub fn new(dvr: &Dvr) -> Result<TestState, String> {
        Ok(TestState {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            tex: dvr.load_texture("/pluto.png", Some(Box::new(|| { web_sys::console::log_1(&JsValue::from_str("Kexy")); })), None)?,
            font: font::Font::new(
                vec![
                    FontSheet::new(
                        ('\0', '\u{00ff}'),
                        dvr.load_texture("/font.png", None, None)?,
                        16,
                        16,
                        0.0,
                        0.0,
                        vec![
                            0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50,
                            0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50,
                            0.25, 0.40, 0.25, 0.05, 0.25, 0.05, 0.10, 0.45, 0.25, 0.25, 0.35, 0.00, 0.40, 0.15, 0.40, 0.15,
                            0.15, 0.30, 0.20, 0.20, 0.15, 0.15, 0.15, 0.10, 0.20, 0.15, 0.40, 0.40, 0.20, 0.15, 0.20, 0.20,
                            0.00, 0.05, 0.25, 0.10, 0.20, 0.15, 0.20, 0.10, 0.15, 0.40, 0.20, 0.10, 0.15, 0.10, 0.15, 0.10,
                            0.20, 0.10, 0.15, 0.15, 0.10, 0.15, 0.10, 0.05, 0.10, 0.10, 0.10, 0.20, 0.15, 0.20, 0.25, 0.15,
                            0.30, 0.20, 0.20, 0.25, 0.20, 0.20, 0.20, 0.15, 0.20, 0.40, 0.35, 0.20, 0.40, 0.10, 0.20, 0.15,
                            0.20, 0.20, 0.25, 0.25, 0.25, 0.20, 0.10, 0.05, 0.20, 0.10, 0.10, 0.30, 0.45, 0.30, 0.05, 0.50,
                            0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50,
                            0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50,
                            0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50,
                            0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50,
                            0.05, 0.05, 0.50, 0.50, 0.05, 0.05, 0.50, 0.50, 0.15, 0.15, 0.15, 0.50, 0.50, 0.50, 0.50, 0.50,
                            0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.10, 0.50, 0.50, 0.50, 0.50, 0.15, 0.15, 0.50, 0.50, 0.50,
                            0.20, 0.20, 0.50, 0.50, 0.20, 0.20, 0.50, 0.50, 0.20, 0.20, 0.20, 0.50, 0.50, 0.50, 0.50, 0.50,
                            0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.15, 0.50, 0.50, 0.50, 0.50, 0.20, 0.20, 0.50, 0.50, 0.50,
                        ]
                    )?
                ],
                0.9,
                '?'
            ),
            inp: Input::new(&dvr, None)?,
            s: String::new(),
        })
    }
}

impl State for TestState {
    fn logic(&mut self) -> Result<LogicStatus, String> {
        self.a += 0.1;
        self.a %= f32::consts::TAU;
        self.b -= 0.03;
        self.b %= f32::consts::TAU;
        if self.inp.key_down("W") {
            self.c += 1.0;
        }
        if self.inp.key_down("S") {
            self.c -= 1.0;
        }
        if random() > 0.99 {
            // return Ok(LogicStatus::NewStateWithClosure(Box::new(|prev: Box<dyn State>| -> Box<dyn State> {
            //     Box::new(TestState2::new(prev))
            // })))
            return Ok(LogicStatus::nswc(|prev: Box<dyn State>| -> Box<dyn State> {
                Box::new(TestState2::new(prev))
            }))
        }
        for e in &self.inp {
            match e {
                Event::Char(chars) => {
                    self.s += &chars;
                },
                Event::KeyDown(e) => {
                    if e.key_code == "Backspace" {
                        self.s.pop();
                    }
                },
                _ => {}
            }
        }
        Ok(LogicStatus::Continue)
    }

    fn draw(&self, dvr: &Dvr) -> Result<(), String> {
        dvr.clear(0.1, 0.0, 0.1, 1.0);
        dvr.draw(
            &self.tex,
            300.0 * f32::cos(self.b) + self.c,
            300.0 * f32::sin(self.b),
            Some((100.0, 100.0)),
            Some(((25.0, 25.0), (50.0, 50.0))),
            self.a
        )?;
        dvr.draw(
            &self.tex,
            0.0,
            0.0,
            Some((500.0, 200.0)),
            Some(((25.0, 25.0), (50.0, 50.0))),
            0.0
        )?;
        dvr.draw(
            &self.tex,
            0.0,
            0.0,
            Some((500.0, 100.0)),
            None,
            f32::consts::PI
        )?;
        dvr.draw(
            &self.tex,
            0.0,
            0.0,
            Some((100.0, 500.0)),
            None,
            f32::consts::PI
        )?;
        let _ = self.font.draw_text(
            &dvr,
            "AbcgÅä¤öe",
            0.0,
            0.0,
            Some(100.0),
            100.0,
            font::Align::Centre,
            font::VAlign::Centre
        );
        let _ = self.font.draw_text(
            &dvr,
            "Abcg åä¤öe\ndreagy\n!&(/|si",
            dvr.get_screen_width() as f32 * -0.5,
            dvr.get_screen_height() as f32 * 0.5,
            None,
            100.0,
            font::Align::Left,
            font::VAlign::Top
        );
        let _ = self.font.draw_text(
            &dvr,
            "Abcgåä¤öe\ndÅreagn\n!&(/|si",
            dvr.get_screen_width() as f32 * 0.5,
            dvr.get_screen_height() as f32 * -0.5,
            None,
            100.0,
            font::Align::Right,
            font::VAlign::Bottom
        );
        let _ = dvr.draw(
            &self.tex,
            dvr.get_screen_width() as f32 * -0.5,
            dvr.get_screen_height() as f32 * 0.5,
            Some((100.0, 100.0)),
            Some(((25.0, 25.0), (50.0, 50.0))),
            0.0
        );
        let _ = dvr.draw(
            &self.tex,
            dvr.get_screen_width() as f32 * -0.5,
            0.0,
            Some((100.0, 100.0)),
            Some(((25.0, 25.0), (50.0, 50.0))),
            0.0
        );
        let _ = self.font.draw_text(
            &dvr,
            &self.s,
            0.0,
            0.0,
            None,
            100.0,
            font::Align::Centre,
            font::VAlign::Centre
        );
        if let Some((x, y)) = self.inp.get_mouse_pos() {
            let (x, y) = dvr.native_mouse_coords_to_dvr((x, y));
            dvr.draw(
                &self.tex,
                x,
                y,
                Some((100.0, 100.0)),
                Some(((25.0, 25.0), (50.0, 50.0))),
                self.a
            )?;
        }
        Ok(())
    }
}

struct TestState2 {
    ts: Option<Box<dyn State>>
}

impl TestState2 {
    pub fn new(ts: Box<dyn State>) -> TestState2 {
        TestState2 { ts: Some(ts) }
    }
}

impl State for TestState2 {
    fn logic(&mut self) -> Result<LogicStatus, String> {
        if random() > 0.95 {
            return Ok(LogicStatus::NewState(self.ts.take().unwrap()))
        }
        Ok(LogicStatus::Continue)
    }

    fn draw(&self, dvr: &Dvr) -> Result<(), String> {
        self.ts.as_ref().unwrap().draw(&dvr)?;
        Ok(())
    }
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("Unable to get window")?;
    let document = window.document().ok_or("Unable to get document")?;

    let canvas: web_sys::HtmlCanvasElement = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let dvr = Dvr::new(context)?;
    let test_state = TestState::new(&dvr)?;
    StateHandler::run(dvr, Box::new(test_state))?;

    Ok(())
}