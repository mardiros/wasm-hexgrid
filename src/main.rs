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
pub const TILE_SIZE: f64 = 32.;
pub const BOARD_WIDTH: f64 = TILE_SIZE * (BOARD_SIZE as f64);

pub fn tile_x() -> f64 {
    TILE_SIZE * ANGLE.cos()
}

pub fn tile_y() -> f64 {
    TILE_SIZE * ANGLE.sin()
}

pub fn board_width() -> f64 {
    tile_x() * BOARD_SIZE as f64
}

pub fn margin() -> f64 {
    tile_x()
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
        let delta: f64 = PI / 3.;
        let mut angle: f64 = PI / 6.;
        let (x, y) = self.coord_to_pos();
        context.begin_path();
        context.move_to(
            margin() + board_width() + x + tile_x(),
            margin() + board_width() + y + TILE_SIZE * angle.sin(),
        );
        for _ in 0..6 {
            angle += delta;
            context.line_to(
                margin() + board_width() + x + TILE_SIZE * angle.cos(),
                margin() + board_width() + y + TILE_SIZE * angle.sin(),
            );
        }
        context.set_fill_style_color("#333");
        context.fill(FillRule::NonZero);
        context.stroke();

        let text = format!("{}   {}", self.q, self.r);
        context.set_fill_style_color("#eee");
        context.fill_text(
            text.as_str(),
            margin() + board_width() + x - 12.,
            margin() + board_width() + y,
            None,
        );
    }
}

pub struct Store {
    game_over: bool,
    cell_width: u32,
}

impl Store {
    fn new(cell_width: u32) -> Self {
        Store {
            cell_width,
            game_over: false,
        }
    }

    fn cell_width(&self) -> u32 {
        self.cell_width
    }

    fn paint(&self, context: &CanvasRenderingContext2d) {
        info!("Paint");

        let maxq: i32 = ((BOARD_SIZE as f32 / 2.).ceil()) as i32;
        let minq: i32 = maxq - BOARD_SIZE as i32;
        context.set_stroke_style_color("#a24"); //red
        for x in minq..maxq {
            for y in minq..maxq {
                if (x + y < maxq) && (x + y > -maxq) {
                    let tile = HexTile::new(x, y);
                    tile.paint(context);
                }
            }
        }
    }

    fn play(&mut self, x: usize, y: usize) -> Result<(), ()> {
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

        let canvas_width = board_width() as u32 * 2 + 2 * margin() as u32;

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
            let x = (event.offset_x() / store.cell_width() as f64) as usize;
            let y = (event.offset_y() / store.cell_width() as f64) as usize;
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

    let store = Store::new(60);
    let canvas = Canvas::new("#game", &store);
    let mut ac = AnimatedCanvas::new(store, canvas);
    ac.attach_event();
    ac.paint();
}
