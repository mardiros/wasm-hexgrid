#[macro_use]
extern crate log;

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use stdweb::traits::*;
use stdweb::unstable::TryInto;

use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d, EventListenerHandle, FillRule};

use stdweb::web::event::{ClickEvent, ConcreteEvent};

pub const ANGLE: f64 = PI / 6.;

pub const BOARD_SIDE: usize = 7;
pub const BOARD_SIZE: usize = (BOARD_SIDE * 2) - 1;
pub const TILE_SIZE: f64 = 22.;

pub fn tile_x() -> f64 {
    TILE_SIZE * ANGLE.cos()
}

pub fn tile_y() -> f64 {
    TILE_SIZE * ANGLE.sin()
}

pub fn half_board_width() -> f64 {
    tile_x() * BOARD_SIZE as f64
}

pub fn margin() -> f64 {
    tile_x()
}

pub fn is_odd(x: i32) -> bool {
    x & 1 == 1
}

pub fn is_even(x: i32) -> bool {
    x & 1 == 0
}

pub struct HexTile {
    q: i32,
    r: i32,
}

impl HexTile {
    pub fn new(q: i32, r: i32) -> Self {
        HexTile { q, r }
    }

    fn coord_to_pos(&self) -> (f64, f64) {
        let q = self.q as f64 * tile_x();
        let r = self.r as f64 * tile_x();
        let x = 2. * q + r;
        let y = 2. * r * ANGLE.cos();
        (x, y)
    }

    pub fn paint(&self, context: &CanvasRenderingContext2d) {
        info!("Paint");
        context.set_stroke_style_color("#a24"); //red
        let delta: f64 = PI / 3.;
        let mut angle: f64 = PI / 6.;
        let (x, y) = self.coord_to_pos();
        context.begin_path();
        context.move_to(
            margin() + half_board_width() + x + tile_x(),
            margin() + half_board_width() + y + TILE_SIZE * angle.sin(),
        );
        for _ in 0..6 {
            angle += delta;
            context.line_to(
                margin() + half_board_width() + x + TILE_SIZE * angle.cos(),
                margin() + half_board_width() + y + TILE_SIZE * angle.sin(),
            );
        }
        if is_even(self.q) && is_even(self.r) {
            context.set_fill_style_color("#111");
        } else if is_odd(self.q) && is_even(self.r) {
            context.set_fill_style_color("#222");
        } else if is_even(self.q) && is_odd(self.r) {
            context.set_fill_style_color("#222");
        } else {
            context.set_fill_style_color("#333");
        }
        context.fill(FillRule::NonZero);
        context.stroke();

        let text = format!("{}   {}", self.q, self.r);
        context.set_fill_style_color("#eee");
        context.fill_text(
            text.as_str(),
            margin() + half_board_width() + x - 12.,
            margin() + half_board_width() + y,
            None,
        );
    }
}

pub struct Store {
    game_over: bool,
    tiles: Vec<HexTile>,
}

impl Store {
    fn new() -> Self {
        let maxq: i32 = ((BOARD_SIZE as f32 / 2.).ceil()) as i32;
        let minq: i32 = maxq - BOARD_SIZE as i32;

        let mut tiles = Vec::with_capacity((maxq * maxq) as usize);
        for x in minq..maxq {
            for y in minq..maxq {
                if (x + y < maxq) && (x + y > -maxq) {
                    tiles.push(HexTile::new(x, y));
                }
            }
        }

        Store {
            game_over: false,
            tiles: tiles,
        }
    }

    fn paint(&self, context: &CanvasRenderingContext2d) {
        info!("Paint");
        for tile in self.tiles.iter() {
            tile.paint(context);
        }
    }

    fn play(&mut self, x: f64, y: f64) -> Result<(), ()> {
        info!("Click on {}, {}", x, y);
        let x = x - margin() - half_board_width();
        let y = y - margin() - half_board_width();
        info!("Translated to {}, {}", x, y);

        let r = y / (2. * ANGLE.cos());
        let q = (x - r) / 2.;

        let r = (r / tile_x()).round() as i32;
        let q = (q / tile_x()).round() as i32;
        info!("Translated to {}, {}", q, r);

        Ok(())
    }
}

struct Canvas {
    canvas: CanvasElement,
}

impl Canvas {
    fn new(selector: &str, store: &Store) -> Canvas {
        let canvas: CanvasElement = document()
            .query_selector(selector)
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();

        let canvas_width = half_board_width() as u32 * 2 + 2 * margin() as u32;

        canvas.set_width(canvas_width);
        canvas.set_height(canvas_width);

        Canvas { canvas }
    }

    fn context(&self) -> CanvasRenderingContext2d {
        self.canvas.get_context().unwrap()
    }

    fn add_event_listener<T, F>(&self, listener: F) -> EventListenerHandle
    where
        T: ConcreteEvent,
        F: FnMut(T) + 'static,
    {
        self.canvas.add_event_listener(listener)
    }
}

struct AnimatedCanvas {
    canvas: Canvas,
    store: Rc<RefCell<Store>>,
}

impl AnimatedCanvas {
    fn new(store: Store, canvas: Canvas) -> AnimatedCanvas {
        let store_rc = Rc::new(RefCell::new(store));
        AnimatedCanvas {
            canvas,
            store: store_rc,
        }
    }
    fn attach_event(&mut self) {
        let context = self.canvas.context();
        let store = self.store.clone();
        self.canvas.add_event_listener(move |event: ClickEvent| {
            let mut store = store.borrow_mut();
            let x = event.offset_x();
            let y = event.offset_y();
            if let Ok(_) = store.play(x, y) {
                store.paint(&context);
            }
        });
    }

    fn paint(&mut self) {
        let context = self.canvas.context();
        let store = self.store.clone();
        store.borrow_mut().paint(&context);
    }
}

fn main() {
    web_logger::init();
    info!("Welcome aboard");

    let store = Store::new();
    let canvas = Canvas::new("#game", &store);
    let mut ac = AnimatedCanvas::new(store, canvas);
    ac.attach_event();
    ac.paint();
}
