//! Blender's default dark theme colors, organized by space type.
//!
//! All values are taken from Blender 4.x's built-in dark theme defaults.
//! Colors are `[R, G, B, A]` in sRGB u8 space.

use super::widget_base::WidgetColors;

/// Complete Blender theme structure.
#[derive(Debug, Clone)]
pub struct Theme {
    pub general: GeneralColors,
    pub view3d: SpaceColors,
    pub outliner: SpaceColors,
    pub properties: SpaceColors,
    pub timeline: SpaceColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            general: GeneralColors::default(),
            view3d: SpaceColors {
                back: view3d::BACK,
                header: view3d::HEADER,
                panel: view3d::PANEL,
                text: general::TEXT,
                text_hi: general::TEXT_HI,
            },
            outliner: SpaceColors {
                back: outliner::BACK,
                header: outliner::HEADER,
                panel: outliner::PANEL,
                text: general::TEXT,
                text_hi: general::TEXT_HI,
            },
            properties: SpaceColors {
                back: properties::BACK,
                header: properties::HEADER,
                panel: properties::PANEL,
                text: general::TEXT,
                text_hi: general::TEXT_HI,
            },
            timeline: SpaceColors {
                back: timeline::BACK,
                header: timeline::HEADER,
                panel: timeline::PANEL,
                text: general::TEXT,
                text_hi: general::TEXT_HI,
            },
        }
    }
}

/// Colors common to all editor spaces.
#[derive(Debug, Clone)]
pub struct GeneralColors {
    pub back: [u8; 4],
    pub header: [u8; 4],
    pub panel: [u8; 4],
    pub text: [u8; 4],
    pub text_hi: [u8; 4],
    pub title: [u8; 4],
    pub wcol_regular: WidgetColors,
    pub wcol_tool: WidgetColors,
    pub wcol_toolbar_item: WidgetColors,
    pub wcol_text: WidgetColors,
    pub wcol_radio: WidgetColors,
    pub wcol_option: WidgetColors,
    pub wcol_toggle: WidgetColors,
    pub wcol_num: WidgetColors,
    pub wcol_numslider: WidgetColors,
    pub wcol_menu: WidgetColors,
    pub wcol_pulldown: WidgetColors,
    pub wcol_menu_back: WidgetColors,
    pub wcol_menu_item: WidgetColors,
    pub wcol_box: WidgetColors,
    pub wcol_scroll: WidgetColors,
    pub wcol_tab: WidgetColors,
}

impl Default for GeneralColors {
    fn default() -> Self {
        Self {
            back: general::BACK,
            header: general::HEADER,
            panel: general::PANEL,
            text: general::TEXT,
            text_hi: general::TEXT_HI,
            title: general::TITLE,
            wcol_regular: wcol::REGULAR,
            wcol_tool: wcol::TOOL,
            wcol_toolbar_item: wcol::TOOLBAR_ITEM,
            wcol_text: wcol::TEXT,
            wcol_radio: wcol::RADIO,
            wcol_option: wcol::OPTION,
            wcol_toggle: wcol::TOGGLE,
            wcol_num: wcol::NUM,
            wcol_numslider: wcol::NUMSLIDER,
            wcol_menu: wcol::MENU,
            wcol_pulldown: wcol::PULLDOWN,
            wcol_menu_back: wcol::MENU_BACK,
            wcol_menu_item: wcol::MENU_ITEM,
            wcol_box: wcol::BOX,
            wcol_scroll: wcol::SCROLL,
            wcol_tab: wcol::TAB,
        }
    }
}

/// Per-space color configuration.
#[derive(Debug, Clone)]
pub struct SpaceColors {
    pub back: [u8; 4],
    pub header: [u8; 4],
    pub panel: [u8; 4],
    pub text: [u8; 4],
    pub text_hi: [u8; 4],
}

/// Panel-specific colors used by [`super::panel`].
#[derive(Debug, Clone, Copy)]
pub struct PanelColors {
    pub header_back: [u8; 4],
    pub body_back: [u8; 4],
    pub text: [u8; 4],
    pub triangle: [u8; 4],
}

impl Default for PanelColors {
    fn default() -> Self {
        Self {
            header_back: [72, 72, 72, 255],
            body_back: general::PANEL,
            text: general::TITLE,
            triangle: [200, 200, 200, 255],
        }
    }
}

// ---------------------------------------------------------------------------
// Constant color modules
// ---------------------------------------------------------------------------

/// General / global theme constants.
pub mod general {
    pub const BACK: [u8; 4] = [61, 61, 61, 255];
    pub const HEADER: [u8; 4] = [48, 48, 48, 179];
    pub const HEADER_TEXT: [u8; 4] = [238, 238, 238, 255];
    pub const PANEL: [u8; 4] = [61, 61, 61, 255];
    pub const SUB_PANEL: [u8; 4] = [61, 61, 61, 255];
    pub const TEXT: [u8; 4] = [230, 230, 230, 255];
    pub const TEXT_HI: [u8; 4] = [255, 255, 255, 255];
    pub const TITLE: [u8; 4] = [238, 238, 238, 255];
    pub const SEPARATOR: [u8; 4] = [36, 36, 36, 255];
    pub const TAB_ACTIVE: [u8; 4] = [76, 76, 76, 255];
    pub const TAB_INACTIVE: [u8; 4] = [46, 46, 46, 255];
    pub const TAB_OUTLINE: [u8; 4] = [36, 36, 36, 255];
    pub const BUTTON: [u8; 4] = [76, 76, 76, 255];
    pub const BUTTON_TITLE: [u8; 4] = [255, 255, 255, 255];
    pub const TOOLTIP_BACK: [u8; 4] = [25, 25, 25, 230];
    pub const TOOLTIP_TEXT: [u8; 4] = [230, 230, 230, 255];
    pub const ICON_ACTIVE: [u8; 4] = [255, 255, 255, 255];
    pub const ICON_INACTIVE: [u8; 4] = [200, 200, 200, 200];
    pub const SHADOW: [u8; 4] = [0, 0, 0, 75];
    pub const SHADOW_OFFSET: f32 = 4.0;
}

/// Widget color presets matching Blender's `bTheme.tui.wcol_*` structs.
pub mod wcol {
    use super::super::widget_base::WidgetColors;

    pub const REGULAR: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [114, 114, 114, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 15,
        shadedown: -15,
        roundness: 0.2,
    };

    pub const TOOL: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [114, 114, 114, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 15,
        shadedown: -15,
        roundness: 0.2,
    };

    pub const TOOLBAR_ITEM: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [100, 100, 100, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 10,
        shadedown: -10,
        roundness: 0.3,
    };

    pub const TEXT: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [25, 25, 25, 255],
        inner_sel: [86, 128, 194, 255],
        item: [153, 153, 153, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 5,
        shadedown: 0,
        roundness: 0.2,
    };

    pub const RADIO: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [86, 86, 86, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 15,
        shadedown: -15,
        roundness: 0.2,
    };

    pub const OPTION: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [86, 86, 86, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 15,
        shadedown: -15,
        roundness: 0.2,
    };

    pub const TOGGLE: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [86, 86, 86, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 15,
        shadedown: -15,
        roundness: 0.2,
    };

    pub const NUM: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [114, 114, 114, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 15,
        shadedown: -15,
        roundness: 0.2,
    };

    pub const NUMSLIDER: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [114, 114, 114, 255],
        inner_sel: [86, 128, 194, 255],
        item: [176, 176, 176, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 15,
        shadedown: -15,
        roundness: 0.2,
    };

    pub const MENU: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [67, 67, 67, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 15,
        shadedown: -15,
        roundness: 0.2,
    };

    pub const PULLDOWN: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [67, 67, 67, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 25,
        shadedown: -20,
        roundness: 0.2,
    };

    pub const MENU_BACK: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [45, 45, 45, 230],
        inner_sel: [45, 45, 45, 230],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: false,
        shadetop: 0,
        shadedown: 0,
        roundness: 0.3,
    };

    pub const MENU_ITEM: WidgetColors = WidgetColors {
        outline: [0, 0, 0, 0],
        inner: [0, 0, 0, 0],
        inner_sel: [86, 128, 194, 255],
        item: [172, 172, 172, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 38,
        shadedown: 0,
        roundness: 0.2,
    };

    pub const BOX: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [86, 86, 86, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: false,
        shadetop: 0,
        shadedown: 0,
        roundness: 0.2,
    };

    pub const SCROLL: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 180],
        inner: [80, 80, 80, 180],
        inner_sel: [100, 100, 100, 180],
        item: [128, 128, 128, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 5,
        shadedown: -5,
        roundness: 0.5,
    };

    pub const TAB: WidgetColors = WidgetColors {
        outline: [50, 50, 50, 255],
        inner: [76, 76, 76, 255],
        inner_sel: [86, 128, 194, 255],
        item: [230, 230, 230, 255],
        text: [230, 230, 230, 255],
        text_sel: [255, 255, 255, 255],
        shaded: true,
        shadetop: 5,
        shadedown: -5,
        roundness: 0.2,
    };
}

/// 3D Viewport space colors.
pub mod view3d {
    pub const BACK: [u8; 4] = [57, 57, 57, 255];
    pub const HEADER: [u8; 4] = [48, 48, 48, 179];
    pub const PANEL: [u8; 4] = [61, 61, 61, 255];
    pub const GRID: [u8; 4] = [76, 76, 76, 255];
    pub const WIRE: [u8; 4] = [35, 35, 35, 255];
    pub const WIRE_EDIT: [u8; 4] = [35, 35, 35, 255];
    pub const OBJECT_SELECTED: [u8; 4] = [237, 143, 46, 255];
    pub const OBJECT_ACTIVE: [u8; 4] = [255, 170, 64, 255];
    pub const TRANSFORM: [u8; 4] = [255, 255, 255, 255];
    pub const VERTEX: [u8; 4] = [0, 0, 0, 255];
    pub const VERTEX_SELECT: [u8; 4] = [255, 133, 51, 255];
    pub const EDGE_SELECT: [u8; 4] = [255, 160, 64, 255];
    pub const FACE_SELECT: [u8; 4] = [255, 133, 51, 76];
    pub const FACE_DOT: [u8; 4] = [255, 133, 51, 255];
    pub const NORMAL: [u8; 4] = [34, 221, 221, 255];
    pub const CURSOR: [u8; 4] = [255, 0, 0, 255];
    pub const GIZMO_X: [u8; 4] = [237, 46, 72, 255];
    pub const GIZMO_Y: [u8; 4] = [103, 196, 56, 255];
    pub const GIZMO_Z: [u8; 4] = [58, 116, 237, 255];
}

/// Outliner space colors.
pub mod outliner {
    pub const BACK: [u8; 4] = [40, 40, 40, 255];
    pub const HEADER: [u8; 4] = [40, 40, 40, 179];
    pub const PANEL: [u8; 4] = [40, 40, 40, 255];
    pub const SELECTED: [u8; 4] = [76, 97, 128, 255];
    pub const ACTIVE: [u8; 4] = [56, 73, 97, 255];
    pub const ROW_ALT: [u8; 4] = [36, 36, 36, 255];
    pub const MATCH: [u8; 4] = [45, 70, 100, 255];
}

/// Properties space colors.
pub mod properties {
    pub const BACK: [u8; 4] = [48, 48, 48, 255];
    pub const HEADER: [u8; 4] = [48, 48, 48, 179];
    pub const PANEL: [u8; 4] = [61, 61, 61, 255];
    pub const SUB_PANEL: [u8; 4] = [61, 61, 61, 255];
    pub const TAB_ACTIVE: [u8; 4] = [76, 76, 76, 255];
    pub const TAB_INACTIVE: [u8; 4] = [46, 46, 46, 255];
}

/// Timeline / Dopesheet space colors.
pub mod timeline {
    pub const BACK: [u8; 4] = [48, 48, 48, 255];
    pub const HEADER: [u8; 4] = [48, 48, 48, 179];
    pub const PANEL: [u8; 4] = [61, 61, 61, 255];
    pub const FRAME_CURRENT: [u8; 4] = [86, 128, 194, 255];
    pub const SCRUB_BACK: [u8; 4] = [38, 38, 38, 255];
    pub const KEYFRAME: [u8; 4] = [232, 232, 14, 255];
    pub const KEYFRAME_SELECTED: [u8; 4] = [189, 137, 57, 255];
    pub const GRID: [u8; 4] = [76, 76, 76, 255];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_default_constructs() {
        let _t = Theme::default();
    }

    #[test]
    fn general_colors_opaque() {
        assert_eq!(general::BACK[3], 255);
        assert_eq!(general::TEXT[3], 255);
    }

    #[test]
    fn panel_colors_default() {
        let pc = PanelColors::default();
        assert_eq!(pc.body_back, general::PANEL);
    }

    #[test]
    fn wcol_regular_has_shading() {
        assert!(wcol::REGULAR.shaded);
        assert!(wcol::REGULAR.shadetop > 0);
        assert!(wcol::REGULAR.shadedown < 0);
    }
}
