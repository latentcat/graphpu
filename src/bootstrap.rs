use eframe::AppCreator;

use crate::{MainApp, APP_VERSION};

pub struct Config {
    native_options: eframe::NativeOptions,
    app_creator: AppCreator,
    app_name: String,
}

pub struct ConfigBuilder {
    native_options: eframe::NativeOptions,
    app_creator: AppCreator,
    app_name: String,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        let native_options = eframe::NativeOptions {
            initial_window_size: Some(egui::Vec2::new(1200.0, 720.0)),
            min_window_size: Some(egui::Vec2::new(960.0, 640.0)),
            drag_and_drop_support: true,
            renderer: eframe::Renderer::Wgpu,
            follow_system_theme: false,
            default_theme: eframe::Theme::Dark,
            ..Default::default()
        };

        Self {
            native_options,
            app_creator: Box::new(|cc| Box::new(MainApp::new(cc))),
            app_name: format!("GraphPU - Dev Demo - {}", APP_VERSION),
        }
    }
}

impl ConfigBuilder {
    pub fn native_options(mut self, native_options: eframe::NativeOptions) -> Self {
        self.native_options = native_options;
        self
    }

    pub fn app_creator(mut self, app_creator: AppCreator) -> Self {
        self.app_creator = app_creator;
        self
    }

    pub fn app_name(mut self, app_name: String) -> Self {
        self.app_name = app_name;
        self
    }

    pub fn build(self) -> Config {
        Config {
            native_options: self.native_options,
            app_creator: self.app_creator,
            app_name: self.app_name,
        }
    }
}

pub fn start_frame(config: Config) {
    eframe::run_native(
        &config.app_name,
        config.native_options,
        config.app_creator,
    );
}
