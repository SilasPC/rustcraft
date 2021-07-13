
use crate::prelude::*;
use super::*;

#[derive(Default)]
pub struct Return {
    pub do_chunk_load: bool,
    pub do_quit: bool,
}

pub fn handle_input(
    data: &mut Data,
    state: &mut GameState,
    event_pump: &mut sdl2::EventPump,
    display: &mut GLDisplay,
    pgui: &mut GUI,
    rdata: &mut RenderData
) -> Return {

    let mut ret = Return::default();

    data.input.start_new_frame();
    display.video.text_input().start();
    //data.input.update_scancodes(event_pump.keyboard_state());
    for event in event_pump.poll_iter() {
        use sdl2::event::Event::*;
        use sdl2::keyboard::Keycode::*;
        use sdl2::event::WindowEvent::*;
        data.input.update(&event);
        match event {
            TextInput { ref text, .. } => {
                let input_text = text;
                match state {
                    GameState::Chat { text, start_frame } if *start_frame != data.frame_time => {
                        let mut txt = text.text().to_owned();
                        txt.push_str(&input_text);
                        text.set_text(txt);
                    },
                    _ => {}
                }
            },
            Quit {..} => ret.do_quit = true,
            KeyDown {keycode: Some(Escape), ..} => {
                *state = match state {
                    GameState::Paused => {
                        display.set_mouse_capture(true);
                        GameState::Playing
                    },
                    GameState::Chat { .. } => {
                        display.set_mouse_capture(true);
                        GameState::Playing
                    },
                    _ => {
                        display.set_mouse_capture(false);
                        GameState::Paused
                    }
                };
            },
            KeyDown {keycode: Some(F11), ..} => display.toggle_fullscren(),
            KeyDown {keycode: Some(R), ..} => ret.do_chunk_load = true,
            KeyDown {keycode: Some(E), ..} => {
                match state {
                    GameState::Playing => {
                        display.set_mouse_capture(false);
                        *state = GameState::Inventory {
                            picked_item: Option::None,
                            inventory: pgui.inventory.clone().into(),
                        }
                    },
                    GameState::Inventory { .. } => {
                        display.set_mouse_capture(true);
                        *state = GameState::Playing
                    },
                    _ => {}
                };
            },
            KeyDown {keycode: Some(Return), ..} => {
                match state {
                    GameState::Chat { text, .. } => {
                        let cmd: Option<Cmd> = text.text().parse().ok();
                        println!("{}\n => {:?}",text.text(),cmd);
                        if let Some(cmd) = cmd {
                            cmd.exec(data);
                        }
                        display.set_mouse_capture(true);
                        *state = GameState::Playing
                    },
                    _ => {}
                };
            },
            KeyDown {keycode: Some(T), ..} => {
                match state {
                    GameState::Playing => {
                        display.set_mouse_capture(false);
                        *state = GameState::Chat { text: rdata.font.build_text("".into()), start_frame: data.frame_time }
                    },
                    _ => {}
                };
            },
            KeyDown {keycode: Some(Backspace), ..} => {
                match state {
                    GameState::Chat { text, start_frame } => {
                        let mut text = text.clone();
                        let mut txt = text.text().to_owned();
                        txt.pop();
                        text.set_text(txt);
                        *state = GameState::Chat { text, start_frame: *start_frame }
                    },
                    _ => {}
                };
            },
            Window { win_event: Resized(..), .. } => display.refresh(),
            _ => {}
        }
    }

    pgui.scroll(-data.input.scroll());

    ret

}