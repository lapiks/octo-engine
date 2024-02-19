use egui::{ahash::{HashSet, HashSetExt}, CentralPanel, Frame, TopBottomPanel, Ui, WidgetText};
use egui_dock::{AllowedSplits, DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};

pub struct GuiContext {
    pub style: Option<Style>,
    open_tabs: HashSet<String>,
}

pub struct Gui {
    context: GuiContext,
    tree: DockState<String>,
}

impl TabViewer for GuiContext {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Viewport" => self.viewport(ui),
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn context_menu(
        &mut self,
        ui: &mut Ui,
        tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
        match tab.as_str() {
            "Simple Demo" => self.simple_demo_menu(ui),
            _ => {
                ui.label(tab.to_string());
                ui.label("This is a context menu");
            }
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        ["Inspector"].contains(&tab.as_str())
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.open_tabs.remove(tab);
        true
    }
}

impl GuiContext {
    fn simple_demo_menu(&mut self, ui: &mut Ui) {
        ui.label("Egui widget example");
        ui.menu_button("Sub menu", |ui| {
            ui.label("hello :)");
        });
    }

    fn viewport(&mut self, ui: &mut Ui) {
        
    }
}

impl Default for Gui {
    fn default() -> Self {
        let mut dock_state = DockState::new(
            vec!["Viewport".to_owned()]
        );
        dock_state.translations.tab_context_menu.eject_button = "Undock".to_owned();
        let [a, b] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.3,
            vec!["Inspector".to_owned()],
        );
        let [_, _] = dock_state.main_surface_mut().split_below(
            a,
            0.7,
            vec!["File Browser".to_owned(), "Asset Manager".to_owned()],
        );
        let [_, _] =
            dock_state
                .main_surface_mut()
                .split_below(b, 0.5, vec!["Hierarchy".to_owned()]);

        let mut open_tabs = HashSet::new();

        for node in dock_state[SurfaceIndex::main()].iter() {
            if let Some(tabs) = node.tabs() {
                for tab in tabs {
                    open_tabs.insert(tab.clone());
                }
            }
        }
        let context = GuiContext {
            style: None,
            open_tabs,
        };

        Self {
            context,
            tree: dock_state,
        }
    }
}

impl Gui {
    pub fn run_ui(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    // allow certain tabs to be toggled
                    for tab in &["File Browser", "Asset Manager"] {
                        if ui
                            .selectable_label(self.context.open_tabs.contains(*tab), *tab)
                            .clicked()
                        {
                            if let Some(index) = self.tree.find_tab(&tab.to_string()) {
                                self.tree.remove_tab(index);
                                self.context.open_tabs.remove(*tab);
                            } else {
                                self.tree[SurfaceIndex::main()]
                                    .push_to_focused_leaf(tab.to_string());
                            }

                            ui.close_menu();
                        }
                    }
                });
            })
        });
        CentralPanel::default()
            // When displaying a DockArea in another UI, it looks better
            // to set inner margins to 0.
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let style = self
                    .context
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
                    .show_inside(ui, &mut self.context);
            });
    }
}