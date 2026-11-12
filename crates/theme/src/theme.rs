#![deny(missing_docs)]

//! # Theme
//!
//! This crate provides the theme system for Zed.
//!
//! ## Overview
//!
//! A theme is a collection of colors used to build a consistent appearance for UI components across the application.

mod buffer_line_height;
mod color_space;
mod default_colors;
mod fallback_themes;
mod font_family_cache;
mod icon_theme;
mod icon_theme_schema;
mod registry;
mod scale;
mod schema;
mod styles;
mod theme_settings_provider;
mod ui_density;

use std::sync::Arc;

use collections::HashMap;
use gpui::BorrowAppContext;
use gpui::Global;
use gpui::{
    App, AssetSource, Hsla, Pixels, SharedString, Styled, Tiling, WindowAppearance,
    WindowBackgroundAppearance, WindowId, px,
};
use serde::Deserialize;

pub use crate::buffer_line_height::*;
pub use crate::color_space::*;
pub use crate::default_colors::*;
pub use crate::fallback_themes::{apply_status_color_defaults, apply_theme_color_defaults};
pub use crate::font_family_cache::*;
pub use crate::icon_theme::*;
pub use crate::icon_theme_schema::*;
pub use crate::registry::*;
pub use crate::scale::*;
pub use crate::schema::*;
pub use crate::styles::*;
pub use crate::theme_settings_provider::*;
pub use crate::ui_density::*;

/// The name of the default dark theme.
pub const DEFAULT_DARK_THEME: &str = "One Dark";

/// Defines window border radius for platforms that use client side decorations.
pub const CLIENT_SIDE_DECORATION_ROUNDING: Pixels = px(10.0);
/// Defines window shadow size for platforms that use client side decorations.
pub const CLIENT_SIDE_DECORATION_SHADOW: Pixels = px(10.0);
/// Defines window border width for platforms that use client side decorations.
pub const CLIENT_SIDE_DECORATION_BORDER: Pixels = px(1.0);

/// Styling helpers for elements that follow client-side window decorations.
pub trait ClientDecorationsExt: Styled {
    /// Rounds each corner whose two adjacent edges are both untiled.
    fn rounded_client_corners(mut self, tiling: Tiling) -> Self {
        if !tiling.top && !tiling.left {
            self = self.rounded_tl(CLIENT_SIDE_DECORATION_ROUNDING);
        }
        if !tiling.top && !tiling.right {
            self = self.rounded_tr(CLIENT_SIDE_DECORATION_ROUNDING);
        }
        if !tiling.bottom && !tiling.left {
            self = self.rounded_bl(CLIENT_SIDE_DECORATION_ROUNDING);
        }
        if !tiling.bottom && !tiling.right {
            self = self.rounded_br(CLIENT_SIDE_DECORATION_ROUNDING);
        }
        self
    }
}

impl<T: Styled> ClientDecorationsExt for T {}

/// The appearance of the theme.
#[derive(Debug, PartialEq, Clone, Copy, Deserialize)]
pub enum Appearance {
    /// A light appearance.
    Light,
    /// A dark appearance.
    Dark,
}

impl Appearance {
    /// Returns whether the appearance is light.
    pub fn is_light(&self) -> bool {
        match self {
            Self::Light => true,
            Self::Dark => false,
        }
    }
}

impl From<WindowAppearance> for Appearance {
    fn from(value: WindowAppearance) -> Self {
        match value {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::Light,
        }
    }
}

/// Which themes should be loaded. This is used primarily for testing.
pub enum LoadThemes {
    /// Only load the base theme.
    ///
    /// No user themes will be loaded.
    JustBase,

    /// Load all of the built-in themes.
    All(Box<dyn AssetSource>),
}

/// Initialize the theme system with default themes.
///
/// This sets up the [`ThemeRegistry`], [`FontFamilyCache`], [`SystemAppearance`],
/// and [`GlobalTheme`] with the default dark theme. It does NOT load bundled
/// themes from JSON or integrate with settings — use `theme_settings::init` for that.
pub fn init(themes_to_load: LoadThemes, cx: &mut App) {
    SystemAppearance::init(cx);
    let assets = match themes_to_load {
        LoadThemes::JustBase => Box::new(()) as Box<dyn AssetSource>,
        LoadThemes::All(assets) => assets,
    };
    ThemeRegistry::set_global(assets, cx);
    FontFamilyCache::init_global(cx);

    let themes = ThemeRegistry::default_global(cx);
    let theme = themes.get(DEFAULT_DARK_THEME).unwrap_or_else(|_| {
        themes
            .list()
            .into_iter()
            .next()
            .map(|m| themes.get(&m.name).unwrap())
            .unwrap()
    });
    let icon_theme = themes.default_icon_theme().unwrap();
    cx.set_global(GlobalTheme::new(theme, icon_theme));
}

/// Implementing this trait allows accessing the active theme.
pub trait ActiveTheme {
    /// Returns the active theme.
    fn theme(&self) -> &Arc<Theme>;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        GlobalTheme::theme(self)
    }
}

/// The appearance of the system.
#[derive(Debug, Clone, Copy)]
pub struct SystemAppearance(pub Appearance);

impl std::ops::Deref for SystemAppearance {
    type Target = Appearance;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for SystemAppearance {
    fn default() -> Self {
        Self(Appearance::Dark)
    }
}

#[derive(Default)]
struct GlobalSystemAppearance(SystemAppearance);

impl std::ops::DerefMut for GlobalSystemAppearance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for GlobalSystemAppearance {
    type Target = SystemAppearance;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Global for GlobalSystemAppearance {}

impl SystemAppearance {
    /// Initializes the [`SystemAppearance`] for the application.
    pub fn init(cx: &mut App) {
        *cx.default_global::<GlobalSystemAppearance>() =
            GlobalSystemAppearance(SystemAppearance(cx.window_appearance().into()));
    }

    /// Returns the global [`SystemAppearance`].
    pub fn global(cx: &App) -> Self {
        cx.global::<GlobalSystemAppearance>().0
    }

    /// Returns a mutable reference to the global [`SystemAppearance`].
    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<GlobalSystemAppearance>()
    }
}

/// A theme family is a grouping of themes under a single name.
///
/// For example, the "One" theme family contains the "One Light" and "One Dark" themes.
///
/// It can also be used to package themes with many variants.
///
/// For example, the "Atelier" theme family contains "Cave", "Dune", "Estuary", "Forest", "Heath", etc.
pub struct ThemeFamily {
    /// The unique identifier for the theme family.
    pub id: String,
    /// The name of the theme family. This will be displayed in the UI, such as when adding or removing a theme family.
    pub name: SharedString,
    /// The author of the theme family.
    pub author: SharedString,
    /// The [Theme]s in the family.
    pub themes: Vec<Theme>,
    /// The color scales used by the themes in the family.
    /// Note: This will be removed in the future.
    pub scales: ColorScales,
}

/// A theme is the primary mechanism for defining the appearance of the UI.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// The unique identifier for the theme.
    pub id: String,
    /// The name of the theme.
    pub name: SharedString,
    /// The appearance of the theme (light or dark).
    pub appearance: Appearance,
    /// The colors and other styles for the theme.
    pub styles: ThemeStyles,
}

impl Theme {
    /// Returns the [`SystemColors`] for the theme.
    #[inline(always)]
    pub fn system(&self) -> &SystemColors {
        &self.styles.system
    }

    /// Returns the [`AccentColors`] for the theme.
    #[inline(always)]
    pub fn accents(&self) -> &AccentColors {
        &self.styles.accents
    }

    /// Returns the [`PlayerColors`] for the theme.
    #[inline(always)]
    pub fn players(&self) -> &PlayerColors {
        &self.styles.player
    }

    /// Returns the [`ThemeColors`] for the theme.
    #[inline(always)]
    pub fn colors(&self) -> &ThemeColors {
        &self.styles.colors
    }

    /// Returns the [`SyntaxTheme`] for the theme.
    #[inline(always)]
    pub fn syntax(&self) -> &Arc<SyntaxTheme> {
        &self.styles.syntax
    }

    /// Returns the [`StatusColors`] for the theme.
    #[inline(always)]
    pub fn status(&self) -> &StatusColors {
        &self.styles.status
    }

    /// Returns the [`Appearance`] for the theme.
    #[inline(always)]
    pub fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// Returns the [`WindowBackgroundAppearance`] for the theme.
    #[inline(always)]
    pub fn window_background_appearance(&self) -> WindowBackgroundAppearance {
        self.styles.window_background_appearance
    }

    /// Darkens the color by reducing its lightness.
    /// The resulting lightness is clamped to ensure it doesn't go below 0.0.
    ///
    /// The first value darkens light appearance mode, the second darkens appearance dark mode.
    ///
    /// Note: This is a tentative solution and may be replaced with a more robust color system.
    pub fn darken(&self, color: Hsla, light_amount: f32, dark_amount: f32) -> Hsla {
        let amount = match self.appearance {
            Appearance::Light => light_amount,
            Appearance::Dark => dark_amount,
        };
        let mut hsla = color;
        hsla.l = (hsla.l - amount).max(0.0);
        hsla
    }
}

/// Deserializes an icon theme from the given bytes.
pub fn deserialize_icon_theme(bytes: &[u8]) -> anyhow::Result<IconThemeFamilyContent> {
    let icon_theme_family: IconThemeFamilyContent = serde_json_lenient::from_slice(bytes)?;

    Ok(icon_theme_family)
}

/// The active theme.
pub struct GlobalTheme {
    theme: Arc<Theme>,
    icon_theme: Arc<IconTheme>,
    window_themes: HashMap<WindowId, Arc<Theme>>,
}
impl Global for GlobalTheme {}

impl GlobalTheme {
    /// Creates a new [`GlobalTheme`] with the given theme and icon theme.
    pub fn new(theme: Arc<Theme>, icon_theme: Arc<IconTheme>) -> Self {
        Self {
            theme,
            icon_theme,
            window_themes: HashMap::default(),
        }
    }

    /// Updates the active theme.
    pub fn update_theme(cx: &mut App, theme: Arc<Theme>) {
        cx.update_global::<Self, _>(|this, _| this.theme = theme);
    }

    /// Updates the active icon theme.
    pub fn update_icon_theme(cx: &mut App, icon_theme: Arc<IconTheme>) {
        cx.update_global::<Self, _>(|this, _| this.icon_theme = icon_theme);
    }

    /// Sets or clears the theme that replaces the active theme inside one window.
    ///
    /// A window theme must keep the syntax styles of the active theme: syntax
    /// highlight ids are resolved against the active theme for all windows.
    pub fn set_window_theme(cx: &mut App, window_id: WindowId, theme: Option<Arc<Theme>>) {
        // Clearing a window that has no theme must not notify observers, since
        // every workspace does that when it opens.
        if theme.is_none() && !cx.global::<Self>().window_themes.contains_key(&window_id) {
            return;
        }
        cx.update_global::<Self, _>(|this, _| match theme {
            Some(theme) => {
                this.window_themes.insert(window_id, theme);
            }
            None => {
                this.window_themes.remove(&window_id);
            }
        });
    }

    /// Returns the active theme: the theme of the window that is being updated
    /// when it has one, and the theme shared by all windows otherwise.
    pub fn theme(cx: &App) -> &Arc<Theme> {
        let this = cx.global::<Self>();
        if this.window_themes.is_empty() {
            return &this.theme;
        }
        cx.updating_window_id()
            .and_then(|window_id| this.window_themes.get(&window_id))
            .unwrap_or(&this.theme)
    }

    /// Returns the theme shared by all windows, ignoring window themes.
    pub fn shared_theme(cx: &App) -> &Arc<Theme> {
        &cx.global::<Self>().theme
    }

    /// Returns the active icon theme.
    pub fn icon_theme(cx: &App) -> &Arc<IconTheme> {
        &cx.global::<Self>().icon_theme
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{EmptyView, TestAppContext};

    use super::*;

    #[gpui::test]
    fn test_window_theme_applies_only_inside_its_window(cx: &mut TestAppContext) {
        cx.update(|cx| init(LoadThemes::JustBase, cx));
        let window = cx.add_window(|_, _| EmptyView);
        let other_window = cx.add_window(|_, _| EmptyView);
        let window_id = window.window_id();

        let shared_theme = cx.update(|cx| GlobalTheme::shared_theme(cx).clone());
        let mut window_theme = (*shared_theme).clone();
        window_theme.styles.colors.title_bar_background = gpui::red();
        let window_theme = Arc::new(window_theme);
        assert_ne!(
            shared_theme.colors().title_bar_background,
            window_theme.colors().title_bar_background
        );

        let notification_count = Rc::new(Cell::new(0));
        let _subscription = cx.update(|cx| {
            cx.observe_global::<GlobalTheme>({
                let notification_count = notification_count.clone();
                move |_| notification_count.set(notification_count.get() + 1)
            })
        });

        cx.update(|cx| GlobalTheme::set_window_theme(cx, window_id, None));
        assert_eq!(
            notification_count.get(),
            0,
            "clearing a window without a theme must not notify observers"
        );

        cx.update(|cx| GlobalTheme::set_window_theme(cx, window_id, Some(window_theme.clone())));
        assert_eq!(notification_count.get(), 1);

        let theme_in_window = window.update(cx, |_, _, cx| cx.theme().clone()).unwrap();
        assert!(Arc::ptr_eq(&theme_in_window, &window_theme));

        let theme_in_other_window = other_window
            .update(cx, |_, _, cx| cx.theme().clone())
            .unwrap();
        assert!(Arc::ptr_eq(&theme_in_other_window, &shared_theme));

        cx.update(|cx| {
            assert!(Arc::ptr_eq(cx.theme(), &shared_theme));
            assert!(Arc::ptr_eq(GlobalTheme::shared_theme(cx), &shared_theme));
        });

        let shared_theme_in_window = window
            .update(cx, |_, _, cx| GlobalTheme::shared_theme(cx).clone())
            .unwrap();
        assert!(Arc::ptr_eq(&shared_theme_in_window, &shared_theme));

        cx.update(|cx| GlobalTheme::set_window_theme(cx, window_id, None));
        assert_eq!(notification_count.get(), 2);

        let theme_in_window = window.update(cx, |_, _, cx| cx.theme().clone()).unwrap();
        assert!(Arc::ptr_eq(&theme_in_window, &shared_theme));

        cx.update(|cx| GlobalTheme::set_window_theme(cx, window_id, None));
        assert_eq!(
            notification_count.get(),
            2,
            "clearing a window whose theme was already cleared must not notify observers"
        );
    }
}
