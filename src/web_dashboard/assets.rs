//! Embedded static assets for the web dashboard

/// Main HTML page
pub const INDEX_HTML: &str = include_str!("assets/index.html");

/// Main stylesheet
pub const STYLE_CSS: &str = include_str!("assets/style.css");

/// Main application JavaScript
pub const APP_JS: &str = include_str!("assets/app.js");

/// xterm.js terminal emulator
pub const XTERM_JS: &str = include_str!("assets/xterm.min.js");

/// xterm.js stylesheet
pub const XTERM_CSS: &str = include_str!("assets/xterm.css");

/// xterm fit addon
pub const XTERM_FIT_JS: &str = include_str!("assets/xterm-addon-fit.min.js");

/// marked.js markdown renderer
pub const MARKED_JS: &str = include_str!("assets/marked.min.js");
