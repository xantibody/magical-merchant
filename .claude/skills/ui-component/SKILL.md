---
name: ui-component
description: UI component patterns for the Dioxus + Open Props desktop app. Use this skill when creating or modifying UI components to ensure consistency with the project's design system, layout conventions, and interaction patterns.
---

# UI Component Patterns

Use this skill when building UI components for the Dioxus desktop app.

## Design System

### Styling: Open Props

All styling uses Open Props CSS custom properties. Never use hardcoded values for colors, spacing, or typography.

```css
/* Good */
color: var(--text-1);
padding: var(--size-3);
border-radius: var(--radius-2);
font-size: var(--font-size-1);

/* Bad */
color: #333;
padding: 12px;
border-radius: 8px;
```

### Icons: Phosphor Icons

Use Phosphor Icons SVGs from `dioxus-app/assets/icons/`. Render via `img` + `asset!()` or inline SVG in RSX.

```rust
// Asset approach
rsx! {
    img {
        src: asset!("/assets/icons/lightning.svg"),
        class: "icon",
        alt: "Timeline"
    }
}
```

### Color Scheme

- Light mode: Open Props defaults
- Dark mode: `@media (prefers-color-scheme: dark)` with CSS custom property overrides
- Accent adjustments: Nightfox Dawnfox (light) / Duskfox (dark) palette when needed

## Component Patterns

### Layout Rules

1. **Full-screen memo area** — The memo/editor always occupies all available space
2. **Minimal header** — Only toggle button + current mode icon
3. **No visible chrome** — Borders, shadows, and decorations are minimal or absent
4. **Breathing room** — Use `var(--size-*)` tokens for consistent spacing

### Action Bar (Hidden Actions)

Actions are hidden by default and revealed on interaction:

```css
.action-bar {
    opacity: 0;
    transition: opacity 150ms var(--ease-2);
    pointer-events: none;
}

.memo-area:hover .action-bar,
.action-bar.visible {
    opacity: 1;
    pointer-events: auto;
}
```

```rust
#[component]
fn ActionBar(children: Element) -> Element {
    rsx! {
        div { class: "action-bar",
            {children}
        }
    }
}
```

### Toggle Menu

Opens/closes a mode selection overlay from the header toggle button:

```rust
#[component]
fn ToggleMenu(is_open: Signal<bool>) -> Element {
    if !is_open() {
        return rsx! {};
    }
    rsx! {
        nav { class: "toggle-menu",
            // Menu items with Phosphor icons
        }
    }
}
```

### Text Input

Full-width, borderless textarea that blends into the memo area:

```css
.memo-input {
    width: 100%;
    height: 100%;
    border: none;
    outline: none;
    resize: none;
    font-family: var(--font-sans);
    font-size: var(--font-size-2);
    line-height: var(--font-lineheight-3);
    background: transparent;
    color: var(--text-1);
}
```

### Markdown Preview

Rendered HTML displayed with `dangerous_inner_html`. Style with Open Props typography tokens:

```css
.markdown-preview {
    font-family: var(--font-sans);
    line-height: var(--font-lineheight-4);
    color: var(--text-1);
}

.markdown-preview h1 { font-size: var(--font-size-5); font-weight: var(--font-weight-7); }
.markdown-preview h2 { font-size: var(--font-size-4); font-weight: var(--font-weight-6); }
.markdown-preview h3 { font-size: var(--font-size-3); font-weight: var(--font-weight-6); }
.markdown-preview code { font-family: var(--font-mono); background: var(--surface-2); padding: var(--size-1); border-radius: var(--radius-1); }
.markdown-preview pre { background: var(--surface-2); padding: var(--size-3); border-radius: var(--radius-2); overflow-x: auto; }
```

## Priority Checklist

Before merging any UI component, verify against design priorities:

1. **Simple** — Does it add unnecessary visual complexity? Can it be simpler?
2. **Lightweight** — Does it add weight (deps, DOM nodes, CSS)? Can it be lighter?
3. **Stylish** — Does it look clean and intentional? Does it use design tokens consistently?
