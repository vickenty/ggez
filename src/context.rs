

use sdl2::{self, Sdl};
use sdl2::render::Renderer;
use sdl2::video::Window;

use std::fmt;
use std::path;

use audio;
use conf;
use filesystem::Filesystem;
use graphics;
use timer;
use util;
use GameError;
use GameResult;


/// A `Context` is an object that holds on to global resources.
/// It basically tracks hardware state such as the screen, audio
/// system, timers, and so on.  Generally this type is **not** thread-
/// safe and only one `Context` can exist at a time.  Trying to create
/// another one will fail.  In normal usage you don't have to worry
/// about this because it gets created and managed by the `Game` object,
/// and is handed to your `GameState` for use in drawing and such.
///
/// Most functions that interact with the hardware, for instance
/// drawing things, playing sounds, or loading resources (which then
/// need to be transformed into a format the hardware likes) will need
/// to access the `Context`.
pub struct Context<'a> {
    pub conf: conf::Conf,
    pub sdl_context: Sdl,
    pub renderer: Renderer<'a>,
    pub filesystem: Filesystem,
    pub gfx_context: graphics::GraphicsContext,
    pub event_context: sdl2::EventSubsystem,
    pub timer_context: timer::TimeContext,
    pub dpi: (f32, f32, f32),
    pub audio_context: audio::AudioContext,
}

impl<'a> fmt::Debug for Context<'a> {
    // TODO: Make this more useful.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Context: {:p}>", self)
    }
}



fn init_window(video: sdl2::VideoSubsystem,
               window_title: &str,
               screen_width: u32,
               screen_height: u32)
               -> GameResult<Window> {

    // Can't hurt
    let _ = sdl2::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "best");

    video.window(window_title, screen_width, screen_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| GameError::VideoError(format!("{}", e)))
}

/// Sets the window icon from the Conf window_icon field.
/// Assumes an empty string in the conf's window_icon
/// means to do nothing.
fn set_window_icon(context: &mut Context) -> GameResult<()> {
    if !context.conf.window_icon.is_empty() {
        // Grrr, hackhackhack here with the icon path clone.
        let icon_path = context.conf.window_icon.clone();
        let path = path::Path::new(&icon_path);
        let icon_surface = util::load_surface(context, path)?;

        if let Some(window) = context.renderer.window_mut() {
            window.set_icon(icon_surface);
        }
    };
    Ok(())
}

impl<'a> Context<'a> {
    /// Tries to create a new Context using settings from the given config file.
    /// Usually called by the engine as part of the set-up code.
    pub fn from_conf(conf: conf::Conf,
                     fs: Filesystem,
                     sdl_context: Sdl)
                     -> GameResult<Context<'a>> {
        let screen_width = conf.window_width;
        let screen_height = conf.window_height;

        let video = sdl_context.video()?;
        let window = {

            let window_title = &conf.window_title;
            init_window(video, &window_title, screen_width, screen_height)?
        };
        let display_index = window.display_index()?;
        let dpi = window.subsystem().display_dpi(display_index)?;

        let renderer = window.renderer()
            .accelerated()
            .build()?;

        let audio_context = audio::AudioContext::new()?;
        let event_context = sdl_context.event()?;
        let timer_context = timer::TimeContext::new();

        let mut ctx = Context {
            conf: conf,
            sdl_context: sdl_context,
            renderer: renderer,
            filesystem: fs,
            gfx_context: graphics::GraphicsContext::new(),
            dpi: dpi,

            event_context: event_context,
            timer_context: timer_context,

            audio_context: audio_context,
        };

        set_window_icon(&mut ctx)?;

        Ok(ctx)
    }

    /// Tries to create a new Context loading a config
    /// file from its default path, using the given Conf
    /// object as a default if none is found.
    pub fn load_from_conf(id: &str, default_config: conf::Conf) -> GameResult<Context<'a>> {

        let sdl_context = sdl2::init()?;
        let mut fs = Filesystem::new(id)?;

        // TODO: Verify config version == this version
        let config = fs.read_config().unwrap_or(default_config);

        Context::from_conf(config, fs, sdl_context)

    }

    /// Prints out information on the resources subsystem.
    pub fn print_resource_stats(&mut self) {
        match self.filesystem.print_all() {
            Err(e) => println!("Error printing out filesystem info: {}", e),
            _ => (),
        }
    }

    /// Triggers a Quit event.
    pub fn quit(&mut self) -> GameResult<()> {
        let now_dur = timer::get_time_since_start(self);
        let now = timer::duration_to_f64(now_dur);
        let e = sdl2::event::Event::Quit { timestamp: now as u32 };
        // println!("Pushing event {:?}", e);
        self.event_context
            .push_event(e)
            .map_err(GameError::from)
    }
}
