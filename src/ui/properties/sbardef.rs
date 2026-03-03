use crate::document::actions::TreeAction;
use crate::document::{DocumentAction, determine_insertion_point};
use crate::models::sbardef::{
    AnimationDef, CanvasDef, ComponentDef, ComponentType, Element, ElementWrapper, ExportTarget,
    FaceDef, GraphicDef, ListDef, NumberDef, NumberType, SBarDefFile, StringDef, TextHelperDef,
};
use crate::ui::context_menu::ContextMenu;
use crate::ui::properties::editor::{LayerContext, PropertiesUI, TickContext, ViewportContext};
use crate::ui::properties::{
    colors, common, descriptions,
    editor::{LumpUI, PropertyContext},
    font_cache::FontCache,
    text_helper,
};
use eframe::egui;
use std::collections::HashSet;

const PROP_TAB_KEY: &str = "cacoco_sbar_tab_state";

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
enum PropertyTab {
    Properties,
    Conditions,
}

impl LumpUI for SBarDefFile {
    fn draw_properties(&mut self, ui: &mut egui::Ui, ctx: &PropertyContext) -> bool {
        let mut changed = false;

        let mut current_tab = ui.data(|d| {
            d.get_temp(egui::Id::new(PROP_TAB_KEY))
                .unwrap_or(PropertyTab::Properties)
        });

        if let Some(path) = ctx.selection.iter().next() {
            if path.len() > 1 {
                ui.columns(2, |uis| {
                    if crate::ui::shared::section_header_button(
                        &mut uis[0],
                        "Properties",
                        None,
                        current_tab == PropertyTab::Properties,
                    )
                    .clicked()
                    {
                        current_tab = PropertyTab::Properties;
                    }
                    if crate::ui::shared::section_header_button(
                        &mut uis[1],
                        "Conditions",
                        None,
                        current_tab == PropertyTab::Conditions,
                    )
                    .clicked()
                    {
                        current_tab = PropertyTab::Conditions;
                    }
                });
                ui.add_space(3.0);
                ui.separator();
                ui.add_space(4.0);
            }
        }

        let font_cache = FontCache::new(self);
        let current_target = ctx.target;

        if let Some(path) = ctx.selection.iter().next() {
            match current_tab {
                PropertyTab::Properties => {
                    if path.len() > 1 {
                        if let Some(el) = self.get_element_mut(path) {
                            ui.vertical_centered(|ui| {
                                if el._cacoco_text.is_none() {
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 210.0).max(0.0) / 2.0);
                                        ui.label("Name:");
                                        let mut name = el._cacoco_name.clone().unwrap_or_default();
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut name)
                                                    .desired_width(150.0),
                                            )
                                            .changed()
                                        {
                                            el._cacoco_name =
                                                if name.is_empty() { None } else { Some(name) };
                                            changed = true;
                                        }
                                    });
                                    ui.add_space(4.0);
                                }
                                changed |= common::draw_transform_editor(ui, el, current_target);
                                ui.add_space(4.0);
                                if el._cacoco_text.is_some() {
                                    changed |= text_helper::draw_text_helper_editor(
                                        ui,
                                        el,
                                        &font_cache,
                                        ctx.assets,
                                    );
                                } else if el.has_specific_fields() {
                                    changed |= el.draw_specific_fields(
                                        ui,
                                        &font_cache,
                                        ctx.assets,
                                        ctx.state,
                                    );
                                }
                            });
                        }
                    } else if let Some(bar) = self.data.status_bars.get_mut(path[0]) {
                        if let Some(reason) = &bar._cacoco_system_locked {
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new(reason).weak());
                                if path[0] == 0 {
                                    ui.label("Managed Non-Fullscreen Slot.");
                                    changed |= ui
                                        .add(egui::DragValue::new(&mut bar.height).range(0..=200))
                                        .changed();
                                }
                            });
                        } else {
                            changed |= common::draw_root_statusbar_fields(ui, bar);
                        }
                    }
                }
                PropertyTab::Conditions => {
                    if path.len() > 1 {
                        if let Some(el) = self.get_element_mut(path) {
                            changed |= crate::ui::properties::conditions::draw_conditions_editor(
                                ui, el, ctx.assets, ctx.state,
                            );
                        }
                    }
                }
            }
        }

        ui.data_mut(|d| d.insert_temp(egui::Id::new(PROP_TAB_KEY), current_tab));
        changed
    }

    fn tick(&self, _ctx: &mut TickContext) {}

    fn draw_layer_list(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut LayerContext,
    ) -> (Vec<DocumentAction>, bool) {
        let mut actions = Vec::new();
        let changed = false;

        let bar_count = self.data.status_bars.len();
        let is_demo_bar =
            self.target == ExportTarget::Basic && *ctx.current_item_idx == bar_count - 1;

        if is_demo_bar {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(
                    egui::RichText::new("Not Editable.")
                        .color(egui::Color32::from_rgb(200, 100, 100))
                        .strong(),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("The KEX Demo Slot must remain empty.")
                        .weak()
                        .italics(),
                );
                ui.add_space(4.0);
                ui.label(egui::RichText::new(
                    "Switch to another layout to add layers.",
                ));
            });
            return (actions, false);
        }

        let layers_menu_id = ui.make_persistent_id("layers_header_new_menu");
        let is_menu_open = ContextMenu::get(ui, layers_menu_id).is_some();
        let header_res = crate::ui::shared::heading_action_button(
            ui,
            "Layers",
            Some("New Layer..."),
            is_menu_open,
        );

        if header_res.clicked() {
            ContextMenu::open(ui, layers_menu_id, header_res.rect.left_bottom());
        }

        if let Some(menu) = ContextMenu::get(ui, layers_menu_id) {
            ContextMenu::show(ui, menu, header_res.clicked(), |ui: &mut egui::Ui| {
                let mut new_element = None;
                let target = self.target;
                let is_extended = target == ExportTarget::Extended;

                let default_hud_font = self.data.hud_fonts.first().map(|font| font.name.clone());
                let default_num_font = self.data.number_fonts.first().map(|font| font.name.clone());
                let has_hud = default_hud_font.is_some();
                let has_num = default_num_font.is_some();

                if ContextMenu::button(ui, "Canvas Group", true) {
                    new_element = Some(ElementWrapper {
                        data: Element::Canvas(CanvasDef::default()),
                        ..Default::default()
                    });
                }
                if is_extended && ContextMenu::button(ui, "List Container", true) {
                    new_element = Some(ElementWrapper {
                        data: Element::List(ListDef::default()),
                        ..Default::default()
                    });
                }
                if is_extended && ContextMenu::button(ui, "Native Container", true) {
                    new_element = Some(ElementWrapper {
                        data: Element::Native(CanvasDef::default()),
                        ..Default::default()
                    });
                }
                if ContextMenu::button(ui, "Text String", has_hud) {
                    let mut el = ElementWrapper {
                        data: Element::Canvas(CanvasDef::default()),
                        _cacoco_text: Some(TextHelperDef {
                            text: "NEW TEXT".to_string(),
                            font: default_hud_font.clone().unwrap(),
                            spacing: 0,
                        }),
                        ..Default::default()
                    };
                    let fonts = FontCache::new(self);
                    text_helper::rebake_text(&mut el, ctx.assets, &fonts);
                    new_element = Some(el);
                }
                ui.separator();
                if ContextMenu::button(ui, "Graphic", true) {
                    new_element = Some(ElementWrapper {
                        data: Element::Graphic(GraphicDef {
                            patch: "HICACOCO".to_string(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                if ContextMenu::button(ui, "Animation", true) {
                    new_element = Some(ElementWrapper {
                        data: Element::Animation(AnimationDef::default()),
                        ..Default::default()
                    });
                }
                if ContextMenu::button(ui, "Doomguy", true) {
                    new_element = Some(ElementWrapper {
                        data: Element::Face(FaceDef::default()),
                        ..Default::default()
                    });
                }
                if ContextMenu::button(ui, "Face Background", true) {
                    new_element = Some(ElementWrapper {
                        data: Element::FaceBackground(FaceDef::default()),
                        ..Default::default()
                    });
                }
                ui.separator();
                if is_extended && ContextMenu::button(ui, "Dynamic String", has_hud) {
                    new_element = Some(ElementWrapper {
                        data: Element::String(StringDef {
                            font: default_hud_font.clone().unwrap(),
                            type_: 1,
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                if ContextMenu::button(ui, "Number", has_num) {
                    new_element = Some(ElementWrapper {
                        data: Element::Number(NumberDef {
                            font: default_num_font.clone().unwrap(),
                            type_: NumberType::Health,
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                if ContextMenu::button(ui, "Selected Ammo", has_num) {
                    new_element = Some(ElementWrapper {
                        data: Element::Number(NumberDef {
                            font: default_num_font.clone().unwrap(),
                            type_: NumberType::AmmoSelected,
                            common: crate::models::sbardef::CommonAttrs::selected_ammo_check(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                if ContextMenu::button(ui, "Percent", has_num) {
                    new_element = Some(ElementWrapper {
                        data: Element::Percent(NumberDef {
                            font: default_num_font.clone().unwrap(),
                            type_: NumberType::Health,
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                if ContextMenu::button(ui, "Selected Ammo %", has_num) {
                    new_element = Some(ElementWrapper {
                        data: Element::Percent(NumberDef {
                            font: default_num_font.unwrap(),
                            type_: NumberType::AmmoSelected,
                            common: crate::models::sbardef::CommonAttrs::selected_ammo_check(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                ui.separator();
                if is_extended && ContextMenu::button(ui, "Component", has_hud) {
                    new_element = Some(ElementWrapper {
                        data: Element::Component(ComponentDef {
                            font: default_hud_font.unwrap(),
                            type_: ComponentType::Time,
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }

                if let Some(element) = new_element {
                    let (parent_path, insert_idx) =
                        determine_insertion_point(self, ctx.selection, *ctx.current_item_idx);
                    actions.push(DocumentAction::UndoSnapshot);
                    actions.push(DocumentAction::Tree(TreeAction::Add {
                        parent_path,
                        insert_idx,
                        element,
                    }));
                    ContextMenu::close(ui);
                }
            });
        }

        if !self.data.status_bars.is_empty() {
            let bar_idx = *ctx.current_item_idx;
            egui::ScrollArea::vertical()
                .id_salt("layers_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(2, 0))
                        .show(ui, |ui| {
                            crate::ui::layers::tree::draw_layer_tree_root(
                                ui,
                                self,
                                bar_idx,
                                ctx.selection,
                                ctx.selection_pivot,
                                ctx.assets,
                                ctx.state,
                                &mut actions,
                                ctx.confirmation_modal,
                            );
                        });
                    ui.add_space(2.0);
                });
        }

        (actions, changed)
    }

    fn header_info(&self, selection: &HashSet<Vec<usize>>) -> (String, String, egui::Color32) {
        if let Some(path) = selection.iter().next() {
            if path.len() > 1 {
                if let Some(el) = self.get_element(path) {
                    let color = colors::get_layer_color(el)
                        .unwrap_or(egui::Color32::TRANSPARENT)
                        .linear_multiply(0.05);
                    return (
                        el.display_name(),
                        descriptions::get_helper_text(el).to_string(),
                        color,
                    );
                }
            } else {
                return (
                    format!("Layout #{}", path[0]),
                    "Root configuration for a HUD layout.".to_string(),
                    egui::Color32::from_white_alpha(10),
                );
            }
        }
        (
            "SBARDEF".into(),
            "Select a layer to edit properties.".into(),
            egui::Color32::TRANSPARENT,
        )
    }

    fn get_preview_content(
        &self,
        ui: &egui::Ui,
        ctx: &PropertyContext,
    ) -> Option<crate::ui::properties::preview::PreviewContent> {
        let path = ctx.selection.iter().next()?;
        if path.len() > 1 {
            let font_cache = FontCache::new(self);
            return self
                .get_element(path)?
                .get_preview_content(ui, &font_cache, ctx.state);
        }
        None
    }

    fn render_viewport(&self, ui: &mut egui::Ui, ctx: &mut ViewportContext) -> Vec<DocumentAction> {
        use crate::render::{self, RenderPass};
        let mut actions = Vec::new();

        let bar_idx = ctx
            .current_item_idx
            .min(self.data.status_bars.len().saturating_sub(1));
        let bar = &self.data.status_bars[bar_idx];

        crate::ui::viewport::render_statusbar_workspace(ui, bar, ctx.assets, ctx.state, ctx.proj);

        let root_y = if bar.fullscreen_render {
            0.0
        } else {
            200.0 - bar.height as f32
        };

        if ctx.selection_mode && !ctx.is_panning {
            let mut hit_result = None;
            let hit_ctx = render::RenderContext {
                painter: ui.painter(),
                assets: ctx.assets,
                file: self,
                state: ctx.state,
                time: ui.input(|i| i.time),
                fps: ctx.state.viewer.display_fps,
                mouse_pos: ctx.state.interaction.virtual_mouse_pos,
                selection: ctx.selection,
                pass: RenderPass::Background,
                proj: ctx.proj,
                is_dragging: ctx.viewport_res.dragged_by(egui::PointerButton::Primary),
                is_viewport_clicked: ctx.viewport_res.contains_pointer() && ctx.primary_down,
                is_native: false,
            };

            if ctx.viewport_res.hovered() {
                for (idx, child) in bar.children.iter().enumerate().rev() {
                    let mut path = vec![bar_idx, idx];
                    if let Some(hit) = render::hit_test(
                        &hit_ctx,
                        child,
                        egui::pos2(ctx.proj.origin_x, root_y),
                        &mut path,
                        true,
                        ctx.container_mode,
                    ) {
                        hit_result = Some(hit);
                        break;
                    }
                }
            }

            ctx.state.interaction.hovered_path = hit_result.clone();

            if ctx.primary_pressed && ctx.viewport_res.hovered() {
                if let Some(path) = hit_result {
                    ctx.state.interaction.grabbed_path = Some(path.clone());
                    if ui.input(|i| i.modifiers.shift) {
                        actions.push(DocumentAction::Tree(TreeAction::ToggleSelection(vec![
                            path,
                        ])));
                    } else {
                        actions.push(DocumentAction::Tree(TreeAction::Select(vec![path])));
                    }
                }
            }
        } else {
            ctx.state.interaction.hovered_path = None;
        }

        let draw_pass = |painter: &egui::Painter, pass: RenderPass| {
            let render_ctx = render::RenderContext {
                painter,
                assets: ctx.assets,
                file: self,
                state: ctx.state,
                time: ui.input(|i| i.time),
                fps: ctx.state.viewer.display_fps,
                mouse_pos: ctx.state.interaction.virtual_mouse_pos,
                selection: ctx.selection,
                pass,
                proj: ctx.proj,
                is_dragging: ctx.viewport_res.dragged_by(egui::PointerButton::Primary),
                is_viewport_clicked: ctx.viewport_res.contains_pointer() && ctx.primary_down,
                is_native: false,
            };

            for (idx, child) in bar.children.iter().enumerate() {
                let mut path = vec![bar_idx, idx];
                render::draw_element_wrapper(
                    &render_ctx,
                    child,
                    egui::pos2(ctx.proj.origin_x, root_y),
                    &mut path,
                    true,
                );
            }
        };

        draw_pass(ui.painter(), RenderPass::Background);

        if !ctx.selection.is_empty() && ctx.state.interaction.strobe_timer > 0.0 {
            draw_pass(ui.painter(), RenderPass::Foreground);
        }

        if ctx.selection_mode || ctx.state.interaction.strobe_timer > 0.0 {
            ui.ctx().request_repaint();
        }

        actions
    }
}
