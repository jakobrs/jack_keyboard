use std::{
    any::Any,
    sync::mpsc::{self, Receiver, Sender},
};

use druid_shell::{Application, Code, KbKey, KeyEvent, WinHandler, WindowBuilder, WindowHandle};
use jack::{Client, ClientOptions, ClosureProcessHandler, ProcessScope, RawMidi};

fn main() {
    let (tx, rx) = mpsc::channel();

    let _async_client = handle_jack(rx);
    run_gui(tx);
}

fn handle_jack(rx: Receiver<KeyboardMsg>) -> impl Any {
    let (client, _client_status) =
        Client::new("jack_keyboard", ClientOptions::NO_START_SERVER).unwrap();

    let mut out = client
        .register_port("out", jack::MidiOut::default())
        .unwrap();

    let process = move |_client: &Client, process_scope: &ProcessScope| -> jack::Control {
        let mut writer = out.writer(process_scope);

        while let Ok(msg) = rx.try_recv() {
            let KeyboardMsg { note, pressed } = msg;

            match writer.write(&RawMidi {
                time: 0,
                bytes: &[
                    if pressed { 0x91 } else { 0x81 }, // Command
                    note.to_midi_value(),              // Note
                    0x7f,                              // Velocity
                ],
            }) {
                Ok(_) => (),
                Err(err) => eprintln!("{:?}", err),
            }
        }

        jack::Control::Continue
    };

    client
        .activate_async((), ClosureProcessHandler::new(process))
        .unwrap()
}

struct AppState {
    handle: WindowHandle,
    tx: Sender<KeyboardMsg>,
}

impl WinHandler for AppState {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    fn prepare_paint(&mut self) {
        self.handle.invalidate();
    }

    fn paint(&mut self, _piet: &mut druid_shell::piet::Piet, _invalid: &druid_shell::Region) {
        // println!("Would paint");
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn key_down(&mut self, event: druid_shell::KeyEvent) -> bool {
        let KeyEvent {
            key, code, repeat, ..
        } = event;

        if repeat {
            return false;
        }

        if key == KbKey::Escape {
            self.handle.close();
        }

        if let Some(note) = Note::from_code(code) {
            self.tx
                .send(KeyboardMsg {
                    note,
                    pressed: true,
                })
                .unwrap();
        }

        true
    }

    fn key_up(&mut self, event: KeyEvent) {
        let KeyEvent { code, .. } = event;

        if let Some(note) = Note::from_code(code) {
            self.tx
                .send(KeyboardMsg {
                    note,
                    pressed: false,
                })
                .unwrap();
        }
    }

    fn request_close(&mut self) {
        self.handle.close();
    }
}

fn run_gui(tx: Sender<KeyboardMsg>) {
    let app_state = AppState {
        tx,
        handle: Default::default(),
    };

    let app = Application::new().unwrap();

    let mut window_builder = WindowBuilder::new(app.clone());
    window_builder.set_title("JACK keyboard");
    window_builder.set_handler(Box::new(app_state));
    let window = window_builder.build().unwrap();
    window.show();

    app.run(None);
}

#[derive(Debug)]
struct KeyboardMsg {
    note: Note,
    pressed: bool,
}

#[derive(Debug, Clone, Copy)]
enum Note {
    C4,
    CSharp4,
    D4,
    DSharp4,
    E4,
    F4,
    FSharp4,
    G4,
    GSharp4,
    A4,
    ASharp4,
    B4,
    C5,
}

impl Note {
    fn from_code(code: druid_shell::Code) -> Option<Note> {
        Some(match code {
            Code::KeyA => Note::C4,
            Code::KeyS => Note::D4,
            Code::KeyD => Note::E4,
            Code::KeyF => Note::F4,
            Code::KeyG => Note::G4,
            Code::KeyH => Note::A4,
            Code::KeyJ => Note::B4,
            Code::KeyK => Note::C5,

            Code::KeyW => Note::CSharp4,
            Code::KeyE => Note::DSharp4,
            Code::KeyT => Note::FSharp4,
            Code::KeyY => Note::GSharp4,
            Code::KeyU => Note::ASharp4,

            _ => return None,
        })
    }

    fn to_midi_value(self) -> u8 {
        match self {
            Note::C4 => 60,
            Note::CSharp4 => 61,
            Note::D4 => 62,
            Note::DSharp4 => 63,
            Note::E4 => 64,
            Note::F4 => 65,
            Note::FSharp4 => 66,
            Note::G4 => 67,
            Note::GSharp4 => 68,
            Note::A4 => 69,
            Note::ASharp4 => 70,
            Note::B4 => 71,
            Note::C5 => 72,
        }
    }
}
