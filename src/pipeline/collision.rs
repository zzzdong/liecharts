//! 统一的标签碰撞检测与处理框架。
//!
//! 解决两类标签碰撞问题，共享同一个包围盒模型与重叠检测：
//! - **坐标轴标签**（line/bar 的 x/y 轴刻度）：位置锁定，只能旋转/抽稀
//! - **数据标签**（饼图外部标签、图例）：位置相对自由，可位移避让
//!
//! 本模块抽象出：
//! - [`LabelBox`]：标签包围盒（含旋转角）
//! - [`boxes_overlap`]：两标签包围盒是否重叠（含旋转投影）
//! - [`CollisionResolver`]：碰撞处理策略接口
//! - 内置策略：[`DisplacementResolver`]（位移避让）、[`RotateThinningResolver`]（旋转+抽稀）

/// 标签包围盒（像素空间，左上角 + 宽高 + 旋转角弧度）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// 旋转角（弧度），0 表示不旋转
    pub rotation: f64,
}

impl LabelBox {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            x,
            y,
            w,
            h,
            rotation: 0.0,
        }
    }

    /// 旋转 θ 弧度后包围盒的投影尺寸 `(width', height')`。
    pub fn rotated_projection(&self, theta: f64) -> (f64, f64) {
        let (s, c) = theta.sin_cos();
        (
            self.w * c.abs() + self.h * s.abs(),
            self.w * s.abs() + self.h * c.abs(),
        )
    }

    /// 当前包围盒的投影尺寸（按自身 rotation）。
    pub fn projected_size(&self) -> (f64, f64) {
        if self.rotation == 0.0 {
            (self.w, self.h)
        } else {
            self.rotated_projection(self.rotation)
        }
    }
}

/// 判断两个标签包围盒（考虑各自旋转）是否在轴对齐投影下重叠。
///
/// 将每个标签按自身旋转角投影到正交包围盒后做 AABB 相交判断。
/// 这是近似的保守检测，足够用于标签避让。
pub fn boxes_overlap(a: &LabelBox, b: &LabelBox) -> bool {
    let (aw, ah) = a.projected_size();
    let (bw, bh) = b.projected_size();

    // AABB 相交：x 方向不相交 或 y 方向不相交 → 不重叠
    let x_overlap = a.x < b.x + bw && b.x < a.x + aw;
    let y_overlap = a.y < b.y + bh && b.y < a.y + ah;
    x_overlap && y_overlap
}

/// 碰撞处理策略。
pub trait CollisionResolver {
    /// 输入标签序列，返回处理后的标签（位置可能被调整）。
    ///
    /// 实现者决定采用位移避让、旋转抽稀或其他策略。
    fn resolve(&self, labels: Vec<LabelBox>) -> Vec<LabelBox>;
}

/// 位移避让策略：在给定轴上相邻标签间距不足时整体错开。
///
/// 用于饼图外部标签 / 图例等位置相对自由、可上下（或左右）移动的场景。
/// 按 axis 指定的坐标值排序后，保证相邻标签间距 ≥ `min_gap`。
pub struct DisplacementResolver {
    /// 最小间距（像素）
    pub min_gap: f64,
    /// 避让轴：`0` 表示按 y 值避让（上下），`1` 表示按 x 值避让（左右）
    pub axis: usize,
}

impl DisplacementResolver {
    pub fn new(min_gap: f64, axis: usize) -> Self {
        Self { min_gap, axis }
    }
}

impl CollisionResolver for DisplacementResolver {
    fn resolve(&self, mut labels: Vec<LabelBox>) -> Vec<LabelBox> {
        if labels.len() < 2 {
            return labels;
        }
        // 按避让轴的坐标排序
        labels.sort_by(|a, b| {
            let av = if self.axis == 0 { a.y } else { a.x };
            let bv = if self.axis == 0 { b.y } else { b.x };
            av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
        });

        for i in 1..labels.len() {
            // 计算当前与上一个在避让轴上的间距
            let prev_coord = if self.axis == 0 {
                labels[i - 1].y
            } else {
                labels[i - 1].x
            };
            let cur_coord = if self.axis == 0 {
                labels[i].y
            } else {
                labels[i].x
            };
            let gap = cur_coord - prev_coord;
            if gap < self.min_gap {
                let shift = self.min_gap - gap;
                if self.axis == 0 {
                    labels[i].y += shift;
                } else {
                    labels[i].x += shift;
                }
            }
        }
        labels
    }
}

/// 旋转 + 抽稀策略：用于坐标轴标签（位置锁定，不能移动）。
///
/// 在给定槽宽下自动选择旋转角（0° → 45° → 90°），旋转后仍放不下则按步长抽稀。
pub struct RotateThinningResolver {
    /// 每个刻度的槽宽（像素）
    pub slot_w: f64,
    /// 最大可接受的抽稀步长（45° 优先时）
    pub max_step: usize,
}

impl RotateThinningResolver {
    pub fn new(slot_w: f64) -> Self {
        Self {
            slot_w,
            max_step: 2,
        }
    }
}

impl CollisionResolver for RotateThinningResolver {
    fn resolve(&self, labels: Vec<LabelBox>) -> Vec<LabelBox> {
        if labels.is_empty() {
            return labels;
        }
        // 找最大宽高，决定统一旋转角
        let max_w = labels.iter().map(|l| l.w).fold(0.0_f64, f64::max);
        let max_h = labels.iter().map(|l| l.h).fold(0.0_f64, f64::max);
        let rotation = auto_rotate(max_w, max_h, self.slot_w);
        let step = label_step(rotated_bounds(max_w, max_h, rotation).0, self.slot_w);

        // 应用旋转 + 抽稀：每隔 step 保留一个标签
        labels
            .into_iter()
            .enumerate()
            .filter(|(i, _)| i % step == 0)
            .map(|(_, mut l)| {
                l.rotation = rotation;
                l
            })
            .collect()
    }
}

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
const MAX_STEP_45: usize = 2;

/// 自动选择旋转角度（弧度）：0° → 45°（优先）→ 90°。
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

/// 计算标签渲染步长：每隔 `step` 个位置显示一个标签。1 表示全部显示。
pub fn label_step(projected_max_w: f64, slot_w: f64) -> usize {
    if slot_w <= 0.0 {
        return 1;
    }
    (projected_max_w / slot_w).ceil().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boxes_overlap() {
        // 不重叠
        let a = LabelBox::new(0.0, 0.0, 10.0, 10.0);
        let b = LabelBox::new(20.0, 0.0, 10.0, 10.0);
        assert!(!boxes_overlap(&a, &b));
        // 重叠
        let c = LabelBox::new(5.0, 5.0, 10.0, 10.0);
        assert!(boxes_overlap(&a, &c));
    }

    #[test]
    fn test_displacement_resolver_y() {
        // 三个 y 相同的标签应被错开
        let labels = vec![
            LabelBox::new(0.0, 100.0, 50.0, 14.0),
            LabelBox::new(0.0, 100.0, 50.0, 14.0),
            LabelBox::new(0.0, 100.0, 50.0, 14.0),
        ];
        let resolver = DisplacementResolver::new(16.0, 0);
        let resolved = resolver.resolve(labels);
        assert!(resolved[1].y - resolved[0].y >= 16.0);
        assert!(resolved[2].y - resolved[1].y >= 16.0);
    }

    #[test]
    fn test_rotate_thinning_resolver() {
        // 槽宽很小，需要旋转/抽稀
        let labels = vec![
            LabelBox::new(0.0, 0.0, 60.0, 13.0),
            LabelBox::new(30.0, 0.0, 60.0, 13.0),
            LabelBox::new(60.0, 0.0, 60.0, 13.0),
        ];
        let n = labels.len();
        let resolver = RotateThinningResolver::new(30.0);
        let resolved = resolver.resolve(labels);
        // 至少抽稀掉一部分（step > 1）
        assert!(resolved.len() < n);
    }

    #[test]
    fn test_auto_rotate_sequence() {
        assert_eq!(auto_rotate(30.0, 13.0, 50.0), 0.0);
        assert_eq!(auto_rotate(60.0, 13.0, 52.0), ROT_45);
        assert_eq!(auto_rotate(200.0, 13.0, 30.0), ROT_90);
    }
}
