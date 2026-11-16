use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GuiTheme {
    #[default]
    Macchiato,
    Latte,
    HighContrast,
}

impl GuiTheme {
    pub fn is_dark(&self) -> bool {
        match self {
            GuiTheme::Macchiato | GuiTheme::HighContrast => true,
            GuiTheme::Latte => false,
        }
    }

    pub fn bg(&self) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => egui::Color32::from_rgb(24, 24, 37), // Base
            GuiTheme::Latte => egui::Color32::from_rgb(239, 241, 245),  // Base
            GuiTheme::HighContrast => egui::Color32::from_rgb(0, 0, 0),
        }
    }

    pub fn top_panel_bg(&self) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => egui::Color32::from_rgb(30, 30, 46), // Mantle
            GuiTheme::Latte => egui::Color32::from_rgb(230, 233, 239),  // Mantle
            GuiTheme::HighContrast => egui::Color32::from_rgb(18, 18, 18),
        }
    }

    pub fn card_bg(&self, completed: bool) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => {
                if completed {
                    egui::Color32::from_rgb(20, 20, 30)
                } else {
                    egui::Color32::from_rgb(30, 30, 46)
                }
            }
            GuiTheme::Latte => {
                if completed {
                    egui::Color32::from_rgb(235, 237, 242)
                } else {
                    egui::Color32::from_rgb(255, 255, 255)
                }
            }
            GuiTheme::HighContrast => {
                if completed {
                    egui::Color32::from_rgb(15, 15, 15)
                } else {
                    egui::Color32::from_rgb(24, 24, 24)
                }
            }
        }
    }

    pub fn card_border(&self, completed: bool) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => {
                if completed {
                    egui::Color32::from_rgb(49, 50, 68)
                } else {
                    egui::Color32::from_rgb(69, 71, 90)
                }
            }
            GuiTheme::Latte => {
                if completed {
                    egui::Color32::from_rgb(204, 208, 218)
                } else {
                    egui::Color32::from_rgb(188, 192, 204)
                }
            }
            GuiTheme::HighContrast => {
                if completed {
                    egui::Color32::from_rgb(100, 100, 100)
                } else {
                    egui::Color32::from_rgb(255, 255, 255)
                }
            }
        }
    }

    pub fn primary_text(&self) -> egui::Color32 {
        match self {
            GuiTheme::Macchiato => egui::Color32::from_rgb(205, 214, 244),
            GuiTheme::Latte => egui::Color32::from_rgb(76, 79, 105),
            GuiTheme::HighContrast => egui::Color32::from_rgb(255, 255, 255),
        }
    }
}
