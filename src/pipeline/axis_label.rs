//! 坐标轴标签布局辅助
//!
//! 用于解决密集时间轴/分类轴的长文本标签互相遮挡问题：
//! - 自动选择旋转角度（0° → 45° → 90°）
//! - 旋转后仍放不下时按步长抽稀标签
//! - 无字体引擎时的文本尺寸估计（GridPlanner 预留边距用）

/// 45° 旋转（弧度）
pub const ROT_45: f64 = std::f64::consts::FRAC_PI_4;
/// 90° 旋转（弧度）
pub const ROT_90: f64 = std::f64::consts::FRAC_PI_2;

/// 文本 w×h 旋转 θ 弧度后投影包围盒的尺寸。
pub fn rotated_bounds(w: f64, h: f64, theta: f64) -> (f64, f64) {
    let (s, c) = theta.sin_cos();
    (w * c.abs() + h * s.abs(), w * s.abs() + h * c.abs())
}

/// 45° 旋转可接受的最大抽稀步长。
///
/// 45° 相比 90° 阅读更友好，因此只要 45° 旋转后通过抽稀仍能保持标签可读
/// （抽稀步长不超过该值，即每隔至多 `MAX_STEP_45 - 1` 个位置显示一个），
/// 就优先采用 45°；否则才降到 90°（90° 投影宽度极小，通常无需抽稀）。
const MAX_STEP_45: usize = 2;

/// 自动选择旋转角度（弧度）：
///
/// - 横向能放下 → 0°
/// - 45° 旋转后投影宽度 ≤ 槽宽，或抽稀步长不超过 `MAX_STEP_45` → 45°（优先）
/// - 否则 → 90°
///
/// 注意：决定旋转角时同样把抽稀纳入考量，避免"45° 略超槽宽但可抽稀"时
/// 误跳到 90°。
pub fn auto_rotate(max_w: f64, max_h: f64, slot_w: f64) -> f64 {
    if max_w <= slot_w {
        return 0.0;
    }
    let proj_45 = rotated_bounds(max_w, max_h, ROT_45).0;
    if proj_45 <= slot_w || label_step(proj_45, slot_w) <= MAX_STEP_45 {
        return ROT_45;
    }
    ROT_90
}

/// 计算标签渲染步长：每隔 `step` 个位置显示一个标签。
/// 1 表示全部显示。`projected_max_w` 是旋转后标签的投影宽度。
pub fn label_step(projected_max_w: f64, slot_w: f64) -> usize {
    if slot_w <= 0.0 {
        return 1;
    }
    (projected_max_w / slot_w).ceil().max(1.0) as usize
}

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
