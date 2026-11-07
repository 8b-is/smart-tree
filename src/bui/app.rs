//! BUI Application for Smart Tree
//!
//! Main application entry point for the BUI-based GUI.

#[cfg(feature = "bui")]
use blenderui::prelude::*;

#[cfg(feature = "bui")]
use crate::bui::components::{ChatPanel, CommandPalette, StatusBar, TreeView};

#[cfg(feature = "bui")]
pub struct BuiApp {
    pub window: Option<Window>,
    pub tree_view: TreeView,
    pub chat_panel: ChatPanel,
    pub status_bar: StatusBar,
    pub command_palette: CommandPalette,
    pub theme: BuiTheme,
}

#[cfg(feature = "bui")]
impl BuiApp {
    pub fn new(theme: BuiTheme) -> Self {
        Self {
            window: None,
            tree_view: TreeView::new(),
            chat_panel: ChatPanel::new(),
            status_bar: StatusBar::new(),
            command_palette: CommandPalette::new(),
            theme,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let app = Application::new()?;

        self.window = Some(Window::new(WindowOptions {
            title: "Smart Tree - BUI".into(),
            width: 1200,
            height: 800,
            theme: self.theme.to_blenderui_theme(),
            ..Default::default()
        })?);

        // Build UI
        self.build_ui()?;

        app.run()
    }

    fn build_ui(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let window = self.window.as_mut().unwrap();

        window.set_content(|cx| {
            VStack::new()
                .child(self.tree_view.render(cx))
                .child(
                    Split::horizontal()
                        .left(self.chat_panel.render(cx))
                        .right(self.command_palette.render(cx)),
                )
                .child(self.status_bar.render(cx))
                .style(|s| s.flex(1).padding(10))
        })?;

        Ok(())
    }
}
