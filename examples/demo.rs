use std::cell::RefCell;

use eframe::App;
use egui::{pos2, Color32, InnerResponse, Ui};
use egui_snarl::{
    ui::{Effects, Forbidden, NodeInPin, NodeOutPin, Pin, SnarlViewer},
    OutPin, Snarl,
};

#[derive(Clone)]
enum DemoNode {
    /// Node with single input.
    /// Displays the value of the input.
    Sink,

    /// Value node with a single output.
    /// The value is editable in UI.
    Integer(i32),

    /// Value node with a single output.
    String(String),

    /// Value node with a single output.
    ///
    /// It has two inputs, ediable if not connected.
    Add(Vec<i32>),

    /// Converts URI to Image
    Show(String),
}

struct DemoViewer;

impl SnarlViewer<DemoNode> for DemoViewer {
    fn node_picker(&mut self, _ui: &mut Ui) -> egui::InnerResponse<Option<DemoNode>> {
        todo!()
    }

    #[inline]
    fn connect(
        &mut self,
        _from: NodeOutPin<DemoNode>,
        to: NodeInPin<DemoNode>,
        effects: &mut Effects<DemoNode>,
    ) -> Result<(), Forbidden> {
        for remote in &to.remotes {
            effects.disconnect(
                OutPin {
                    node: remote.node_idx,
                    output: remote.pin_idx,
                },
                to.in_pin,
            );
        }

        Ok(())
    }

    fn size_hint(&self, _node: &DemoNode) -> egui::Vec2 {
        egui::vec2(100.0, 50.0)
    }

    fn title(&mut self, node: &DemoNode) -> &str {
        match node {
            DemoNode::Sink => "Sink",
            DemoNode::Integer(_) => "Integer",
            DemoNode::String(_) => "String",
            DemoNode::Add(_) => "Add",
            DemoNode::Show(_) => "Show",
        }
    }

    fn inputs(&mut self, node: &DemoNode) -> usize {
        match node {
            DemoNode::Sink => 1,
            DemoNode::Integer(_) => 0,
            DemoNode::String(_) => 0,
            DemoNode::Add(values) => values.len() + 1,
            DemoNode::Show(_) => 1,
        }
    }

    fn outputs(&mut self, node: &DemoNode) -> usize {
        match node {
            DemoNode::Sink => 0,
            DemoNode::Integer(_) => 1,
            DemoNode::String(_) => 1,
            DemoNode::Add(_) => 1,
            DemoNode::Show(_) => 1,
        }
    }

    fn show_input(
        &mut self,
        node: &RefCell<DemoNode>,
        pin: Pin<DemoNode>,
        ui: &mut Ui,
        _effects: &mut Effects<DemoNode>,
    ) -> egui::InnerResponse<Color32> {
        let demo_node = node.borrow().clone();
        match demo_node {
            DemoNode::Sink => {
                assert_eq!(pin.pin_idx, 0, "Sink node has only one input");

                match &*pin.remotes {
                    [] => {
                        let r = ui.label("None");
                        InnerResponse::new(Color32::GRAY, r)
                    }
                    [remote] => match *remote.node.borrow() {
                        DemoNode::Sink => unreachable!("Sink node has no outputs"),
                        DemoNode::Integer(value) => {
                            assert_eq!(remote.pin_idx, 0, "Integer node has only one output");
                            let r = ui.label(format!("{}", value));
                            InnerResponse::new(Color32::RED, r)
                        }
                        DemoNode::String(ref value) => {
                            assert_eq!(remote.pin_idx, 0, "String node has only one output");
                            let r = ui.label(format!("{:?}", value));
                            InnerResponse::new(Color32::RED, r)
                        }
                        DemoNode::Add(ref values) => {
                            assert_eq!(remote.pin_idx, 0, "Integer node has only one output");
                            let r = ui.label(format!("{}", values.iter().copied().sum::<i32>()));
                            InnerResponse::new(Color32::RED, r)
                        }
                        DemoNode::Show(ref uri) => {
                            assert_eq!(remote.pin_idx, 0, "Show node has only one output");

                            let image = egui::Image::new(uri)
                                .fit_to_original_size(1.0)
                                .show_loading_spinner(true);
                            let r = ui.add(image);

                            InnerResponse::new(Color32::GOLD, r)
                        }
                    },
                    _ => unreachable!("Sink input has only one wire"),
                }
            }
            DemoNode::Integer(_) => {
                unreachable!("Integer node has no inputs")
            }
            DemoNode::String(_) => {
                unreachable!("String node has no inputs")
            }
            DemoNode::Add(values) => match &*pin.remotes {
                [] => {
                    if pin.pin_idx < values.len() {
                        match &mut *node.borrow_mut() {
                            DemoNode::Add(values) => {
                                let r = ui.add(egui::DragValue::new(&mut values[pin.pin_idx]));
                                InnerResponse::new(Color32::GREEN, r)
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        assert_eq!(
                            pin.pin_idx,
                            values.len(),
                            "Add node has exactly one more inputs than values"
                        );

                        let r = ui.button("+");
                        if r.clicked() {
                            match &mut *node.borrow_mut() {
                                DemoNode::Add(values) => {
                                    values.push(0);
                                }
                                _ => unreachable!(),
                            }
                        }
                        InnerResponse::new(Color32::YELLOW, r)
                    }
                }
                [remote] => {
                    if pin.pin_idx >= values.len() {
                        match &mut *node.borrow_mut() {
                            DemoNode::Add(values) => {
                                values.resize(pin.pin_idx + 1, 0);
                            }
                            _ => unreachable!(),
                        }
                    }

                    let remote_node = remote.node.borrow().clone();
                    match remote_node {
                        DemoNode::Sink => unreachable!("Sink node has no outputs"),
                        DemoNode::Integer(value) => {
                            assert_eq!(remote.pin_idx, 0, "Integer node has only one output");
                            match &mut *node.borrow_mut() {
                                DemoNode::Add(values) => {
                                    values[pin.pin_idx] = value;
                                }
                                _ => unreachable!(),
                            }
                            let r = ui.label(format!("{}", value));
                            InnerResponse::new(Color32::RED, r)
                        }
                        DemoNode::Add(values) => {
                            let sum = values.iter().copied().sum::<i32>();

                            assert_eq!(remote.pin_idx, 0, "Integer node has only one output");
                            match &mut *node.borrow_mut() {
                                DemoNode::Add(values) => {
                                    values[pin.pin_idx] = sum;
                                }
                                _ => unreachable!(),
                            }
                            let r = ui.label(format!("{}", sum));
                            InnerResponse::new(Color32::RED, r)
                        }
                        DemoNode::Show(_) => {
                            unreachable!("Show node has no outputs")
                        }
                        DemoNode::String(_) => {
                            unreachable!("Invalid connection")
                        }
                    }
                }
                _ => unreachable!("Add node has only one wire"),
            },
            DemoNode::Show(_) => match &*pin.remotes {
                [] => match &mut *node.borrow_mut() {
                    DemoNode::Show(uri) => {
                        let r = ui.text_edit_singleline(uri);
                        InnerResponse::new(Color32::GREEN, r)
                    }
                    _ => unreachable!(),
                },
                [remote] => match remote.node.borrow().clone() {
                    DemoNode::Sink => unreachable!("Sink node has no outputs"),
                    DemoNode::Show(_) => {
                        unreachable!("Show node has no outputs")
                    }
                    DemoNode::Integer(_) | DemoNode::Add(_) => {
                        unreachable!("Invalid connection")
                    }
                    DemoNode::String(value) => match &mut *node.borrow_mut() {
                        DemoNode::Show(uri) => {
                            *uri = value.clone();
                            let r = ui.text_edit_singleline(&mut &**uri);
                            InnerResponse::new(Color32::GREEN, r)
                        }
                        _ => unreachable!(),
                    },
                },
                _ => unreachable!("Sink input has only one wire"),
            },
        }
    }

    fn show_output(
        &mut self,
        node: &RefCell<DemoNode>,
        pin: Pin<DemoNode>,
        ui: &mut Ui,
        _effects: &mut Effects<DemoNode>,
    ) -> egui::InnerResponse<Color32> {
        match *node.borrow_mut() {
            DemoNode::Sink => {
                unreachable!("Sink node has no outputs")
            }
            DemoNode::Integer(ref mut value) => {
                assert_eq!(pin.pin_idx, 0, "Integer node has only one output");
                let r = ui.add(egui::DragValue::new(value));
                InnerResponse::new(Color32::RED, r)
            }
            DemoNode::String(ref mut value) => {
                assert_eq!(pin.pin_idx, 0, "String node has only one output");
                let r = ui.text_edit_singleline(value);
                InnerResponse::new(Color32::GREEN, r)
            }
            DemoNode::Add(ref values) => {
                let sum = values.iter().copied().sum::<i32>();
                assert_eq!(pin.pin_idx, 0, "Add node has only one output");
                let r = ui.label(format!("{}", sum));
                InnerResponse::new(Color32::RED, r)
            }
            DemoNode::Show(_) => {
                let (_, r) = ui.allocate_exact_size(egui::Vec2::ZERO, egui::Sense::hover());
                InnerResponse::new(Color32::GOLD, r)
            }
        }
    }
}

pub struct DemoApp {
    snarl: Snarl<DemoNode>,
}

impl DemoApp {
    pub fn new() -> Self {
        let mut snarl = Snarl::new();

        snarl.add_node(DemoNode::Integer(42), pos2(10.0, 20.0));

        snarl.add_node(DemoNode::Add(vec![]), pos2(30.0, 80.0));

        snarl.add_node(DemoNode::Add(vec![]), pos2(40.0, 100.0));

        snarl.add_node(DemoNode::String("".to_owned()), pos2(20.0, 150.0));

        snarl.add_node(DemoNode::Show("".to_owned()), pos2(120.0, 20.0));

        snarl.add_node(DemoNode::Sink, pos2(190.0, 60.0));

        DemoApp { snarl }
    }
}

impl App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.snarl.show(&mut DemoViewer, egui::Id::new("snarl"), ui);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-snarl demo",
        native_options,
        Box::new(|_| Box::new(DemoApp::new())),
    )
}
