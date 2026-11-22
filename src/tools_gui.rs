use crate::{
    biome_installation::BiomeInstallation,
    tool_id::ToolId,
    tool_manager::ToolManager,
};
use arboard::Clipboard;
use eframe::egui;
use std::{
    error::Error,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
};

pub(crate) struct ToolsGui;

enum ToolOperation {
    Install,
    Reinstall,
    Uninstall,
}

struct RunningOperation {
    operation: ToolOperation,
    receiver: Receiver<Result<String, String>>,
}

impl RunningOperation {
    fn label(&self) -> &'static str {
        match self.operation {
            ToolOperation::Install => "Installing Biome…",
            ToolOperation::Reinstall => "Reinstalling Biome…",
            ToolOperation::Uninstall => "Uninstalling Biome…",
        }
    }
}

impl ToolsGui {
    pub(crate) fn run(
        project_root: Option<PathBuf>,
    ) -> Result<(), Box<dyn Error>> {
        let mut status = ToolManager::status(
            ToolId::Biome,
            project_root.as_deref(),
        )?;

        let mut message = String::new();
        let mut clipboard = Clipboard::new().ok();
        let mut running_operation: Option<RunningOperation> = None;

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Standardizer")
                .with_inner_size([760.0, 440.0])
                .with_min_inner_size([620.0, 360.0]),
            ..Default::default()
        };

        eframe::run_ui_native(
            "standardizer",
            options,
            move |ui, _frame| {
                // -----------------------------------------------------------
                // Check asynchronous operation result
                // -----------------------------------------------------------

                let operation_result = running_operation
                    .as_ref()
                    .and_then(|operation| operation.receiver.try_recv().ok());

                if let Some(result) = operation_result {
                    message = match result {
                        Ok(result) => result,
                        Err(error) => error,
                    };

                    running_operation = None;

                    match ToolManager::status(
                        ToolId::Biome,
                        project_root.as_deref(),
                    ) {
                        Ok(new_status) => {
                            status = new_status;
                        }
                        Err(error) => {
                            message = format!(
                                "{message}\nStatus refresh failed: {error}"
                            );
                        }
                    }
                }

                if running_operation.is_some() {
                    ui.ctx().request_repaint();
                }

                // -----------------------------------------------------------
                // UI
                // -----------------------------------------------------------

                egui::CentralPanel::default().show(ui, |ui| {
                    ui.add_space(8.0);

                    ui.heading("Standardizer");
                    ui.label("Tool manager");

                    ui.add_space(18.0);

                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.horizontal(|ui| {
                            ui.heading("Biome");
                            ui.add_space(8.0);

                            if status.active().is_some() {
                                ui.label("Available");
                            } else {
                                ui.label("Not installed");
                            }

                            if let Some(operation) = &running_operation {
                                ui.add_space(12.0);
                                ui.spinner();
                                ui.label(operation.label());
                            }
                        });

                        ui.add_space(10.0);

                        let required_version =
                            BiomeInstallation::VERSION.to_owned();

                        let active_version = status
                            .active_version()
                            .unwrap_or("Not available")
                            .to_owned();

                        let active_source = status
                            .active()
                            .map(|tool| tool.source().label())
                            .unwrap_or("none")
                            .to_owned();

                        let active_path = status
                            .active()
                            .map(|tool| {
                                tool.path().display().to_string()
                            })
                            .unwrap_or_else(|| {
                                "Not available".to_owned()
                            });

                        let managed_state = if status.managed_installed() {
                            "Installed"
                        } else {
                            "Not installed"
                        }
                        .to_owned();

                        let managed_path =
                            status.managed_path().display().to_string();

                        egui::Grid::new("biome_status")
                            .num_columns(3)
                            .spacing([24.0, 8.0])
                            .striped(true)
                            .show(ui, |ui| {
                                Self::value_row(
                                    ui,
                                    &mut clipboard,
                                    "Required version",
                                    &required_version,
                                    true,
                                );

                                Self::value_row(
                                    ui,
                                    &mut clipboard,
                                    "Active version",
                                    &active_version,
                                    status.active_version().is_some(),
                                );

                                Self::value_row(
                                    ui,
                                    &mut clipboard,
                                    "Active source",
                                    &active_source,
                                    status.active().is_some(),
                                );

                                Self::value_row(
                                    ui,
                                    &mut clipboard,
                                    "Active path",
                                    &active_path,
                                    status.active().is_some(),
                                );

                                Self::value_row(
                                    ui,
                                    &mut clipboard,
                                    "Managed copy",
                                    &managed_state,
                                    false,
                                );

                                Self::value_row(
                                    ui,
                                    &mut clipboard,
                                    "Managed path",
                                    &managed_path,
                                    true,
                                );
                            });

                        ui.add_space(16.0);

                        let busy = running_operation.is_some();

                        ui.horizontal(|ui| {
                            let install_label =
                                if status.managed_installed() {
                                    "Reinstall"
                                } else {
                                    "Install"
                                };

                            if ui
                                .add_enabled(
                                    !busy,
                                    egui::Button::new(install_label),
                                )
                                .clicked()
                            {
                                message.clear();

                                running_operation = Some(
                                    if status.managed_installed() {
                                        Self::start_reinstall()
                                    } else {
                                        Self::start_install()
                                    },
                                );
                            }

                            if ui
                                .add_enabled(
                                    !busy && status.managed_installed(),
                                    egui::Button::new("Uninstall"),
                                )
                                .clicked()
                            {
                                message.clear();

                                running_operation =
                                    Some(Self::start_uninstall());
                            }

                            if ui
                                .add_enabled(
                                    !busy,
                                    egui::Button::new("Refresh"),
                                )
                                .clicked()
                            {
                                match ToolManager::status(
                                    ToolId::Biome,
                                    project_root.as_deref(),
                                ) {
                                    Ok(new_status) => {
                                        status = new_status;
                                        message =
                                            "Status refreshed".to_owned();
                                    }
                                    Err(error) => {
                                        message = format!(
                                            "Refresh failed: {error}"
                                        );
                                    }
                                }
                            }
                        });

                        if let Some(operation) = &running_operation {
                            ui.add_space(12.0);

                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(operation.label());
                            });
                        }
                    });

                    if !message.is_empty() {
                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.horizontal_wrapped(|ui| {
                            ui.add(
                                egui::Label::new(&message)
                                    .selectable(true),
                            );

                            if ui.small_button("Copy").clicked() {
                                if let Err(error) =
                                    Self::copy_text(
                                        &mut clipboard,
                                        &message,
                                    )
                                {
                                    eprintln!(
                                        "standardizer: clipboard unavailable: {error}"
                                    );
                                }
                            }
                        });
                    }

                    ui.add_space(18.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(
                        "Standardizer only removes tools that it installed itself.",
                    );
                });
            },
        )?;

        Ok(())
    }

    fn start_install() -> RunningOperation {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let result = ToolManager::install(ToolId::Biome)
                .map(|path| {
                    format!(
                        "Biome {} installed: {}",
                        BiomeInstallation::VERSION,
                        path.display()
                    )
                })
                .map_err(|error| {
                    format!("Installation failed: {error}")
                });

            let _ = sender.send(result);
        });

        RunningOperation {
            operation: ToolOperation::Install,
            receiver,
        }
    }

    fn start_reinstall() -> RunningOperation {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let result = ToolManager::reinstall(ToolId::Biome)
                .map(|path| {
                    format!(
                        "Biome {} reinstalled: {}",
                        BiomeInstallation::VERSION,
                        path.display()
                    )
                })
                .map_err(|error| {
                    format!("Reinstallation failed: {error}")
                });

            let _ = sender.send(result);
        });

        RunningOperation {
            operation: ToolOperation::Reinstall,
            receiver,
        }
    }

    fn start_uninstall() -> RunningOperation {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let result = ToolManager::uninstall(ToolId::Biome)
                .map(|removed| {
                    if removed {
                        "Managed Biome removed".to_owned()
                    } else {
                        "No managed Biome installation".to_owned()
                    }
                })
                .map_err(|error| {
                    format!("Uninstall failed: {error}")
                });

            let _ = sender.send(result);
        });

        RunningOperation {
            operation: ToolOperation::Uninstall,
            receiver,
        }
    }

    fn copy_text(
        clipboard: &mut Option<Clipboard>,
        text: &str,
    ) -> Result<(), Box<dyn Error>> {
        if clipboard.is_none() {
            *clipboard = Some(Clipboard::new()?);
        }

        let clipboard = clipboard
            .as_mut()
            .ok_or("clipboard unavailable")?;

        clipboard.set_text(text.to_owned())?;

        Ok(())
    }

    fn value_row(
        ui: &mut egui::Ui,
        clipboard: &mut Option<Clipboard>,
        name: &str,
        value: &str,
        copyable: bool,
    ) {
        ui.label(name);

        ui.add(
            egui::Label::new(value)
                .selectable(true),
        );

        if copyable {
            if ui.small_button("Copy").clicked() {
                if let Err(error) =
                    Self::copy_text(clipboard, value)
                {
                    eprintln!(
                        "standardizer: clipboard unavailable: {error}"
                    );
                }
            }
        } else {
            ui.label("");
        }

        ui.end_row();
    }
}
