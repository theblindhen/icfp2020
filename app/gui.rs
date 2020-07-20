use crate::value_tree::ValueTree;
use crate::draw::Point;
use crate::interpreter::{self, Env};
use crate::aplang::Var;
use std::cell::RefMut;
use std::ops::DerefMut;

use log::*;

struct Game {
    prg_var: Var,
    env: Env,
    state: ValueTree,
    screen_offset: Point,
}

fn step(g: &mut Game, click: Point) -> crate::draw::Overlay {
    println!("Sent point: ({}, {})", click.0, click.1);
    info!("State\n{}\n{}", g.state, crate::encodings::modulate(&g.state));
    let (new_state, screens) = interpreter::interact(g.prg_var, &mut g.env.clone(), &g.state, click);
    let overlay = crate::draw::Overlay::new(screens);
    g.state = new_state;
    g.screen_offset = Point(overlay.xstart, overlay.ystart);
    println!();
    println!("Waiting for input");
    overlay
}

#[cfg(feature = "gui")]
pub fn gui(prg_var: Var, env: Env, state: ValueTree, scale: i32) -> Result<(), Box<dyn std::error::Error>> {
    use fltk::{app, button::*, draw::*, frame::*, window::*};
    use std::cell::RefCell;
    use std::rc::Rc;

    let (dim_x, dim_y) = (300 * scale, 300 * scale);

    let state_rc = Rc::from(RefCell::from(Game { prg_var, env, state, screen_offset: Point(0,0) }));

    let app = app::App::default().with_scheme(app::AppScheme::Gtk);

    let mut wind = DoubleWindow::new(100, 100, dim_x, dim_y, "Viz");
    let mut frame = Frame::new(0, 0, dim_x, dim_y, "");
    frame.set_color(Color::White);
    frame.set_frame(FrameType::DownBox);

    wind.end();
    wind.show();

    let offs = Offscreen::new(dim_x, dim_y).unwrap();
    offs.begin();
    set_draw_color(Color::Black);
    draw_rectf(0, 0, dim_x, dim_y);
    offs.end();

    let mut frame_c = frame.clone();
    let offs = Rc::from(RefCell::from(offs));
    let offs_rc = offs.clone();

    frame.draw(Box::new(move || {
        if offs_rc.borrow().is_valid() {
            offs_rc.borrow().copy(0, 0, dim_x, dim_y, 0, 0);
        }
    }));

    let mut x = 0;
    let mut y = 0;

    frame_c.handle(Box::new(move |ev| {
        match ev {
            Event::Push => {
                let coords = app::event_coords();
                let mut overlay = {
                    let mut state = state_rc.borrow_mut();
                    let mut point = Point(coords.0 as i64 / scale as i64, coords.1 as i64 / scale as i64);
                    point.0 += state.screen_offset.0;
                    point.1 += state.screen_offset.1;
                    step(state.deref_mut(), point)
                };
                let mut image = fltk::image::RgbImage::new(&overlay.as_rgba(), overlay.width(), overlay.height(), 4).unwrap();
                let (w, h) = (overlay.width() as i32, overlay.height() as i32);
                image.scale(w*scale, h*scale, true, true);
                offs.borrow().begin();
                set_draw_color(Color::Black);
                draw_rectf(0, 0, dim_x, dim_y);
                image.draw(0, 0, w*scale, h*scale);
                offs.borrow().end();
                frame.redraw();
                false
            }
            _ => false,
        }
    }));

    app.run()?;
    Ok(())
}

#[cfg(not(feature = "gui"))]
pub fn gui(prg_var: Var, env: Env, state: ValueTree, scale: i32) -> Result<(), Box<dyn std::error::Error>> {
    panic!("GUI feature not enabled. You've built with `--no-default-features`.")
}
