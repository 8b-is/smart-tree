//! BUI Components for Smart Tree
//!
//! Reusable Blender-style UI components.

#[cfg(feature = "bui")]
use blenderui::prelude::*;

#[cfg(feature = "bui")]
pub struct TreeView {
    pub root_path: Option<std::path::PathBuf>,
    pub expanded_nodes: std::collections::HashSet<std::path::PathBuf>,
    pub filter: String,
}

#[cfg(feature = "bui")]
impl TreeView {
    pub fn new() -> Self {
        Self {
            root_path: None,
            expanded_nodes: std::collections::HashSet::new(),
            filter: String::new(),
        }
    }

    pub fn set_root(&mut self, path: std::path::PathBuf) {
        self.root_path = Some(path);
    }

    #[cfg(feature = "bui")]
    pub fn render(&self, cx: &mut Context) -> impl Element {
        VStack::new()
            .child(
                TextInput::new()
                    .placeholder("Filter files...")
                    .on_change(|s| {
                        // Filter logic
                    })
                    .style(|s| s.width(300).margin_bottom(10)),
            )
            .child(
                Scroll::vertical()
                    .child(
                        Tree::new()
                            .root(".")
                            .expanded(self.expanded_nodes.clone())
                            .filter(&self.filter),
                    )
                    .style(|s| s.flex(1)),
            )
            .style(|s| s.flex(1).padding(10))
    }
}

#[cfg(feature = "bui")]
pub struct ChatPanel {
    pub messages: Vec<ChatMessage>,
    pub input: String,
}

#[cfg(feature = "bui")]
#[derive(Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "bui")]
impl ChatPanel {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
        }
    }

    pub fn add_message(&mut self, role: String, content: String) {
        self.messages.push(ChatMessage {
            role,
            content,
            timestamp: chrono::Utc::now(),
        });
    }

    #[cfg(feature = "bui")]
    pub fn render(&self, cx: &mut Context) -> impl Element {
        VStack::new()
            .child(
                Scroll::vertical().child(VStack::new().children(self.messages.iter().map(|msg| {
                    HStack::new()
                        .child(Text::new(&msg.role).style(|s| {
                            s.bold().color(if msg.role == "user" {
                                Color::BLUE
                            } else {
                                Color::GREEN
                            })
                        }))
                        .child(Text::new(&msg.content))
                        .style(|s| s.padding(5).margin_bottom(5))
                }))),
            )
            .child(
                HStack::new()
                    .child(
                        TextInput::new()
                            .placeholder("Ask Smart Tree...")
                            .on_submit(|s| {
                                // Send message
                            }),
                    )
                    .child(Button::new("Send").on_click(|| {
                        // Send
                    }))
                    .style(|s| s.padding(10)),
            )
            .style(|s| s.flex(1).padding(10).background(Color::from_hex("#1e1e1e")))
    }
}

#[cfg(feature = "bui")]
pub struct StatusBar {
    pub status: String,
    pub mode: String,
    pub credits: f64,
}

#[cfg(feature = "bui")]
impl StatusBar {
    pub fn new() -> Self {
        Self {
            status: "Ready".into(),
            mode: "scan".into(),
            credits: 0.0,
        }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    #[cfg(feature = "bui")]
    pub fn render(&self, cx: &mut Context) -> impl Element {
        HStack::new()
            .child(Text::new(&self.status).style(|s| s.color(Color::GREEN)))
            .child(Separator::vertical())
            .child(Text::new(&format!("Mode: {}", self.mode)).style(|s| s.color(Color::BLUE)))
            .child(Separator::vertical())
            .child(
                Text::new(&format!("Credits: {:.2} FKN", self.credits))
                    .style(|s| s.color(Color::YELLOW)),
            )
            .style(|s| {
                s.padding(5..10)
                    .background(Color::from_hex("#1a1a1a"))
                    .border_top(1, Color::from_hex("#333"))
            })
    }
}

#[cfg(feature = "bui")]
pub struct CommandPalette {
    pub query: String,
    pub results: Vec<CommandResult>,
    pub visible: bool,
}

#[cfg(feature = "bui")]
#[derive(Clone)]
pub struct CommandResult {
    pub title: String,
    pub description: String,
    pub action: String,
}

#[cfg(feature = "bui")]
impl CommandPalette {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    #[cfg(feature = "bui")]
    pub fn render(&self, cx: &mut Context) -> impl Element {
        if !self.visible {
            return Empty::new().into_element();
        }

        VStack::new()
            .child(
                TextInput::new()
                    .placeholder("Command palette... (Ctrl+K)")
                    .on_change(|s| {
                        // Filter results
                    })
                    .style(|s| s.width(600).padding(10).font_size(16)),
            )
            .child(
                Scroll::vertical()
                    .child(VStack::new().children(self.results.iter().map(|r| {
                        HStack::new()
                            .child(Text::new(&r.title).style(|s| s.bold()))
                            .child(Text::new(&r.description).style(|s| s.color(Color::GRAY)))
                            .style(|s| s.padding(10).width(600))
                    })))
                    .style(|s| s.max_height(300)),
            )
            .style(|s| {
                s.padding(20)
                    .background(Color::from_hex("#1e1e1e"))
                    .border(1, Color::from_hex("#333"))
                    .rounded(8)
            })
            .style(|s| s.position(Position::Absolute).top(50).center_x())
    }
}
