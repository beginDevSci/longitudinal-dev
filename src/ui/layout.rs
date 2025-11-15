use leptos::prelude::*;

// ============================================================================
// Layout Primitives
// ============================================================================

/// Column ratio for Split component
#[derive(Clone, Copy, Debug)]
pub enum Ratio {
    OneOne, // 1:1 (default, equal columns)
    TwoOne, // 2:1 (left column twice as wide)
    OneTwo, // 1:2 (right column twice as wide)
}

/// Two-column responsive grid (1 col mobile, 2+ cols desktop)
///
/// Uses static class literals for reliable Tailwind v4 content detection.
#[component]
pub fn Split<L, R>(
    left: L,
    right: R,
    #[prop(default = false)] reverse_on_md: bool,
    #[prop(default = Ratio::OneOne)] ratio: Ratio,
) -> impl IntoView
where
    L: IntoView + 'static,
    R: IntoView + 'static,
{
    // Use .into_any() to unify all match arm types
    match (ratio, reverse_on_md) {
        // 1:1 ratio (equal columns at md+)
        (Ratio::OneOne, false) => view! {
            <div class="grid grid-cols-1 gap-section section-padding md:grid-cols-2">
                <div>{left}</div>
                <div>{right}</div>
            </div>
        }
        .into_any(),

        (Ratio::OneOne, true) => view! {
            <div class="grid grid-cols-1 gap-section section-padding md:grid-cols-2">
                <div class="md:order-2">{left}</div>
                <div class="md:order-1">{right}</div>
            </div>
        }
        .into_any(),

        // 2:1 ratio (left column twice as wide)
        (Ratio::TwoOne, false) => view! {
            <div class="grid grid-cols-1 gap-section section-padding-xs md:grid-cols-3">
                <div class="md:col-span-2">{left}</div>
                <div class="md:col-span-1">{right}</div>
            </div>
        }
        .into_any(),

        (Ratio::TwoOne, true) => view! {
            <div class="grid grid-cols-1 gap-section section-padding md:grid-cols-3">
                <div class="md:col-span-2 md:order-2">{left}</div>
                <div class="md:col-span-1 md:order-1">{right}</div>
            </div>
        }
        .into_any(),

        // 1:2 ratio (right column twice as wide)
        (Ratio::OneTwo, false) => view! {
            <div class="grid grid-cols-1 gap-section section-padding md:grid-cols-3">
                <div class="md:col-span-1">{left}</div>
                <div class="md:col-span-2">{right}</div>
            </div>
        }
        .into_any(),

        (Ratio::OneTwo, true) => view! {
            <div class="grid grid-cols-1 gap-section section-padding md:grid-cols-3">
                <div class="md:col-span-1 md:order-2">{left}</div>
                <div class="md:col-span-2 md:order-1">{right}</div>
            </div>
        }
        .into_any(),
    }
}

/// Gap spacing values for Stack component (static classes only)
#[derive(Clone, Copy, Debug)]
pub enum Gap {
    G0,
    G1,
    G2,
    G3,
    G4,
    G6,
    G8,
}

impl Gap {
    /// Convert gap enum to static Tailwind class string
    pub const fn class(self) -> &'static str {
        match self {
            Gap::G0 => "gap-0",
            Gap::G1 => "gap-1",
            Gap::G2 => "gap-2",
            Gap::G3 => "gap-3",
            Gap::G4 => "gap-4",
            Gap::G6 => "gap-6",
            Gap::G8 => "gap-8",
        }
    }
}

/// Grid presets for sm/lg breakpoints (compile-time literal classes)
#[derive(Clone, Copy)]
pub enum GridSmLg {
    One,         // grid-cols-1
    OneTwo,      // grid-cols-1 sm:grid-cols-2
    OneTwoThree, // grid-cols-1 sm:grid-cols-2 lg:grid-cols-3   ← default
    OneTwoFour,  // grid-cols-1 sm:grid-cols-2 lg:grid-cols-4
}

impl GridSmLg {
    pub const fn classes(self) -> &'static str {
        match self {
            GridSmLg::One => "grid grid-cols-1 gap-component section-padding",
            GridSmLg::OneTwo => "grid grid-cols-1 sm:grid-cols-2 gap-component section-padding",
            GridSmLg::OneTwoThree => {
                "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-component section-padding"
            }
            GridSmLg::OneTwoFour => {
                "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-component section-padding"
            }
        }
    }
}

/// Grid presets for md breakpoint (compile-time literal classes)
#[derive(Clone, Copy)]
pub enum GridMd {
    One,      // grid-cols-1
    OneTwo,   // grid-cols-1 md:grid-cols-2   ← default
    TwoThree, // grid-cols-2 md:grid-cols-3
}

impl GridMd {
    pub const fn classes(self) -> &'static str {
        match self {
            GridMd::One => "grid grid-cols-1 gap-component section-padding-sm",
            GridMd::OneTwo => "grid grid-cols-1 md:grid-cols-2 gap-component section-padding-sm",
            GridMd::TwoThree => "grid grid-cols-2 md:grid-cols-3 gap-component section-padding-sm",
        }
    }
}

/// Vertical flex container with configurable gap
#[component]
pub fn Stack(children: Children, #[prop(default = Gap::G6)] gap: Gap) -> impl IntoView {
    view! {
        <div class={
        let gap_class = gap.class();
        match gap_class {
            "gap-0" => "flex flex-col gap-0",
            "gap-1" => "flex flex-col gap-1",
            "gap-2" => "flex flex-col gap-2",
            "gap-3" => "flex flex-col gap-3",
            "gap-4" => "flex flex-col gap-4",
            "gap-6" => "flex flex-col gap-6",
            "gap-8" => "flex flex-col gap-8",
            _ => "flex flex-col gap-6",
        }
    }>
            {children()}
        </div>
    }
}

/// Responsive card grid (1/2/3 cols)
#[component]
pub fn CardGrid(
    children: Children,
    #[prop(default = GridSmLg::OneTwoThree)] layout: GridSmLg,
) -> impl IntoView {
    view! {
        <div class=layout.classes()>
            {children()}
        </div>
    }
}

/// Responsive callout grid (1/2 cols)
#[component]
pub fn CalloutGrid(
    children: Children,
    #[prop(default = GridMd::OneTwo)] layout: GridMd,
) -> impl IntoView {
    view! {
        <div class=layout.classes()>
            {children()}
        </div>
    }
}
