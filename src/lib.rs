use xilem::core::{Edit, lens, map_action};
use xilem::masonry::properties::types::AsUnit;
use xilem::style::Style;
use xilem::view::{
    FlexExt, MainAxisAlignment, flex_col, flex_row, indexed_stack, portal, sized_box, text_button,
};
use xilem::{AppState, WidgetView, WindowId, WindowView, window};

pub mod animation;
pub mod math;
pub mod thing;
pub mod units;
pub mod utils;
pub mod viewport;

use crate::thing::Thing;
use crate::viewport::Viewport;

#[derive(Copy, Clone)]
enum Tab {
    Data,
    Preview,
}

pub struct State {
    running: bool,
    window_id: WindowId,
    tab: Tab,
    things: Vec<Thing>,
    viewport: Viewport,
}

impl AppState for State {
    fn keep_running(&self) -> bool {
        self.running
    }
}

impl State {
    pub fn new(things: Vec<Thing>) -> Self {
        let viewport = Viewport::init(&things);
        Self {
            running: true,
            window_id: WindowId::next(),
            tab: Tab::Preview,
            viewport,
            things,
        }
    }

    pub fn data_view(&mut self) -> impl WidgetView<Edit<Self>> + use<> {
        let things = self
            .things
            .iter()
            .enumerate()
            .map(|(i, _)| {
                map_action(
                    lens(Thing::view, move |state: &mut Self, ()| {
                        state.things.get_mut(i).unwrap()
                    }),
                    move |state: &mut Self, delete| {
                        if delete {
                            state.things.remove(i);
                        }
                    },
                )
            })
            .collect::<Vec<_>>();
        let new_btn = flex_row(text_button("Add new", |state: &mut Self| {
            state.things.push(Thing::default());
        }))
        .must_fill_major_axis(true)
        .main_axis_alignment(MainAxisAlignment::Center);
        let list = portal(flex_col((things, new_btn)).padding(10.));
        let controls = flex_row(text_button("Preview", |state: &mut Self| {
            state.viewport = Viewport::init(&state.things);
            state.tab = Tab::Preview;
        }))
        .main_axis_alignment(MainAxisAlignment::Center)
        .background_color(Viewport::FOOTER_AREA_COLOR);
        flex_col((
            sized_box(list).expand_height().width(800.px()).flex(1.),
            sized_box(controls).expand_width().height(75.px()),
        ))
        .must_fill_major_axis(true)
        .main_axis_alignment(MainAxisAlignment::End)
    }

    pub fn view(&mut self) -> impl Iterator<Item = WindowView<Self>> + use<> {
        std::iter::once(
            window(
                self.window_id,
                format!("Scale Comparison{}", self.viewport.animation.info()),
                indexed_stack((self.data_view(), self.viewport.view())).active(self.tab as usize),
            )
            .with_options(|options: xilem::WindowOptions<_>| {
                options.on_close(|state: &mut State| state.running = false)
            }),
        )
    }
}
