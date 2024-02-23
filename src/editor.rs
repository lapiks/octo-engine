use egui::{ahash::{HashSet, HashSetExt}, CentralPanel, Frame, TopBottomPanel, Ui, WidgetText};
use egui_dock::{AllowedSplits, DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};

use crate::{game::Game, renderer_context::RendererContext};


#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
enum GuiTab {
    GameView,
    Hierarchy,
    Inspector,
    RendererContext,
}

struct GuiContext<'a> {
    viewport_rect: &'a mut (bool, egui::Rect), 
    open_tabs: HashSet<GuiTab>,
    game: &'a Game,
    renderer: &'a RendererContext,
    game_texture: Option<egui::TextureId>,
}

pub struct Editor {
    viewport_rect: (bool, egui::Rect),
    pub style: Option<Style>,
    open_tabs: HashSet<GuiTab>,
    tree: DockState<GuiTab>,
}

impl TabViewer for GuiContext<'_> {
    type Tab = GuiTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        format!("{tab:?}").into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            GuiTab::GameView => self.game_view(ui),
            _ => {}
        }
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.open_tabs.remove(tab);
        true
    }
}

impl GuiContext<'_> {
    fn simple_demo_menu(&mut self, ui: &mut Ui) {
        ui.label("Egui widget example");
        ui.menu_button("Sub menu", |ui| {
            ui.label("hello :)");
        });
    }

    fn game_view(&mut self, ui: &mut Ui) {
        self.viewport_rect.1 = ui.clip_rect();
        if let Some(game_texture) = self.game_texture {
            ui.image(
                (game_texture, ui.available_size())
            );  
        }
    }
}

impl Editor {
    pub fn new() -> Self {
        let mut dock_state = DockState::new(
            vec![GuiTab::GameView]
        );
        dock_state.translations.tab_context_menu.eject_button = "Undock".to_owned();
        let tree = dock_state.main_surface_mut();
        let [game, _inspector] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![GuiTab::Inspector],
        );
        let [game, _hierarchy] = tree.split_left(
            game,
            0.2,
            vec![GuiTab::Hierarchy],
        );
        let [_, _] = tree.split_below(
            game, 
            0.8, 
            vec![GuiTab::RendererContext]
        );

        let mut open_tabs = HashSet::new();

        for node in dock_state[SurfaceIndex::main()].iter() {
            if let Some(tabs) = node.tabs() {
                for tab in tabs {
                    open_tabs.insert(*tab);
                }
            }
        }

        Self {
            viewport_rect: (true, egui::Rect::NOTHING),
            style: None,
            open_tabs,
            tree: dock_state,
        }   
    }

    pub fn run_ui(&mut self, ctx: &egui::Context, game: &Game, renderer: &RendererContext, game_texture: Option<egui::TextureId>) {
        let mut gui_context = GuiContext {
            viewport_rect: &mut self.viewport_rect,
            open_tabs: self.open_tabs.clone(),
            game,
            renderer,
            game_texture,
        };

        TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {

                });
            })
        });
        CentralPanel::default()
            // When displaying a DockArea in another UI, it looks better
            // to set inner margins to 0.
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let style = self
                    .style
                    .get_or_insert(Style::from_egui(ui.style()))
                    .clone();

                DockArea::new(&mut self.tree)
                    .style(style)
                    .show_close_buttons(true)
                    .show_add_buttons(false)
                    .draggable_tabs(true)
                    .show_tab_name_on_hover(false)
                    .allowed_splits(AllowedSplits::All)
                    .show_window_close_buttons(true)
                    .show_window_collapse_buttons(true)
                    .show_inside(ui, &mut gui_context);
            });
    }

    pub fn viewport_changed(&self) -> bool {
        self.viewport_rect.0
    }

    pub fn viewport_rect(&self) -> egui::Rect {
        self.viewport_rect.1
    }
}