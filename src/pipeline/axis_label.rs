//! 坐标轴标签布局辅助
//!
//! 用于解决密集时间轴/分类轴的长文本标签互相遮挡问题。
//!
//! 旋转/抽稀策略统一由 [`super::collision`] 提供（`CollisionResolver` /
//! `auto_rotate` / `label_step` 等），本模块在此基础之上 re-export，
//! 供坐标轴渲染层复用，并保留无字体引擎时的文本尺寸估计。

pub use super::collision::{
    ROT_45, ROT_90, auto_rotate, label_step, rotated_bounds,
};

/// 粗略估计文本渲染尺寸（像素），不依赖字体引擎。
///
/// 全角字符（CJK 等）按 1.0em、半角字符按 0.55em 估算宽度，
/// 高度按 1.2em 估算。用于 GridPlanner 在布局阶段预留坐标轴标签空间。
pub fn estimate_text_size(text: &str, font_size: f64) -> (f64, f64) {
    let mut width = 0.0;
    for ch in text.chars() {
        width += if is_wide_char(ch) {
            font_size
        } else {
            font_size * 0.55
        };
    }
    (width.max(font_size * 0.6), font_size * 1.2)
}

/// 全角/宽字符判断（CJK 统一表意文字、假名、谚文、全角标点等）
fn is_wide_char(c: char) -> bool {
    let code = c as u32;
    (0x1100..=0x115F).contains(&code) // 谚文 Jamo
        || (0x2E80..=0xA4CF).contains(&code) // CJK 部首/汉字/假名等
        || (0xAC00..=0xD7A3).contains(&code) // 谚文音节
        || (0xF900..=0xFAFF).contains(&code) // CJK 兼容表意文字
        || (0xFE30..=0xFE4F).contains(&code) // CJK 兼容符号
        || (0xFF00..=0xFF60).contains(&code) // 全角形式
        || (0xFFE0..=0xFFE6).contains(&code) // 全角符号
        || (0x3000..=0x303F).contains(&code) // CJK 符号和标点
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotated_bounds() {
        // 0°：保持原尺寸
        assert_eq!(rotated_bounds(60.0, 13.0, 0.0), (60.0, 13.0));
        // 90°：宽高互换
        let (w90, h90) = rotated_bounds(60.0, 13.0, ROT_90);
        assert!((w90 - 13.0).abs() < 1e-6 && (h90 - 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_auto_rotate_sequence() {
        // 槽宽很大 → 不旋转
        assert_eq!(auto_rotate(30.0, 13.0, 50.0), 0.0);
        // 横向放不下、45° 能放下
        assert_eq!(auto_rotate(60.0, 13.0, 52.0), ROT_45);
        // 45° 放不下且抽稀步长过大（> MAX_STEP_45）→ 90°
        assert_eq!(auto_rotate(200.0, 13.0, 30.0), ROT_90);
    }

    #[test]
    fn test_auto_rotate_prefer_45_via_thinning() {
        // 45° 投影宽度略超槽宽，但抽稀步长仍可控（≤ MAX_STEP_45）→ 优先 45°
        // 对应真实场景：长日期标签 w≈160、槽宽≈84 → 45° 投影≈122，步长=2
        let max_w = 160.0;
        let max_h = 13.0;
        let slot_w = 84.0;
        let proj_45 = rotated_bounds(max_w, max_h, ROT_45).0;
        assert!(proj_45 > slot_w, "前置条件：45° 投影宽度应略超槽宽");
        assert_eq!(label_step(proj_45, slot_w), 2, "45° 抽稀步长应可控");
        // 尽管 45° 投影 > 槽宽，仍应优先 45° 而非直接 90°
        assert_eq!(auto_rotate(max_w, max_h, slot_w), ROT_45);
    }

    #[test]
    fn test_label_step() {
        assert_eq!(label_step(30.0, 100.0), 1);
        assert_eq!(label_step(100.0, 30.0), 4);
        assert_eq!(label_step(0.0, 0.0), 1);
    }

    #[test]
    fn test_estimate_text_size() {
        // CJK 字符按 1em 计
        let (w_cjk, h) = estimate_text_size("周一", 11.0);
        assert!((w_cjk - 22.0).abs() < 1e-6);
        assert!((h - 13.2).abs() < 1e-6);
        // 半角按 0.55em 计
        let (w_latin, _) = estimate_text_size("abc", 11.0);
        assert!((w_latin - 11.0 * 0.55 * 3.0).abs() < 1e-6);
    }
}
