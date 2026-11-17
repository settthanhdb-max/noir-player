//! Runtime descriptors for settings, used to generate the settings UI.

/// A mutable borrow of one concrete setting value.
#[derive(Debug)]
pub enum SettingValue<'a> {
    Bool(&'a mut bool),
    U32(&'a mut u32),
    I32(&'a mut i32),
    F32(&'a mut f32),
    Text(&'a mut String),
    /// Enum-like selection among fixed options, stored as the selected index
    /// into [`SettingKind::options`].
    Choice(&'a mut usize),
}

impl SettingValue<'_> {
    /// Current value formatted for display/search.
    pub fn display(&self) -> String {
        match self {
            SettingValue::Bool(v) => v.to_string(),
            SettingValue::U32(v) => v.to_string(),
            SettingValue::I32(v) => v.to_string(),
            SettingValue::F32(v) => format!("{v:.2}"),
            SettingValue::Text(v) => v.to_string(),
            SettingValue::Choice(v) => v.to_string(),
        }
    }

    /// Whether the value differs from the supplied default representation.
    pub fn is_default(&self, default: &str) -> bool {
        self.display() == default
    }
}

/// How the settings UI should render (and constrain) the value.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingKind {
    Toggle,
    Number {
        min: f64,
        max: f64,
        step: f64,
    },
    Slider {
        min: f64,
        max: f64,
        step: f64,
        /// Optional unit suffix, e.g. "s", "%".
        unit: &'static str,
    },
    Dropdown {
        options: Vec<&'static str>,
    },
    Text,
    /// Opens a directory picker.
    Directory,
    /// Opens a file picker.
    File {
        filters: Vec<(&'static str, &'static str)>,
    },
    /// Not directly editable; informational (capability reports etc.).
    ReadOnly,
}

impl SettingKind {
    pub fn default_display(&self) -> String {
        match self {
            SettingKind::Toggle => "false".into(),
            SettingKind::Number { .. } | SettingKind::Slider { .. } => "0".into(),
            SettingKind::Dropdown { options } => {
                options.first().map(|s| s.to_string()).unwrap_or_default()
            }
            SettingKind::Text | SettingKind::Directory | SettingKind::File { .. } => String::new(),
            SettingKind::ReadOnly => String::new(),
        }
    }
}

/// One renderable settings row.
pub struct SettingEntry<'a> {
    /// Dotted key path, e.g. `playback.seek_backward_s`.
    pub key: &'static str,
    /// Translation key for the row label.
    pub label: &'a str,
    /// Translation key for the tooltip / description.
    pub description: &'a str,
    pub kind: SettingKind,
    pub value: SettingValue<'a>,
    /// Only shown when advanced mode is enabled.
    pub advanced: bool,
    /// Applying requires an engine/window restart.
    pub restart_required: bool,
    /// Backend capability gate, e.g. `"hwdec"`. Empty = always available.
    pub requires_capability: &'static str,
}

impl<'a> SettingEntry<'a> {
    pub fn new(
        key: &'static str,
        label: &'a str,
        description: &'a str,
        kind: SettingKind,
        value: SettingValue<'a>,
    ) -> Self {
        Self {
            key,
            label,
            description,
            kind,
            value,
            advanced: false,
            restart_required: false,
            requires_capability: "",
        }
    }

    pub fn advanced(mut self) -> Self {
        self.advanced = true;
        self
    }

    pub fn restart(mut self) -> Self {
        self.restart_required = true;
        self
    }

    pub fn requires(mut self, capability: &'static str) -> Self {
        self.requires_capability = capability;
        self
    }
}
