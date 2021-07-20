
use crate::prelude::*;
use super::*;

#[derive(Default)]
pub struct Return {
    pub do_chunk_load: bool,
    pub do_quit: bool,
}

pub fn handle_input(
    data: &mut data::Data,
    world: &mut WorldData,
    state: &mut GameState,
    pgui: &mut GUI,
    rdata: &mut data::RData,
    idata: &data::IData,
) -> Return {

    let mut ret = Return::default();

    data.input.start_new_frame();
    data.display.video.text_input().start();
    data.input.update_flags(data.display.sdl.keyboard().mod_state());
    for event in data.event_pump.poll_iter() {
        use sdl2::event::Event::*;
        use sdl2::keyboard::Keycode::*;
        use sdl2::event::WindowEvent::*;
        data.input.update(&event);
        match event {
            TextInput { ref text, .. } => {
                let input_text = text;
                match state {
                    GameState::Chat { text, start_frame } if *start_frame != rdata.frame_time => {
                        let mut txt = text.text().to_owned();
                        txt.push_str(&input_text);
                        text.set_text(txt);
                    },
                    _ => {}
                }
            },
            Quit {..} => ret.do_quit = true,
            KeyDown {keycode: Some(Escape), repeat: false, ..} => {
                *state = match state {
                    GameState::Paused => {
                        data.display.set_mouse_capture(true);
                        GameState::Playing { breaking: std::option::Option::None }
                    },
                    GameState::Chat { .. } => {
                        data.display.set_mouse_capture(true);
                        GameState::Playing { breaking: std::option::Option::None }
                    },
                    _ => {
                        data.display.set_mouse_capture(false);
                        GameState::Paused
                    }
                };
            },
            KeyDown {keycode: Some(F5), repeat: false, .. } => data.settings.third_person ^= true,
            KeyDown {keycode: Some(F11), repeat: false, ..} => data.display.set_fullscreen(!data.display.state.fullscreen),
            KeyDown {keycode: Some(R), ..} => ret.do_chunk_load = true,
            KeyDown {keycode: Some(E), repeat: false, ..} => {
                match state {
                    GameState::Playing {..} => {
                        data.display.set_mouse_capture(false);
                        *state = GameState::Inventory {
                            picked_item: Option::None,
                            inventory: pgui.inventory.clone().into(),
                        }
                    },
                    GameState::Inventory { .. } => {
                        data.display.set_mouse_capture(true);
                        *state = GameState::Playing { breaking: std::option::Option::None }
                    },
                    _ => {}
                };
            },
            KeyDown {keycode: Some(Return), repeat: false, ..} => {
                match state {
                    GameState::Chat { text, .. } => {
                        let cmd: Option<Cmd> = text.text().parse().ok();
                        println!("{}\n => {:?}",text.text(),cmd);
                        if let Some(cmd) = cmd {
                            cmd.exec(world, idata);
                        }
                        data.display.set_mouse_capture(true);
                        *state = GameState::Playing { breaking: std::option::Option::None }
                    },
                    _ => {}
                };
            },
            KeyDown {keycode: Some(T), ..} => {
                match state {
                    GameState::Playing {..} => {
                        data.display.set_mouse_capture(false);
                        *state = GameState::Chat { text: idata.font.build_text("".into()), start_frame: rdata.frame_time }
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
            Window { win_event: Resized(..), .. } => data.display.refresh(),
            _ => {}
        }
    }

    pgui.scroll(-data.input.scroll());

    ret

}