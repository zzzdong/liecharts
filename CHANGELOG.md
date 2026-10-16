# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-10-16

### Added

- **Font `body_font_size` as rem root** (ECharts `option.textStyle.fontSize` parity):
  - `Chart::new(...).body_font_size(n)` is now equivalent to JSON
    `option.textStyle.fontSize`. It injects a `1rem` root so the whole chart's
    text scales proportionally by theme token factors — title `1.5×`, axis/value
    labels `1.0×`, auxiliary text `0.833×`.
  - Extracted `chart_pipeline::apply_rem_root` so both the JSON and api-builder
    paths share one root-injection path. Added `tests/font_rem_test.rs` covering
    both paths.

### Fixed

- **Option/Default trap**: `label_font_size` (and related) were hardcoded defaults
  that were indistinguishable from an explicit `12`. They are now `Option<f64>`, so
  an unset label inherits the rem root instead of being misread as "explicit 12".

## [0.3.0] - 2026-10-14

### ⚠️ Breaking Changes

This release upgrades the `lievisual` dependency from `0.2` to `0.3`, which is a
breaking release of the rendering backend. Because `liecharts` re-exports several
`lievisual` types (`SceneNode`, text-layout types, etc.), code that directly
accesses those re-exported internals will need updates:

- **Node container model**: `SceneNode` no longer exposes `&node.element` by field.
  Use the `element()` accessor (and `element_mut()` for mutable access). `Group`
  nodes are now accessed via `children()` / `is_group()` instead of matching on a
  public field.
- **`RoundedRect` radii**: the radius is now carried in a `radii` field rather than
  a scalar. The `Element::rounded_rect` scalar-radius constructor is unchanged.
- **Gradients**: construct with `LinearGradient::new(start, end, stops)`; a new
  `spread` field was added (`LinearGradient { .. }` struct literal no longer works).

The primary `builder` / `option` / `Chart` / `ChartBuilder` API surface is
unchanged. Direct `parley` / `vello_cpu` dependencies were removed from
`liecharts`; raster output now uses `lievisual::Pixmap`.

### Fixed

- **Pie / Legend** (aligned with ECharts v6):
  - Legend row metric now uses `max(itemHeight, font_size) + itemGap` (22px/row),
    replacing the old `font_size*1.4 + 16 + 6`, so an 8-item vertical legend no
    longer compresses the plot area.
  - Added `resolve_legend()` so explicitly configured and auto legends share one
    resolution path; the bottom legend band is now a hard constraint for both the
    plot bottom edge and bottom X-axis labels (fixes overlap on `mixed_grid` /
    `hug_demo_3`).
  - Polar series now lay out against the canvas (center 50%/50%, radius
    `min(w,h)/2`) when no explicit `grid` is set, instead of being clamped into a
    grid's inscribed radius.
  - Pie center returns to 50% of bounds (was 0.55 offset); outside labels use a
    ECharts-style radial push + same-side vertical avoidance, leader lines use the
    sector color, and labels get a 1px white outline.
  - `{d}` percentage now formats to 2 decimals (e.g. `82.62%`).
  - Series now bind to their own grid's first x/y axis by default, fixing
    multi-subplot category-slot and Y-axis-range bleed.
  - Added `tests/legend_overlap_test.rs`.
- **Candlestick**: entity bodies now use ECharts' default solid fill, and the
  color source was consolidated. Added a regression test for entity fill.

[0.3.1]: https://github.com/zzzdong/liecharts/releases/tag/v0.3.1
[0.3.0]: https://github.com/zzzdong/liecharts/releases/tag/v0.3.0
