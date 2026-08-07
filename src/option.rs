use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
};

use crate::sampling::SamplingOption;

/// Configuration that can be either a single item or multiple items.
/// Used for ECharts-compatible JSON parsing where a field can be an object or array.
#[derive(Debug, Clone)]
pub enum SingleOrMultiple<T> {
    Single(T),
    Multiple(Vec<T>),
}

impl<T> Default for SingleOrMultiple<T> {
    fn default() -> Self {
        SingleOrMultiple::Multiple(Vec::new())
    }
}

impl<T: Serialize> Serialize for SingleOrMultiple<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SingleOrMultiple::Single(item) => item.serialize(serializer),
            SingleOrMultiple::Multiple(items) => items.serialize(serializer),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for SingleOrMultiple<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SingleOrMultipleVisitor<T> {
            _phantom: std::marker::PhantomData<T>,
        }

        impl<'de, T: Deserialize<'de>> Visitor<'de> for SingleOrMultipleVisitor<T> {
            type Value = SingleOrMultiple<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a single object or an array of objects")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let item = T::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(SingleOrMultiple::Single(item))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let items = Vec::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                Ok(SingleOrMultiple::Multiple(items))
            }
        }

        deserializer.deserialize_any(SingleOrMultipleVisitor {
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<T> SingleOrMultiple<T> {
    /// Returns the items as a slice.
    pub fn as_slice(&self) -> &[T] {
        match self {
            SingleOrMultiple::Single(item) => std::slice::from_ref(item),
            SingleOrMultiple::Multiple(items) => items.as_slice(),
        }
    }

    /// Returns true if there are no items configured.
    pub fn is_empty(&self) -> bool {
        match self {
            SingleOrMultiple::Single(_) => false,
            SingleOrMultiple::Multiple(items) => items.is_empty(),
        }
    }

    /// Returns the number of items.
    pub fn len(&self) -> usize {
        match self {
            SingleOrMultiple::Single(_) => 1,
            SingleOrMultiple::Multiple(items) => items.len(),
        }
    }

    /// Returns an iterator over the items.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        match self {
            SingleOrMultiple::Single(item) => std::slice::from_ref(item).iter(),
            SingleOrMultiple::Multiple(items) => items.iter(),
        }
    }

    /// Returns a reference to the item at the given index.
    pub fn get(&self, index: usize) -> Option<&T> {
        match self {
            SingleOrMultiple::Single(item) => {
                if index == 0 {
                    Some(item)
                } else {
                    None
                }
            }
            SingleOrMultiple::Multiple(items) => items.get(index),
        }
    }
}

/// Type alias for grid configuration.
pub type GridConfig = SingleOrMultiple<GridOption>;

/// Type alias for axis configuration.
pub type AxisConfig = SingleOrMultiple<AxisOption>;

// ═══════════════════════════════════════════════════════════════════
// 容错类型 — 用于 ECharts JSON 兼容性
// ═══════════════════════════════════════════════════════════════════

/// 接受字符串或 usize 的灵活类型。
///
/// 用于 ECharts 中既可以是数字索引（`0`）又可以是字符串列名（`"product"`）的字段，
/// 例如 `series.encode.x` / `series.encode.y`。
#[derive(Debug, Clone, PartialEq)]
pub enum StringOrInt {
    Str(String),
    Int(usize),
}

impl StringOrInt {
    /// 返回字符串表示（数字会转为字符串）。
    pub fn as_str(&self) -> String {
        match self {
            StringOrInt::Str(s) => s.clone(),
            StringOrInt::Int(n) => n.to_string(),
        }
    }

    /// 如果是 Int，返回对应的 usize。
    pub fn as_int(&self) -> Option<usize> {
        match self {
            StringOrInt::Int(n) => Some(*n),
            StringOrInt::Str(s) => s.parse::<usize>().ok(),
        }
    }
}

impl Serialize for StringOrInt {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            StringOrInt::Str(s) => serializer.serialize_str(s),
            StringOrInt::Int(n) => serializer.serialize_u64(*n as u64),
        }
    }
}

impl<'de> Deserialize<'de> for StringOrInt {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StringOrIntVisitor;

        impl<'de> Visitor<'de> for StringOrIntVisitor {
            type Value = StringOrInt;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string or a number")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(StringOrInt::Str(v.to_string()))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(StringOrInt::Str(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(StringOrInt::Int(v as usize))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(StringOrInt::Int(v as usize))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(StringOrInt::Int(v as usize))
            }
        }

        deserializer.deserialize_any(StringOrIntVisitor)
    }
}

/// 接受数字或百分比字符串（如 `"100%"`）的灵活类型。
///
/// 用于 `dataZoom.handleSize` 等 ECharts 字段。
#[derive(Debug, Clone, PartialEq)]
pub enum NumberOrPercent {
    Number(f64),
    Percent(f64),
}

impl NumberOrPercent {
    /// 解析为最终数值：`Number(n)` 返回 `n`；`Percent(p)` 返回 `p`（百分比数值本身，由调用方换算）。
    pub fn raw_value(&self) -> f64 {
        match self {
            NumberOrPercent::Number(n) => *n,
            NumberOrPercent::Percent(p) => *p,
        }
    }

    /// 根据基准值换算百分比。`Number(n)` 返回 `n`；`Percent(p)` 返回 `base * p / 100.0`。
    pub fn resolve(&self, base: f64) -> f64 {
        match self {
            NumberOrPercent::Number(n) => *n,
            NumberOrPercent::Percent(p) => base * *p / 100.0,
        }
    }
}

impl Default for NumberOrPercent {
    fn default() -> Self {
        NumberOrPercent::Number(0.0)
    }
}

impl Serialize for NumberOrPercent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            NumberOrPercent::Number(n) => serializer.serialize_f64(*n),
            NumberOrPercent::Percent(p) => serializer.serialize_str(&format!("{}%", p)),
        }
    }
}

impl<'de> Deserialize<'de> for NumberOrPercent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct NumberOrPercentVisitor;

        impl<'de> Visitor<'de> for NumberOrPercentVisitor {
            type Value = NumberOrPercent;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number or a percentage string like \"100%\"")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                if let Some(stripped) = v.strip_suffix('%') {
                    let p = stripped
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid percentage: {}", v)))?;
                    Ok(NumberOrPercent::Percent(p))
                } else {
                    let n = v
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid number: {}", v)))?;
                    Ok(NumberOrPercent::Number(n))
                }
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(NumberOrPercent::Number(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(NumberOrPercent::Number(v as f64))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(NumberOrPercent::Number(v as f64))
            }
        }

        deserializer.deserialize_any(NumberOrPercentVisitor)
    }
}

/// 容错数字：接受 number、"auto" 或百分比字符串。
///
/// 用于 ECharts 中既可以是数字、也可以是 "auto" 或 "50%" 的字段，
/// 例如 `symbolSize`、`lineStyle.width`、`borderWidth` 等。
#[derive(Debug, Clone, PartialEq)]
pub enum LenientNumber {
    /// 具体数值
    Number(f64),
    /// "auto" 关键字
    Auto,
    /// 百分比字符串（如 "50%"）
    Percent(f64),
    /// 任意字符串（如分类轴引用 "Mon"）
    Category(String),
}

impl LenientNumber {
    /// 解析为最终数值。
    /// - `Number(n)` 返回 `n`
    /// - `Auto` 返回 `default_value`
    /// - `Percent(p)` 返回 `base * p / 100.0`
    /// - `Category(_)` 返回 `default_value`
    pub fn resolve(&self, base: f64, default_value: f64) -> f64 {
        match self {
            LenientNumber::Number(n) => *n,
            LenientNumber::Auto => default_value,
            LenientNumber::Percent(p) => base * *p / 100.0,
            LenientNumber::Category(_) => default_value,
        }
    }

    /// 如果是 Number，返回数值；否则返回 None。
    pub fn as_number(&self) -> Option<f64> {
        match self {
            LenientNumber::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// 如果是 Category，返回字符串引用。
    pub fn as_category(&self) -> Option<&str> {
        match self {
            LenientNumber::Category(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl Default for LenientNumber {
    fn default() -> Self {
        LenientNumber::Number(0.0)
    }
}

impl Serialize for LenientNumber {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LenientNumber::Number(n) => serializer.serialize_f64(*n),
            LenientNumber::Auto => serializer.serialize_str("auto"),
            LenientNumber::Percent(p) => serializer.serialize_str(&format!("{}%", p)),
            LenientNumber::Category(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for LenientNumber {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenientNumberVisitor;

        impl<'de> Visitor<'de> for LenientNumberVisitor {
            type Value = LenientNumber;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(
                    "a number, \"auto\", a percentage string like \"50%\", or a category string",
                )
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let v = v.trim();
                if v.eq_ignore_ascii_case("auto") {
                    Ok(LenientNumber::Auto)
                } else if let Some(stripped) = v.strip_suffix('%') {
                    let p = stripped
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid percentage: {}", v)))?;
                    Ok(LenientNumber::Percent(p))
                } else if let Ok(n) = v.parse::<f64>() {
                    Ok(LenientNumber::Number(n))
                } else {
                    // 任意字符串，作为分类引用
                    Ok(LenientNumber::Category(v.to_string()))
                }
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(LenientNumber::Number(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(LenientNumber::Number(v as f64))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(LenientNumber::Number(v as f64))
            }
        }

        deserializer.deserialize_any(LenientNumberVisitor)
    }
}

/// 容错内边距：接受 number、string 或 array。
///
/// 用于 ECharts 中的 padding 字段，可以是：
/// - 单个数字：`10`
/// - 字符串：`"10"` 或 `"10%"`
/// - 数组：`[10, 20]` 或 `[10, 20, 30, 40]`
#[derive(Debug, Clone, PartialEq)]
pub enum LenientPadding {
    /// 单值（像素）
    Single(f64),
    /// 百分比
    Percent(f64),
    /// 四边独立值 [top, right, bottom, left]
    Array(Vec<LenientNumber>),
}

impl LenientPadding {
    /// 解析为 [top, right, bottom, left] 四值数组。
    pub fn resolve(&self, base: f64, default_value: f64) -> [f64; 4] {
        match self {
            LenientPadding::Single(v) => [*v; 4],
            LenientPadding::Percent(p) => {
                let v = base * *p / 100.0;
                [v; 4]
            }
            LenientPadding::Array(arr) => match arr.len() {
                0 => [default_value; 4],
                1 => [arr[0].resolve(base, default_value); 4],
                2 => {
                    let v = [
                        arr[0].resolve(base, default_value),
                        arr[1].resolve(base, default_value),
                    ];
                    [v[0], v[1], v[0], v[1]]
                }
                3 => {
                    let v0 = arr[0].resolve(base, default_value);
                    let v1 = arr[1].resolve(base, default_value);
                    let v2 = arr[2].resolve(base, default_value);
                    [v0, v1, v2, v1]
                }
                _ => {
                    let v0 = arr[0].resolve(base, default_value);
                    let v1 = arr[1].resolve(base, default_value);
                    let v2 = arr[2].resolve(base, default_value);
                    let v3 = arr[3].resolve(base, default_value);
                    [v0, v1, v2, v3]
                }
            },
        }
    }
}

impl Default for LenientPadding {
    fn default() -> Self {
        LenientPadding::Single(0.0)
    }
}

impl Serialize for LenientPadding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LenientPadding::Single(v) => serializer.serialize_f64(*v),
            LenientPadding::Percent(p) => serializer.serialize_str(&format!("{}%", p)),
            LenientPadding::Array(arr) => arr.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for LenientPadding {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenientPaddingVisitor;

        impl<'de> Visitor<'de> for LenientPaddingVisitor {
            type Value = LenientPadding;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number, string, or array for padding")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let v = v.trim();
                if let Some(stripped) = v.strip_suffix('%') {
                    let p = stripped
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid percentage: {}", v)))?;
                    Ok(LenientPadding::Percent(p))
                } else {
                    let n = v
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid number: {}", v)))?;
                    Ok(LenientPadding::Single(n))
                }
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(LenientPadding::Single(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(LenientPadding::Single(v as f64))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(LenientPadding::Single(v as f64))
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut arr = Vec::new();
                while let Some(v) = seq.next_element::<LenientNumber>()? {
                    arr.push(v);
                }
                Ok(LenientPadding::Array(arr))
            }
        }

        deserializer.deserialize_any(LenientPaddingVisitor)
    }
}

/// 容错 bool：接受 bool 或 "true"/"false" 字符串。
///
/// LLM 偶尔会输出 `"animation":"true"`，此类型用于让解析不报错。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LenientBool(pub bool);

/// 支持单个值或数组的灵活类型。
///
/// 用于 ECharts 中的 `radius` 等字段，可以是：
/// - 单个值：`"75%"` 或 `75`
/// - 数组：`["40%", "70%"]` 或 `[40, 70]`
#[derive(Debug, Clone, PartialEq)]
pub enum SingleOrArray<T> {
    Single(T),
    Array(Vec<T>),
}

impl<T: Clone> SingleOrArray<T> {
    /// 转换为数组形式。
    pub fn to_vec(&self) -> Vec<T> {
        match self {
            SingleOrArray::Single(v) => vec![v.clone()],
            SingleOrArray::Array(v) => v.clone(),
        }
    }
}

impl<T: Serialize> Serialize for SingleOrArray<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SingleOrArray::Single(v) => v.serialize(serializer),
            SingleOrArray::Array(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for SingleOrArray<LenientNumber> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SingleOrArrayVisitor;

        impl<'de> Visitor<'de> for SingleOrArrayVisitor {
            type Value = SingleOrArray<LenientNumber>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a single value or an array")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let ln = LenientNumber::deserialize(de::value::StrDeserializer::new(v))?;
                Ok(SingleOrArray::Single(ln))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                let ln = LenientNumber::deserialize(de::value::StringDeserializer::new(v))?;
                Ok(SingleOrArray::Single(ln))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(SingleOrArray::Single(LenientNumber::Number(v)))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(SingleOrArray::Single(LenientNumber::Number(v as f64)))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(SingleOrArray::Single(LenientNumber::Number(v as f64)))
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut arr = Vec::new();
                while let Some(v) = seq.next_element::<LenientNumber>()? {
                    arr.push(v);
                }
                Ok(SingleOrArray::Array(arr))
            }
        }

        deserializer.deserialize_any(SingleOrArrayVisitor)
    }
}

impl From<bool> for LenientBool {
    fn from(b: bool) -> Self {
        LenientBool(b)
    }
}

impl From<LenientBool> for bool {
    fn from(b: LenientBool) -> Self {
        b.0
    }
}

impl Serialize for LenientBool {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(self.0)
    }
}

impl<'de> Deserialize<'de> for LenientBool {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenientBoolVisitor;

        impl<'de> Visitor<'de> for LenientBoolVisitor {
            type Value = LenientBool;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a boolean or a \"true\"/\"false\" string")
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(LenientBool(v))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v.trim().to_ascii_lowercase().as_str() {
                    "true" | "1" => Ok(LenientBool(true)),
                    "false" | "0" => Ok(LenientBool(false)),
                    _ => Err(de::Error::custom(format!("invalid bool string: {}", v))),
                }
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(LenientBool(v != 0))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(LenientBool(v != 0))
            }
        }

        deserializer.deserialize_any(LenientBoolVisitor)
    }
}

/// 布尔值或字符串，用于 `selectedMode` 等同时支持 bool/string 的字段。
///
/// ECharts 的 `legend.selectedMode` / `pie.selectedMode` 支持：
/// - 布尔值：`true`（多选）/ `false`（禁用）
/// - 字符串：`"single"` / `"multiple"`
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum LenientBoolOrString {
    Bool(bool),
    Str(String),
}

impl<'de> Deserialize<'de> for LenientBoolOrString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = LenientBoolOrString;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a boolean or a string")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(LenientBoolOrString::Bool(v))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(LenientBoolOrString::Str(v.to_string()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(LenientBoolOrString::Str(v))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(LenientBoolOrString::Bool(v != 0))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(LenientBoolOrString::Bool(v != 0))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

/// 灵活的轴数据：接受字符串数组或数字数组。
///
/// ECharts 的 `axis.data` 可以是 `["a", "b"]` 或 `[1, 2, 3]`。
#[derive(Debug, Clone)]
pub struct LenientAxisData(pub Vec<String>);

impl std::ops::Deref for LenientAxisData {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for LenientAxisData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LenientAxisData {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenientAxisDataVisitor;
        impl<'de> de::Visitor<'de> for LenientAxisDataVisitor {
            type Value = LenientAxisData;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string array or number array")
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut result = Vec::new();
                while let Some(v) = seq.next_element::<serde_json::Value>()? {
                    match v {
                        serde_json::Value::String(s) => result.push(s),
                        serde_json::Value::Number(n) => result.push(n.to_string()),
                        _ => {
                            return Err(de::Error::custom(
                                "expected string or number in axis data",
                            ));
                        }
                    }
                }
                Ok(LenientAxisData(result))
            }
        }
        deserializer.deserialize_seq(LenientAxisDataVisitor)
    }
}

/// 灵活的轴范围值：接受数字或 `"dataMin"` / `"dataMax"` 字符串。
///
/// ECharts 的 `axis.min` / `axis.max` 支持数值和特殊字符串。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LenientAxisLimit {
    Value(f64),
    DataMin,
    DataMax,
}

impl Serialize for LenientAxisLimit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LenientAxisLimit::Value(v) => v.serialize(serializer),
            LenientAxisLimit::DataMin => "dataMin".serialize(serializer),
            LenientAxisLimit::DataMax => "dataMax".serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for LenientAxisLimit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenientAxisLimitVisitor;
        impl<'de> de::Visitor<'de> for LenientAxisLimitVisitor {
            type Value = LenientAxisLimit;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number or \"dataMin\"/\"dataMax\" string")
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(LenientAxisLimit::Value(v))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(LenientAxisLimit::Value(v as f64))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(LenientAxisLimit::Value(v as f64))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "dataMin" => Ok(LenientAxisLimit::DataMin),
                    "dataMax" => Ok(LenientAxisLimit::DataMax),
                    _ => Err(de::Error::custom(format!("unknown axis limit: {}", v))),
                }
            }
        }
        deserializer.deserialize_any(LenientAxisLimitVisitor)
    }
}

/// 灵活的边界间隙：接受布尔值或 `[string, string]` 数组。
///
/// ECharts 的 `boundaryGap` 可以是 `true` / `false` 或 `["20%", "20%"]`。
#[derive(Debug, Clone, PartialEq)]
pub enum LenientBoundaryGap {
    Bool(bool),
    Gap(LenientNumber, LenientNumber),
}

impl Default for LenientBoundaryGap {
    fn default() -> Self {
        LenientBoundaryGap::Bool(true)
    }
}

impl Serialize for LenientBoundaryGap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LenientBoundaryGap::Bool(b) => b.serialize(serializer),
            LenientBoundaryGap::Gap(a, b) => [a, b].serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for LenientBoundaryGap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenientBoundaryGapVisitor;
        impl<'de> de::Visitor<'de> for LenientBoundaryGapVisitor {
            type Value = LenientBoundaryGap;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a boolean or a [number|string, number|string] array")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(LenientBoundaryGap::Bool(v))
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let a = seq
                    .next_element::<LenientNumber>()?
                    .ok_or_else(|| de::Error::custom("expected at least 2 elements"))?;
                let b = seq
                    .next_element::<LenientNumber>()?
                    .ok_or_else(|| de::Error::custom("expected at least 2 elements"))?;
                Ok(LenientBoundaryGap::Gap(a, b))
            }
        }
        deserializer.deserialize_any(LenientBoundaryGapVisitor)
    }
}

/// 灵活的 step 类型：接受布尔值或 `"start"` / `"middle"` / `"end"` 字符串。
///
/// ECharts 的 `step` 可以是 `true` / `false` 或 `'start'` / `'middle'` / `'end'`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LenientStep {
    Bool(bool),
    Start,
    Middle,
    End,
}

impl Serialize for LenientStep {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LenientStep::Bool(b) => b.serialize(serializer),
            LenientStep::Start => "start".serialize(serializer),
            LenientStep::Middle => "middle".serialize(serializer),
            LenientStep::End => "end".serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for LenientStep {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenientStepVisitor;
        impl<'de> de::Visitor<'de> for LenientStepVisitor {
            type Value = LenientStep;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a boolean or \"start\"/\"middle\"/\"end\" string")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(LenientStep::Bool(v))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "start" => Ok(LenientStep::Start),
                    "middle" => Ok(LenientStep::Middle),
                    "end" => Ok(LenientStep::End),
                    _ => Err(de::Error::custom(format!("unknown step value: {}", v))),
                }
            }
        }
        deserializer.deserialize_any(LenientStepVisitor)
    }
}

/// 灵活的 bar 尺寸：接受数字或字符串（如 `"20%"`）。
///
/// ECharts 的 `barWidth` / `barMaxWidth` / `barMinWidth` / `barGap` / `barCategoryGap`
/// 可以是数字（像素值）或字符串（如 `"20%"`）。
#[derive(Debug, Clone, PartialEq)]
pub struct LenientBarSize(pub String);

impl Serialize for LenientBarSize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LenientBarSize {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LenientBarSizeVisitor;
        impl<'de> de::Visitor<'de> for LenientBarSizeVisitor {
            type Value = LenientBarSize;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number or a string like \"20%\"")
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(LenientBarSize(v.to_string()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(LenientBarSize(v.to_string()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(LenientBarSize(v.to_string()))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(LenientBarSize(v.to_string()))
            }
        }
        deserializer.deserialize_any(LenientBarSizeVisitor)
    }
}

/// 图例数据项：字符串或 `{name, icon}` 对象。
///
/// ECharts 允许 `legend.data` 同时包含字符串和对象。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LegendDataItem {
    Str(String),
    Object {
        name: String,
        #[serde(default)]
        icon: Option<String>,
        #[serde(default)]
        color: Option<ColorOption>,
    },
}

impl LegendDataItem {
    /// 返回数据项的名称。
    pub fn name(&self) -> &str {
        match self {
            LegendDataItem::Str(s) => s,
            LegendDataItem::Object { name, .. } => name,
        }
    }
}

/// 区间值：数字或 "auto"。
///
/// 用于 `axisLabel.interval` 等 ECharts 字段。
#[derive(Debug, Clone, PartialEq, Default)]
pub enum IntervalOption {
    #[default]
    Auto,
    Fixed(f64),
}

impl Serialize for IntervalOption {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            IntervalOption::Auto => serializer.serialize_str("auto"),
            IntervalOption::Fixed(n) => serializer.serialize_f64(*n),
        }
    }
}

impl<'de> Deserialize<'de> for IntervalOption {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct IntervalVisitor;

        impl<'de> Visitor<'de> for IntervalVisitor {
            type Value = IntervalOption;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number or the string \"auto\"")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                if v.trim().eq_ignore_ascii_case("auto") {
                    Ok(IntervalOption::Auto)
                } else {
                    let n = v
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid interval: {}", v)))?;
                    Ok(IntervalOption::Fixed(n))
                }
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(IntervalOption::Fixed(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(IntervalOption::Fixed(v as f64))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(IntervalOption::Fixed(v as f64))
            }
        }

        deserializer.deserialize_any(IntervalVisitor)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tooltip — 提示框组件
// ═══════════════════════════════════════════════════════════════════

/// Tooltip trigger type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum TooltipTrigger {
    #[default]
    Item,
    Axis,
    None,
}

/// Tooltip configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TooltipOption {
    pub show: Option<bool>,
    pub trigger: Option<TooltipTrigger>,
    pub formatter: Option<String>,
    pub value_formatter: Option<String>,
    pub background_color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub padding: Option<LenientPadding>,
    pub text_style: Option<TextStyleOption>,
    pub axis_pointer: Option<AxisPointerOption>,
    pub always_show_content: Option<bool>,
    pub trigger_on: Option<String>,
    pub confine: Option<bool>,
    pub hide_delay: Option<f64>,
    pub show_delay: Option<f64>,
    pub transition_duration: Option<f64>,
    pub enterable: Option<bool>,
    pub render_mode: Option<String>,
}

impl Default for TooltipOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            trigger: Some(TooltipTrigger::Item),
            formatter: None,
            value_formatter: None,
            background_color: None,
            border_color: None,
            border_width: None,
            padding: None,
            text_style: None,
            axis_pointer: None,
            always_show_content: None,
            trigger_on: None,
            confine: None,
            hide_delay: Some(100.0),
            show_delay: None,
            transition_duration: Some(0.4),
            enterable: None,
            render_mode: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// AxisPointer — 坐标轴指示器
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum AxisPointerType {
    #[default]
    Line,
    Shadow,
    Cross,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisPointerOption {
    pub show: Option<bool>,
    #[serde(rename = "type")]
    pub pointer_type: Option<AxisPointerType>,
    pub snap: Option<bool>,
    pub z: Option<f64>,
    pub label: Option<AxisPointerLabelOption>,
    pub line_style: Option<LineStyleOption>,
    pub shadow_style: Option<ShadowStyleOption>,
    pub trigger_tooltip: Option<bool>,
    pub value: Option<f64>,
    pub status: Option<bool>,
    pub handle: Option<HandleOption>,
}

impl Default for AxisPointerOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            pointer_type: Some(AxisPointerType::Line),
            snap: None,
            z: None,
            label: None,
            line_style: None,
            shadow_style: None,
            trigger_tooltip: Some(true),
            value: None,
            status: None,
            handle: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisPointerLabelOption {
    pub show: Option<bool>,
    pub formatter: Option<String>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub padding: Option<LenientPadding>,
    pub background_color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
}

impl Default for AxisPointerLabelOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            formatter: None,
            color: None,
            font_size: None,
            font_family: None,
            font_weight: None,
            padding: None,
            background_color: None,
            border_color: None,
            border_width: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct HandleOption {
    pub show: Option<bool>,
    pub icon: Option<String>,
    pub size: Option<f64>,
    pub margin: Option<f64>,
    pub color: Option<ColorOption>,
    pub throttle: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════
// Dataset — 数据集声明
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct DatasetOption {
    pub id: Option<String>,
    pub source: Option<Vec<Vec<serde_json::Value>>>,
    pub source_header: Option<bool>,
    pub dimensions: Option<Vec<String>>,
    pub from_dataset_index: Option<usize>,
    pub from_transform_result: Option<usize>,
}

// ═══════════════════════════════════════════════════════════════════
// Animation — 动画配置
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum EasingFunction {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    SinIn,
    SinOut,
    SinInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BackIn,
    BackOut,
    BackInOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationOption {
    pub duration: Option<f64>,
    pub easing: Option<EasingFunction>,
    pub delay: Option<f64>,
    pub duration_update: Option<f64>,
    pub easing_update: Option<EasingFunction>,
    pub delay_update: Option<f64>,
}

impl Default for AnimationOption {
    fn default() -> Self {
        Self {
            duration: Some(1000.0),
            easing: Some(EasingFunction::CubicOut),
            delay: None,
            duration_update: Some(300.0),
            easing_update: Some(EasingFunction::CubicInOut),
            delay_update: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// VisualMap — 视觉映射组件
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum VisualMapType {
    #[default]
    Continuous,
    Piecewise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualMapOption {
    pub show: Option<bool>,
    #[serde(rename = "type")]
    pub visual_map_type: Option<VisualMapType>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub calculable: Option<bool>,
    pub series_index: Option<OneOrMany<usize>>,
    pub dimension: Option<usize>,
    pub in_range: Option<VisualMapRangeOption>,
    pub out_of_range: Option<VisualMapRangeOption>,
    pub text: Option<Vec<String>>,
    pub text_style: Option<TextStyleOption>,
    pub color: Option<Vec<ColorOption>>,
    pub hover_link: Option<bool>,
    pub inverse: Option<bool>,
    pub precision: Option<usize>,
    pub item_width: Option<f64>,
    pub item_height: Option<f64>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub handle_icon: Option<String>,
    pub handle_size: Option<f64>,
    pub indicator_icon: Option<String>,
    pub indicator_size: Option<f64>,
    pub left: Option<PositionOption>,
    pub top: Option<PositionOption>,
    pub right: Option<PositionOption>,
    pub bottom: Option<PositionOption>,
    pub orient: Option<Orient>,
    pub padding: Option<LenientPadding>,
    pub background_color: Option<ColorOption>,
}

impl Default for VisualMapOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            visual_map_type: Some(VisualMapType::Continuous),
            min: Some(0.0),
            max: Some(100.0),
            calculable: Some(false),
            series_index: None,
            dimension: None,
            in_range: None,
            out_of_range: None,
            text: None,
            text_style: None,
            color: None,
            hover_link: Some(true),
            inverse: None,
            precision: None,
            item_width: None,
            item_height: None,
            border_color: None,
            border_width: None,
            handle_icon: None,
            handle_size: None,
            indicator_icon: None,
            indicator_size: None,
            left: None,
            top: None,
            right: None,
            bottom: None,
            orient: None,
            padding: None,
            background_color: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualMapRangeOption {
    pub color: Option<OneOrMany<ColorOption>>,
    pub symbol: Option<String>,
    pub symbol_size: Option<OneOrMany<f64>>,
    pub color_alpha: Option<OneOrMany<f64>>,
    pub color_lightness: Option<OneOrMany<f64>>,
    pub color_saturation: Option<OneOrMany<f64>>,
    pub color_hue: Option<OneOrMany<f64>>,
}

// ═══════════════════════════════════════════════════════════════════
// DataZoom — 数据区域缩放组件
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum DataZoomType {
    #[default]
    Inside,
    Slider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataZoomOption {
    pub show: Option<bool>,
    #[serde(rename = "type")]
    pub zoom_type: Option<DataZoomType>,
    pub x_axis_index: Option<OneOrMany<usize>>,
    pub y_axis_index: Option<OneOrMany<usize>>,
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub start_value: Option<f64>,
    pub end_value: Option<f64>,
    pub min_value_span: Option<f64>,
    pub max_value_span: Option<f64>,
    pub orient: Option<Orient>,
    pub zoom_lock: Option<bool>,
    pub throttle: Option<f64>,
    pub range_mode: Option<String>,
    pub left: Option<PositionOption>,
    pub top: Option<PositionOption>,
    pub right: Option<PositionOption>,
    pub bottom: Option<PositionOption>,
    pub background_color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub filler_color: Option<ColorOption>,
    pub handle_icon: Option<String>,
    /// 缩放条手柄尺寸，接受数字或百分比字符串（如 `100%`）
    pub handle_size: Option<LenientNumber>,
    pub handle_style: Option<ItemStyleOption>,
    pub data_background_color: Option<ColorOption>,
    pub selected_data_background_color: Option<ColorOption>,
}

impl Default for DataZoomOption {
    fn default() -> Self {
        Self {
            show: None,
            zoom_type: Some(DataZoomType::Inside),
            x_axis_index: None,
            y_axis_index: None,
            start: Some(0.0),
            end: Some(100.0),
            start_value: None,
            end_value: None,
            min_value_span: None,
            max_value_span: None,
            orient: None,
            zoom_lock: None,
            throttle: None,
            range_mode: None,
            left: None,
            top: None,
            right: None,
            bottom: None,
            background_color: None,
            border_color: None,
            border_width: None,
            filler_color: None,
            handle_icon: None,
            handle_size: None,
            handle_style: None,
            data_background_color: None,
            selected_data_background_color: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Brush — 区域选择组件
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum BrushType {
    #[default]
    Rect,
    Polygon,
    LineX,
    LineY,
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum BrushMode {
    #[default]
    Single,
    Multiple,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrushOption {
    pub brush_type: Option<BrushType>,
    pub brush_mode: Option<BrushMode>,
    pub transformable: Option<bool>,
    pub brush_style: Option<ItemStyleOption>,
    pub throttle: Option<f64>,
    pub geo_index: Option<Vec<usize>>,
    pub x_axis_index: Option<Vec<usize>>,
    pub y_axis_index: Option<Vec<usize>>,
    pub brush_link: Option<Vec<usize>>,
    pub series_index: Option<Vec<usize>>,
    pub in_brush: Option<BrushSelectOption>,
    pub out_of_brush: Option<BrushSelectOption>,
    pub z: Option<f64>,
}

impl Default for BrushOption {
    fn default() -> Self {
        Self {
            brush_type: Some(BrushType::Rect),
            brush_mode: Some(BrushMode::Single),
            transformable: Some(true),
            brush_style: None,
            throttle: None,
            geo_index: None,
            x_axis_index: None,
            y_axis_index: None,
            brush_link: None,
            series_index: None,
            in_brush: None,
            out_of_brush: None,
            z: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrushSelectOption {
    pub color: Option<ColorOption>,
    pub color_alpha: Option<f64>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════
// SplitArea — 轴分隔区域
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitAreaOption {
    pub show: Option<bool>,
    pub interval: Option<f64>,
    pub area_style: Option<AreaStyleOption>,
    pub color: Option<Vec<ColorOption>>,
}

impl Default for SplitAreaOption {
    fn default() -> Self {
        Self {
            show: Some(false),
            interval: None,
            area_style: None,
            color: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// ShadowStyle — 阴影样式
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct ShadowStyleOption {
    pub color: Option<ColorOption>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub opacity: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════
// MarkPoint / MarkLine / MarkArea — 标记系列
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkPointOption {
    pub data: Option<Vec<MarkPointDataOption>>,
    pub symbol: Option<SymbolType>,
    pub symbol_size: Option<LenientNumber>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
}

impl Default for MarkPointOption {
    fn default() -> Self {
        Self {
            data: None,
            symbol: Some(SymbolType::Pin),
            symbol_size: Some(LenientNumber::Number(50.0)),
            item_style: None,
            label: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct MarkPointDataOption {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub value_index: Option<usize>,
    pub value_dim: Option<String>,
    pub coord: Option<Vec<f64>>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub value: Option<f64>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub symbol: Option<SymbolType>,
    pub symbol_size: Option<LenientNumber>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct MarkLineOption {
    pub data: Option<Vec<OneOrMany<MarkLineDataOption>>>,
    pub symbol: Option<OneOrMany<SymbolType>>,
    pub symbol_size: Option<Vec<f64>>,
    pub line_style: Option<LineStyleOption>,
    pub label: Option<LabelOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub precision: Option<usize>,
    pub silent: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct MarkLineDataOption {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub value_index: Option<usize>,
    pub value_dim: Option<String>,
    pub x_axis: Option<LenientNumber>,
    pub y_axis: Option<LenientNumber>,
    pub coord: Option<Vec<LenientNumber>>,
    pub x: Option<LenientNumber>,
    pub y: Option<LenientNumber>,
    pub value: Option<f64>,
    pub line_style: Option<LineStyleOption>,
    pub label: Option<LabelOption>,
    pub symbol: Option<OneOrMany<SymbolType>>,
    pub symbol_size: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct MarkAreaOption {
    pub data: Option<Vec<Vec<MarkAreaDataOption>>>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub silent: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct MarkAreaDataOption {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub value_index: Option<usize>,
    pub value_dim: Option<String>,
    pub x_axis: Option<LenientNumber>,
    pub y_axis: Option<LenientNumber>,
    pub coord: Option<Vec<LenientNumber>>,
    pub x: Option<LenientNumber>,
    pub y: Option<LenientNumber>,
    pub value: Option<f64>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
}

// ═══════════════════════════════════════════════════════════════════
// SeriesEncode — 数据编码
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct SeriesEncodeOption {
    pub x: Option<OneOrMany<StringOrInt>>,
    pub y: Option<OneOrMany<StringOrInt>>,
    pub width: Option<OneOrMany<StringOrInt>>,
    pub height: Option<OneOrMany<StringOrInt>>,
    pub angle: Option<OneOrMany<StringOrInt>>,
    pub radius: Option<OneOrMany<StringOrInt>>,
    pub value: Option<OneOrMany<StringOrInt>>,
    pub item_name: Option<OneOrMany<StringOrInt>>,
    pub item_group_id: Option<OneOrMany<StringOrInt>>,
    pub tooltip: Option<OneOrMany<StringOrInt>>,
    pub series_name: Option<OneOrMany<StringOrInt>>,
}

/// Root chart configuration.
///
/// This is the raw option struct that mirrors the ECharts JSON schema. In most cases
/// you should use [`ChartBuilder`](crate::ChartBuilder) instead of constructing this directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct ChartOption {
    pub title: Option<TitleOption>,
    pub legend: Option<LegendOption>,
    #[serde(default)]
    pub grid: GridConfig,
    pub radar: Option<RadarOption>,
    #[serde(default)]
    pub x_axis: AxisConfig,
    #[serde(default)]
    pub y_axis: AxisConfig,
    #[serde(default)]
    pub series: Vec<SeriesOption>,
    pub color: Option<OneOrMany<ColorOption>>,
    pub background_color: Option<ColorOption>,
    pub theme: Option<String>,
    pub text_style: Option<TextStyleOption>,
    pub tooltip: Option<TooltipOption>,
    pub visual_map: Option<SingleOrMultiple<VisualMapOption>>,
    pub data_zoom: Option<SingleOrMultiple<DataZoomOption>>,
    pub dataset: Option<SingleOrMultiple<DatasetOption>>,
}

/// Chart title configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleOption {
    pub text: Option<String>,
    pub subtext: Option<String>,
    pub left: Option<PositionOption>,
    pub top: Option<PositionOption>,
    pub right: Option<PositionOption>,
    pub bottom: Option<PositionOption>,
    pub text_style: Option<TextStyleOption>,
    pub subtext_style: Option<TextStyleOption>,
    pub text_align: Option<String>,
    pub text_vertical_align: Option<String>,
    pub item_gap: Option<f64>,
    pub show: Option<bool>,
    pub target: Option<String>,
    pub sublink: Option<String>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub border_radius: Option<LenientNumber>,
    pub background_color: Option<ColorOption>,
    pub padding: Option<LenientPadding>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub trigger_event: Option<bool>,
}

impl Default for TitleOption {
    fn default() -> Self {
        Self {
            text: None,
            subtext: None,
            left: Some(PositionOption::center()),
            top: Some(PositionOption::auto()),
            right: None,
            bottom: None,
            text_style: None,
            subtext_style: None,
            text_align: None,
            text_vertical_align: None,
            item_gap: None,
            show: Some(true),
            target: None,
            sublink: None,
            z: None,
            zlevel: None,
            border_color: None,
            border_width: None,
            border_radius: None,
            background_color: None,
            padding: None,
            shadow_blur: None,
            shadow_color: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            trigger_event: None,
        }
    }
}

impl TitleOption {
    /// 创建一个标题，text 为必填
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// Sets the subtitle.
    pub fn subtext(mut self, subtext: impl Into<String>) -> Self {
        self.subtext = Some(subtext.into());
        self
    }

    pub fn left(mut self, left: PositionOption) -> Self {
        self.left = Some(left);
        self
    }

    pub fn top(mut self, top: PositionOption) -> Self {
        self.top = Some(top);
        self
    }

    pub fn text_style(mut self, style: TextStyleOption) -> Self {
        self.text_style = Some(style);
        self
    }

    pub fn subtext_style(mut self, style: TextStyleOption) -> Self {
        self.subtext_style = Some(style);
        self
    }
}

/// Legend configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LegendOption {
    pub show: Option<bool>,
    /// 图例数据项，可以是字符串数组或 `{name, icon}` 对象数组。
    pub data: Option<Vec<LegendDataItem>>,
    pub left: Option<PositionOption>,
    pub top: Option<PositionOption>,
    pub right: Option<PositionOption>,
    pub bottom: Option<PositionOption>,
    pub orient: Option<Orient>,
    pub text_style: Option<TextStyleOption>,
    pub item_width: Option<f64>,
    pub item_height: Option<f64>,
    pub symbol_size: Option<LenientNumber>,
    pub icon: Option<String>,
    pub align: Option<String>,
    pub item_gap: Option<f64>,
    pub formatter: Option<String>,
    pub selected_mode: Option<LenientBoolOrString>,
    pub inactive_color: Option<ColorOption>,
    pub inactive_border_color: Option<ColorOption>,
    pub inactive_border_width: Option<f64>,
    pub selector: Option<bool>,
    pub selector_label: Option<String>,
    pub selector_position: Option<String>,
    pub selector_item_gap: Option<f64>,
    pub selector_button_gap: Option<f64>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub border_radius: Option<LenientNumber>,
    pub background_color: Option<ColorOption>,
    pub padding: Option<LenientPadding>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub scroll_data_index: Option<usize>,
    pub page_button_item_gap: Option<f64>,
    pub page_button_gap: Option<f64>,
    pub page_icon_color: Option<String>,
    pub page_icon_inactive_color: Option<String>,
    pub page_icon_size: Option<OneOrMany<f64>>,
    pub animation_duration_update: Option<f64>,
    pub type_: Option<String>,
    pub selected: Option<std::collections::HashMap<String, bool>>,
}

impl LegendOption {
    /// 设置图例数据项（字符串形式）。
    pub fn data(mut self, data: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data = Some(
            data.into_iter()
                .map(|s| LegendDataItem::Str(s.into()))
                .collect(),
        );
        self
    }

    pub fn left(mut self, left: PositionOption) -> Self {
        self.left = Some(left);
        self
    }

    pub fn top(mut self, top: PositionOption) -> Self {
        self.top = Some(top);
        self
    }

    pub fn orient(mut self, orient: Orient) -> Self {
        self.orient = Some(orient);
        self
    }

    pub fn show(mut self, show: bool) -> Self {
        self.show = Some(show);
        self
    }
}

/// Grid region configuration for multi-layout charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridOption {
    pub left: Option<PositionOption>,
    pub right: Option<PositionOption>,
    pub top: Option<PositionOption>,
    pub bottom: Option<PositionOption>,
    pub contain_label: Option<bool>,
    pub background_color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub show: Option<bool>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub tooltip: Option<TooltipOption>,
}

impl Default for GridOption {
    fn default() -> Self {
        Self {
            left: Some(PositionOption::percent(10.0)),
            right: Some(PositionOption::percent(10.0)),
            top: Some(PositionOption::percent(15.0)),
            bottom: Some(PositionOption::percent(15.0)),
            contain_label: Some(true),
            background_color: None,
            border_color: None,
            border_width: None,
            shadow_blur: None,
            shadow_color: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            show: Some(true),
            z: None,
            zlevel: None,
            tooltip: None,
        }
    }
}

impl GridOption {
    pub fn left(mut self, left: PositionOption) -> Self {
        self.left = Some(left);
        self
    }

    pub fn right(mut self, right: PositionOption) -> Self {
        self.right = Some(right);
        self
    }

    pub fn top(mut self, top: PositionOption) -> Self {
        self.top = Some(top);
        self
    }

    pub fn bottom(mut self, bottom: PositionOption) -> Self {
        self.bottom = Some(bottom);
        self
    }

    pub fn contain_label(mut self, contain: bool) -> Self {
        self.contain_label = Some(contain);
        self
    }
}

/// Axis configuration (category, value, or time).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisOption {
    #[serde(rename = "type")]
    pub axis_type: Option<AxisType>,
    pub data: Option<LenientAxisData>,
    pub name: Option<String>,
    pub name_location: Option<NameLocation>,
    pub name_text_style: Option<TextStyleOption>,
    pub name_gap: Option<f64>,
    pub name_rotate: Option<f64>,
    pub axis_label: Option<AxisLabelOption>,
    pub axis_line: Option<AxisLineOption>,
    pub axis_tick: Option<AxisTickOption>,
    pub split_line: Option<SplitLineOption>,
    pub split_area: Option<SplitAreaOption>,
    pub min: Option<LenientAxisLimit>,
    pub max: Option<LenientAxisLimit>,
    pub min_interval: Option<f64>,
    pub max_interval: Option<f64>,
    pub interval: Option<f64>,
    pub boundary_gap: Option<LenientBoundaryGap>,
    pub position: Option<AxisPosition>,
    pub grid_index: Option<usize>,
    pub align_ticks: Option<bool>,
    pub axis_pointer: Option<AxisPointerOption>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub inverse: Option<bool>,
    pub log_base: Option<f64>,
    pub silent: Option<bool>,
    pub trigger_event: Option<bool>,
}

impl Default for AxisOption {
    fn default() -> Self {
        Self {
            axis_type: Some(AxisType::Category),
            data: None,
            name: None,
            name_location: Some(NameLocation::End),
            name_text_style: None,
            name_gap: None,
            name_rotate: None,
            axis_label: None,
            axis_line: None,
            axis_tick: None,
            grid_index: None,
            split_line: None,
            split_area: None,
            min: None,
            max: None,
            min_interval: None,
            max_interval: None,
            interval: None,
            boundary_gap: Some(LenientBoundaryGap::Bool(true)),
            position: None,
            align_ticks: None,
            axis_pointer: None,
            z: None,
            zlevel: None,
            inverse: None,
            log_base: None,
            silent: None,
            trigger_event: None,
        }
    }
}

impl AxisOption {
    /// 创建类目轴
    pub fn category() -> Self {
        Self {
            axis_type: Some(AxisType::Category),
            ..Default::default()
        }
    }

    /// 创建数值轴
    pub fn value() -> Self {
        Self {
            axis_type: Some(AxisType::Value),
            ..Default::default()
        }
    }

    pub fn data(mut self, data: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data = Some(LenientAxisData(data.into_iter().map(Into::into).collect()));
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(LenientAxisLimit::Value(min));
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(LenientAxisLimit::Value(max));
        self
    }

    pub fn position(mut self, position: AxisPosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn grid_index(mut self, index: usize) -> Self {
        self.grid_index = Some(index);
        self
    }

    pub fn boundary_gap(mut self, gap: bool) -> Self {
        self.boundary_gap = Some(LenientBoundaryGap::Bool(gap));
        self
    }

    pub fn axis_label(mut self, label: AxisLabelOption) -> Self {
        self.axis_label = Some(label);
        self
    }

    pub fn split_line(mut self, split: SplitLineOption) -> Self {
        self.split_line = Some(split);
        self
    }
}

/// Axis label configuration (axis label style).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisLabelOption {
    pub show: Option<bool>,
    pub rotate: Option<f64>,
    pub formatter: Option<String>,
    /// 标签颜色，可以是单色或颜色数组（如 `["#333","#666"]`）
    pub color: Option<OneOrMany<ColorOption>>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub align: Option<LabelAlign>,
    pub vertical_align: Option<LabelVerticalAlign>,
    pub margin: Option<f64>,
    /// 标签显示间隔，可以是数字或字符串 `"auto"`
    pub interval: Option<IntervalOption>,
    pub inside: Option<bool>,
    pub show_min_label: Option<bool>,
    pub show_max_label: Option<bool>,
    pub hide_overlap: Option<bool>,
    pub color_func: Option<String>,
    pub font_style: Option<String>,
    pub line_height: Option<f64>,
    pub background_color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub border_radius: Option<LenientNumber>,
    pub padding: Option<LenientPadding>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub overflow: Option<String>,
    pub ellipsis: Option<String>,
    pub rich: Option<serde_json::Value>,
}

impl Default for AxisLabelOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            rotate: None,
            formatter: None,
            color: None,
            font_size: Some(12.0),
            font_family: None,
            font_weight: None,
            align: None,
            vertical_align: None,
            margin: None,
            interval: None,
            inside: None,
            show_min_label: None,
            show_max_label: None,
            hide_overlap: None,
            color_func: None,
            font_style: None,
            line_height: None,
            background_color: None,
            border_color: None,
            border_width: None,
            border_radius: None,
            padding: None,
            shadow_blur: None,
            shadow_color: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            width: None,
            height: None,
            overflow: None,
            ellipsis: None,
            rich: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisLineOption {
    pub show: Option<bool>,
    pub line_style: Option<LineStyleOption>,
    pub on_zero: Option<bool>,
    pub on_zero_axis_index: Option<usize>,
    /// 轴线两端的箭头，可以是字符串或字符串数组（如 `"none"` 或 `["none","arrow"]`）
    pub symbol: Option<OneOrMany<String>>,
    pub symbol_size: Option<Vec<f64>>,
    pub symbol_offset: Option<Vec<f64>>,
}

impl Default for AxisLineOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            line_style: None,
            on_zero: None,
            on_zero_axis_index: None,
            symbol: None,
            symbol_size: None,
            symbol_offset: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisTickOption {
    pub show: Option<bool>,
    pub align_with_label: Option<bool>,
    /// 刻度显示间隔，可以是数字或字符串 `"auto"`
    pub interval: Option<IntervalOption>,
    pub inside: Option<bool>,
    pub length: Option<f64>,
    pub line_style: Option<LineStyleOption>,
}

impl Default for AxisTickOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            align_with_label: None,
            interval: None,
            inside: None,
            length: None,
            line_style: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitLineOption {
    pub show: Option<bool>,
    pub interval: Option<f64>,
    pub length: Option<f64>,
    pub line_style: Option<LineStyleOption>,
}

impl Default for SplitLineOption {
    fn default() -> Self {
        Self {
            show: Some(false),
            interval: None,
            length: None,
            line_style: None,
        }
    }
}

/// 兼容的线条颜色类型：支持单色或仪表盘分段颜色（如 [[0.3,"#67e0e3"],[1,"#fd666d"]]）
#[derive(Debug, Clone)]
pub enum LenientLineColor {
    Single(ColorOption),
    Segments(Vec<(LenientNumber, ColorOption)>),
}

impl Serialize for LenientLineColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LenientLineColor::Single(c) => c.serialize(serializer),
            LenientLineColor::Segments(segs) => {
                let arr: Vec<(LenientNumber, ColorOption)> = segs.clone();
                arr.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for LenientLineColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LineColorVisitor;
        impl<'de> Visitor<'de> for LineColorVisitor {
            type Value = LenientLineColor;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(
                    "a color string/object, or an array of [number, color] segment pairs, or a color array",
                )
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let c = ColorOption::deserialize(de::IntoDeserializer::<E>::into_deserializer(v))?;
                Ok(LenientLineColor::Single(c))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }
            fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                let c = ColorOption::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(LenientLineColor::Single(c))
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                // 先尝试解析为分段：[[n, color], ...]
                // 收集所有元素先
                let mut elems: Vec<serde_json::Value> = Vec::new();
                while let Some(e) = seq.next_element::<serde_json::Value>()? {
                    elems.push(e);
                }
                if elems.is_empty() {
                    return Ok(LenientLineColor::Segments(Vec::new()));
                }
                // 看第一个元素是否是数组（表明是分段格式）
                let first_is_array = matches!(elems.first(), Some(serde_json::Value::Array(_)));
                if first_is_array {
                    let mut segs = Vec::with_capacity(elems.len());
                    for e in &elems {
                        if let serde_json::Value::Array(pair) = e
                            && pair.len() >= 2
                        {
                            let n = LenientNumber::deserialize(&pair[0])
                                .unwrap_or(LenientNumber::Number(0.0));
                            let c = ColorOption::deserialize(&pair[1]).unwrap_or_default();
                            segs.push((n, c));
                        }
                    }
                    Ok(LenientLineColor::Segments(segs))
                } else {
                    // 单色数组（区域配色），取第一个
                    if let Some(first) = elems.into_iter().next()
                        && let Ok(c) = ColorOption::deserialize(first)
                    {
                        return Ok(LenientLineColor::Single(c));
                    }
                    Ok(LenientLineColor::Single(ColorOption::default()))
                }
            }
        }
        deserializer.deserialize_any(LineColorVisitor)
    }
}

/// Line style configuration (color, width, dash, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineStyleOption {
    pub color: Option<LenientLineColor>,
    pub width: Option<LenientNumber>,
    #[serde(rename = "type")]
    pub line_type: Option<LineType>,
    pub opacity: Option<f64>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub curveness: Option<f64>,
    pub cap: Option<String>,
    pub join: Option<String>,
    pub miter_limit: Option<f64>,
}

impl Default for LineStyleOption {
    fn default() -> Self {
        Self {
            color: None,
            width: Some(LenientNumber::Number(2.0)),
            line_type: Some(LineType::Solid),
            opacity: None,
            shadow_blur: None,
            shadow_color: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            curveness: None,
            cap: None,
            join: None,
            miter_limit: None,
        }
    }
}

/// Global text style applied when no per-item style is specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStyleOption {
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub align: Option<TextAlignOption>,
    pub vertical_align: Option<LabelVerticalAlign>,
    pub line_height: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub text_border_color: Option<ColorOption>,
    pub text_border_width: Option<f64>,
    pub text_shadow_blur: Option<f64>,
    pub text_shadow_color: Option<ColorOption>,
    pub text_shadow_offset_x: Option<f64>,
    pub text_shadow_offset_y: Option<f64>,
    pub overflow: Option<String>,
    pub ellipsis: Option<String>,
    pub rich: Option<serde_json::Value>,
    pub padding: Option<LenientPadding>,
    pub background_color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub border_radius: Option<LenientNumber>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub tag: Option<String>,
}

impl Default for TextStyleOption {
    fn default() -> Self {
        Self {
            color: None,
            font_size: Some(12.0),
            font_family: None,
            font_weight: Some(FontWeight::Named(FontWeightNamed::Normal)),
            font_style: None,
            align: None,
            vertical_align: None,
            line_height: None,
            width: None,
            height: None,
            text_border_color: None,
            text_border_width: None,
            text_shadow_blur: None,
            text_shadow_color: None,
            text_shadow_offset_x: None,
            text_shadow_offset_y: None,
            overflow: None,
            ellipsis: None,
            rich: None,
            padding: None,
            background_color: None,
            border_color: None,
            border_width: None,
            border_radius: None,
            shadow_blur: None,
            shadow_color: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            tag: None,
        }
    }
}

impl TextStyleOption {
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn color(mut self, color: ColorOption) -> Self {
        self.color = Some(color);
        self
    }

    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    pub fn align(mut self, align: TextAlignOption) -> Self {
        self.align = Some(align);
        self
    }
}

/// 表格 header 配置：支持字符串数组（列名）或完整配置对象
#[derive(Debug, Clone)]
pub enum LenientTableHeader {
    Columns(Vec<String>),
    Config(Box<TableHeaderOption>),
}

impl Serialize for LenientTableHeader {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LenientTableHeader::Columns(v) => v.serialize(serializer),
            LenientTableHeader::Config(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for LenientTableHeader {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct HeaderVisitor;
        impl<'de> Visitor<'de> for HeaderVisitor {
            type Value = LenientTableHeader;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string array (column names) or a header config object")
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
                let items: Vec<serde_json::Value> =
                    Vec::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                let mut cols = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        serde_json::Value::String(s) => cols.push(s),
                        other => cols.push(other.to_string()),
                    }
                }
                Ok(LenientTableHeader::Columns(cols))
            }
            fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                let cfg =
                    TableHeaderOption::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(LenientTableHeader::Config(Box::new(cfg)))
            }
        }
        deserializer.deserialize_any(HeaderVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSeriesOption {
    pub name: Option<String>,
    pub data: Option<Vec<serde_json::Value>>,
    pub columns: Option<Vec<String>>,
    pub header: Option<LenientTableHeader>,
    pub body: Option<TableBodyOption>,
    pub row_style: Option<TableRowStyleOption>,
    pub cell_style: Option<TableCellStyleOption>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub left: Option<f64>,
    pub top: Option<f64>,
    /// 表格所属的 grid 索引，默认 0
    pub grid_index: Option<usize>,
    /// 是否自动调整 grid 大小以适应表格内容
    pub auto_fit_grid: Option<bool>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub silent: Option<bool>,
    pub data_group_id: Option<String>,
    pub page_size: Option<usize>,
    pub page_button_position: Option<String>,
}

impl Default for TableSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: None,
            columns: None,
            header: Some(LenientTableHeader::Config(Box::default())),
            body: Some(TableBodyOption::default()),
            row_style: Some(TableRowStyleOption::default()),
            cell_style: Some(TableCellStyleOption::default()),
            width: None,
            height: None,
            left: None,
            top: None,
            grid_index: Some(0),
            auto_fit_grid: Some(false),
            z: None,
            zlevel: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            silent: None,
            data_group_id: None,
            page_size: None,
            page_button_position: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableHeaderOption {
    pub show: Option<bool>,
    pub height: Option<f64>,
    pub style: Option<TextStyleOption>,
    pub background_color: Option<ColorOption>,
    pub align: Option<TextAlignOption>,
}

impl Default for TableHeaderOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            height: Some(40.0),
            style: Some(TextStyleOption {
                color: Some(ColorOption::new(51, 51, 51)),
                font_size: Some(14.0),
                font_family: Some("Arial, sans-serif".to_string()),
                font_weight: Some(FontWeight::Named(FontWeightNamed::Bold)),
                font_style: None,
                align: None,
                vertical_align: None,
                ..Default::default()
            }),
            background_color: Some(ColorOption::new(248, 248, 248)),
            align: Some(TextAlignOption::Center),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBodyOption {
    pub show: Option<bool>,
    pub style: Option<TextStyleOption>,
    pub row_height: Option<f64>,
    pub even_row_background_color: Option<ColorOption>,
    pub odd_row_background_color: Option<ColorOption>,
    pub align: Option<TextAlignOption>,
}

impl Default for TableBodyOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            style: Some(TextStyleOption {
                color: Some(ColorOption::new(51, 51, 51)),
                font_size: Some(12.0),
                font_family: Some("Arial, sans-serif".to_string()),
                font_weight: Some(FontWeight::Named(FontWeightNamed::Normal)),
                font_style: None,
                align: None,
                vertical_align: None,
                ..Default::default()
            }),
            row_height: Some(32.0),
            even_row_background_color: Some(ColorOption::new(255, 255, 255)),
            odd_row_background_color: Some(ColorOption::new(250, 250, 250)),
            align: Some(TextAlignOption::Center),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRowStyleOption {
    pub border_color: Option<ColorOption>,
    pub border_width: Option<f64>,
}

impl Default for TableRowStyleOption {
    fn default() -> Self {
        Self {
            border_color: Some(ColorOption::new(220, 220, 220)),
            border_width: Some(1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCellStyleOption {
    pub padding: Option<LenientPadding>,
}

impl Default for TableCellStyleOption {
    fn default() -> Self {
        Self {
            padding: Some(LenientPadding::Single(8.0)),
        }
    }
}

/// Abstract series type that dispatches to concrete variants (bar, line, pie, etc.).
///
/// 未识别的 series 类型（如 `funnel`、`treemap`、`graph` 等）会被解析为
/// [`SeriesOption::Unknown`]，下游 pipeline 会跳过这些系列而不报错，
/// 以保证任意 LLM 输出的 ECharts JSON 都能成功解析。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SeriesOption {
    #[serde(rename = "line")]
    Line(LineSeriesOption),
    #[serde(rename = "bar")]
    Bar(BarSeriesOption),
    #[serde(rename = "candlestick")]
    Candlestick(CandlestickSeriesOption),
    #[serde(rename = "boxplot")]
    Boxplot(BoxplotSeriesOption),
    #[serde(rename = "heatmap")]
    Heatmap(HeatmapSeriesOption),
    #[serde(rename = "pie")]
    Pie(PieSeriesOption),
    #[serde(rename = "scatter")]
    Scatter(ScatterSeriesOption),
    #[serde(rename = "radar")]
    Radar(RadarSeriesOption),
    #[serde(rename = "polarBar")]
    PolarBar(PolarBarSeriesOption),
    #[serde(rename = "polarScatter")]
    PolarScatter(PolarScatterSeriesOption),
    #[serde(rename = "bubble")]
    Bubble(BubbleSeriesOption),
    #[serde(rename = "gauge")]
    Gauge(GaugeSeriesOption),
    #[serde(rename = "table")]
    Table(TableSeriesOption),
    /// 未识别的 series 类型。解析时不报错，渲染时跳过。
    #[serde(other)]
    Unknown,
}

/// Line series configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineSeriesOption {
    pub name: Option<String>,
    #[serde(default)]
    pub data: Vec<DataPoint>,
    pub stack: Option<String>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    /// 坐标系类型（`"polar"` / `"cartesian2d"` 等）；`"polar"` 时自动路由到极坐标渲染
    pub coordinate_system: Option<String>,
    pub smooth: Option<bool>,
    pub symbol: Option<SymbolType>,
    pub symbol_size: Option<LenientNumber>,
    pub line_style: Option<LineStyleOption>,
    pub item_style: Option<ItemStyleOption>,
    pub area_style: Option<AreaStyleOption>,
    pub label: Option<LabelOption>,
    pub step: Option<LenientStep>,
    pub connect_nulls: Option<bool>,
    pub show_symbol: Option<bool>,
    pub show_all_symbol: Option<bool>,
    pub legend_hover_link: Option<bool>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub sampling: Option<SamplingOption>,
    #[serde(default)]
    pub dataset_index: Option<usize>,
    pub series_layout_by: Option<String>,
    pub data_group_id: Option<String>,
    pub silent: Option<bool>,
}

impl Default for LineSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            stack: None,
            x_axis_index: None,
            y_axis_index: None,
            grid_index: None,
            coordinate_system: None,
            smooth: Some(false),
            symbol: Some(SymbolType::Circle),
            symbol_size: Some(LenientNumber::Number(4.0)),
            line_style: None,
            item_style: None,
            area_style: None,
            label: None,
            step: None,
            connect_nulls: None,
            show_symbol: None,
            show_all_symbol: None,
            legend_hover_link: Some(true),
            z: None,
            zlevel: None,
            encode: None,
            mark_point: None,
            mark_line: None,
            mark_area: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            sampling: None,
            dataset_index: None,
            series_layout_by: None,
            data_group_id: None,
            silent: None,
        }
    }
}

impl LineSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn smooth(mut self, smooth: bool) -> Self {
        self.smooth = Some(smooth);
        self
    }

    pub fn stack(mut self, stack: impl Into<String>) -> Self {
        self.stack = Some(stack.into());
        self
    }

    pub fn area_style(mut self, style: AreaStyleOption) -> Self {
        self.area_style = Some(style);
        self
    }

    pub fn sampling(mut self, sampling: SamplingOption) -> Self {
        self.sampling = Some(sampling);
        self
    }
}

/// Bar/column series configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct BarSeriesOption {
    pub name: Option<String>,
    #[serde(default)]
    pub data: Vec<DataPoint>,
    pub stack: Option<String>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    /// 坐标系类型（`"polar"` / `"cartesian2d"` 等）；`"polar"` 时自动路由到极坐标渲染
    pub coordinate_system: Option<String>,
    pub bar_width: Option<LenientBarSize>,
    pub bar_max_width: Option<LenientBarSize>,
    pub bar_min_width: Option<LenientBarSize>,
    pub bar_gap: Option<LenientBarSize>,
    pub bar_category_gap: Option<LenientBarSize>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    /// 分组索引，自动分组时无需设置
    pub group_index: Option<usize>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub sampling: Option<SamplingOption>,
    pub dataset_index: Option<usize>,
    pub series_layout_by: Option<String>,
    pub data_group_id: Option<String>,
    pub silent: Option<bool>,
    pub round_cap: Option<bool>,
    pub show_background: Option<bool>,
    pub background_style: Option<ItemStyleOption>,
}

impl BarSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn stack(mut self, stack: impl Into<String>) -> Self {
        self.stack = Some(stack.into());
        self
    }

    pub fn sampling(mut self, sampling: SamplingOption) -> Self {
        self.sampling = Some(sampling);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct CandlestickSeriesOption {
    pub name: Option<String>,
    pub data: Vec<CandlestickDataPoint>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub item_style: Option<CandlestickItemStyleOption>,
    pub label: Option<LabelOption>,
    pub bar_width: Option<LenientBarSize>,
    pub bar_max_width: Option<LenientBarSize>,
    pub bar_min_width: Option<LenientBarSize>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub sampling: Option<SamplingOption>,
    pub dataset_index: Option<usize>,
    pub series_layout_by: Option<String>,
    pub data_group_id: Option<String>,
    pub silent: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandlestickDataPoint {
    pub open: f64,
    pub close: f64,
    pub low: f64,
    pub high: f64,
    pub name: Option<String>,
}

impl<'de> Deserialize<'de> for CandlestickDataPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            open: f64,
            close: f64,
            low: f64,
            high: f64,
            name: Option<String>,
        }

        struct CandlestickVisitor;

        impl<'de> de::Visitor<'de> for CandlestickVisitor {
            type Value = CandlestickDataPoint;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array [open, close, low, high] or an object")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let open = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &"at least 4 elements"))?;
                let close = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| de::Error::invalid_length(1, &"at least 4 elements"))?;
                let low = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| de::Error::invalid_length(2, &"at least 4 elements"))?;
                let high = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| de::Error::invalid_length(3, &"at least 4 elements"))?;

                Ok(CandlestickDataPoint {
                    open,
                    close,
                    low,
                    high,
                    name: None,
                })
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let helper = Helper::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(CandlestickDataPoint {
                    open: helper.open,
                    close: helper.close,
                    low: helper.low,
                    high: helper.high,
                    name: helper.name,
                })
            }
        }

        deserializer.deserialize_any(CandlestickVisitor)
    }
}

impl CandlestickDataPoint {
    pub fn new(open: f64, close: f64, low: f64, high: f64) -> Self {
        Self {
            open,
            close,
            low,
            high,
            name: None,
        }
    }

    pub fn is_up(&self) -> bool {
        self.close >= self.open
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct CandlestickItemStyleOption {
    pub color: Option<ColorOption>,
    pub color0: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_color0: Option<ColorOption>,
}

// ═══════════════════════════════════════════════════════════════════
// BoxplotSeriesOption - 箱线图系列
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct BoxplotSeriesOption {
    pub name: Option<String>,
    pub data: Vec<BoxplotDataPoint>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub item_style: Option<BoxplotItemStyleOption>,
    pub label: Option<LabelOption>,
    pub bar_width: Option<LenientBarSize>,
    pub bar_max_width: Option<LenientBarSize>,
    pub bar_min_width: Option<LenientBarSize>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub sampling: Option<SamplingOption>,
    pub dataset_index: Option<usize>,
    pub series_layout_by: Option<String>,
    pub data_group_id: Option<String>,
    pub silent: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoxplotDataPoint {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub name: Option<String>,
}

impl BoxplotDataPoint {
    pub fn new(min: f64, q1: f64, median: f64, q3: f64, max: f64) -> Self {
        Self {
            min,
            q1,
            median,
            q3,
            max,
            name: None,
        }
    }
}

impl From<[f64; 5]> for BoxplotDataPoint {
    fn from(values: [f64; 5]) -> Self {
        BoxplotDataPoint::new(values[0], values[1], values[2], values[3], values[4])
    }
}

impl<'de> Deserialize<'de> for BoxplotDataPoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BoxplotDataPointVisitor;

        impl<'de> Visitor<'de> for BoxplotDataPointVisitor {
            type Value = BoxplotDataPoint;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a [min, q1, median, q3, max] array or {value: [...], name?} object")
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                use serde::de::Error;
                let mut values = [0.0; 5];
                for (i, slot) in values.iter_mut().enumerate() {
                    *slot = seq.next_element::<f64>()?.ok_or_else(|| {
                        A::Error::custom(format!("expected 5 numbers, got only {}", i))
                    })?;
                }
                // 忽略多余元素
                while seq.next_element::<serde_json::Value>()?.is_some() {}
                Ok(BoxplotDataPoint::new(
                    values[0], values[1], values[2], values[3], values[4],
                ))
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                use serde::de::Error;
                let mut name: Option<String> = None;
                let mut value: Option<serde_json::Value> = None;

                while let Some(k) = map.next_key::<String>()? {
                    match k.as_str() {
                        "name" => name = Some(map.next_value()?),
                        "value" => value = Some(map.next_value()?),
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let arr = match value {
                    Some(serde_json::Value::Array(arr)) => arr,
                    Some(other) => {
                        return Err(A::Error::custom(format!(
                            "boxplot value must be an array, got {:?}",
                            other
                        )));
                    }
                    None => {
                        return Err(A::Error::custom("boxplot data object missing value field"));
                    }
                };

                if arr.len() < 5 {
                    return Err(A::Error::custom(format!(
                        "boxplot value array expected 5 numbers, got {}",
                        arr.len()
                    )));
                }

                let nums: Vec<f64> = arr
                    .into_iter()
                    .take(5)
                    .map(|v| {
                        v.as_f64().ok_or_else(|| {
                            A::Error::custom("boxplot value array element must be a number")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(BoxplotDataPoint {
                    min: nums[0],
                    q1: nums[1],
                    median: nums[2],
                    q3: nums[3],
                    max: nums[4],
                    name,
                })
            }
        }

        deserializer.deserialize_any(BoxplotDataPointVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct BoxplotItemStyleOption {
    pub color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PieSeriesOption {
    pub name: Option<String>,
    #[serde(default)]
    pub data: Vec<DataPoint>,
    pub radius: Option<SingleOrArray<LenientNumber>>,
    pub center: Option<Vec<LenientNumber>>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub label_line: Option<LabelLineOption>,
    pub rose_type: Option<String>,
    pub selected_mode: Option<LenientBoolOrString>,
    pub selected_offset: Option<f64>,
    pub clockwise: Option<bool>,
    pub start_angle: Option<f64>,
    pub min_angle: Option<f64>,
    pub avoid_label_overlap: Option<bool>,
    pub still_show_zero_sum: Option<bool>,
    pub percent_precision: Option<usize>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub grid_index: Option<usize>,
    pub dataset_index: Option<usize>,
    pub series_layout_by: Option<String>,
    pub data_group_id: Option<String>,
    pub silent: Option<bool>,
}

impl Default for PieSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            radius: Some(SingleOrArray::Array(vec![
                LenientNumber::Percent(0.0),
                LenientNumber::Percent(75.0),
            ])),
            center: Some(vec![
                LenientNumber::Percent(50.0),
                LenientNumber::Percent(50.0),
            ]),
            item_style: None,
            label: None,
            label_line: None,
            rose_type: None,
            selected_mode: None,
            selected_offset: None,
            clockwise: Some(true),
            start_angle: Some(90.0),
            min_angle: None,
            avoid_label_overlap: Some(true),
            still_show_zero_sum: Some(true),
            percent_precision: None,
            z: None,
            zlevel: None,
            encode: None,
            mark_point: None,
            mark_line: None,
            mark_area: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            grid_index: None,
            dataset_index: None,
            series_layout_by: None,
            data_group_id: None,
            silent: None,
        }
    }
}

impl PieSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelLineOption {
    pub show: Option<bool>,
    pub length: Option<f64>,
    pub length2: Option<f64>,
    pub smooth: Option<bool>,
    pub min_turn_angle: Option<f64>,
    pub line_style: Option<LineStyleOption>,
}

impl Default for LabelLineOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            length: None,
            length2: None,
            smooth: None,
            min_turn_angle: None,
            line_style: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Heatmap — 热力图系列
// ═══════════════════════════════════════════════════════════════════

/// Heatmap series configuration.
///
/// 数据格式与 ECharts 一致：每项为 `[x, y, value]` 三元组，
/// 其中 x/y 通常是 category 轴的索引，value 用于 visualMap 颜色映射。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct HeatmapSeriesOption {
    pub name: Option<String>,
    #[serde(default)]
    pub data: Vec<HeatmapDataPoint>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub coordinate_system: Option<String>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub sampling: Option<SamplingOption>,
    pub dataset_index: Option<usize>,
    pub series_layout_by: Option<String>,
    pub data_group_id: Option<String>,
    pub silent: Option<bool>,
    pub progressive: Option<usize>,
}

impl HeatmapSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<HeatmapDataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

/// 单个热力图数据点：`[x, y, value]`。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapDataPoint {
    pub x: f64,
    pub y: f64,
    pub value: f64,
    pub name: Option<String>,
}

impl HeatmapDataPoint {
    pub fn new(x: f64, y: f64, value: f64) -> Self {
        Self {
            x,
            y,
            value,
            name: None,
        }
    }
}

impl From<(f64, f64, f64)> for HeatmapDataPoint {
    fn from((x, y, value): (f64, f64, f64)) -> Self {
        Self::new(x, y, value)
    }
}

impl From<[f64; 3]> for HeatmapDataPoint {
    fn from(values: [f64; 3]) -> Self {
        Self::new(values[0], values[1], values[2])
    }
}

impl<'de> Deserialize<'de> for HeatmapDataPoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct HeatmapDataPointVisitor;

        impl<'de> Visitor<'de> for HeatmapDataPointVisitor {
            type Value = HeatmapDataPoint;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a [x, y, value] array or {value: [x, y, value], name?} object")
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                use serde::de::Error;
                let mut elems: Vec<serde_json::Value> = Vec::new();
                while let Some(e) = seq.next_element::<serde_json::Value>()? {
                    elems.push(e);
                }
                if elems.is_empty() {
                    return Err(A::Error::custom("heatmap data array must not be empty"));
                }

                // 容错解析：
                // - [x, y, value] 标准形式
                // - [x, y] / [x] 缺失值补 0
                // - [date/name, value]（如 calendar 热力图）降级为 (0, 0, value)
                let mut values = [0.0; 3];
                if elems[0].is_string() {
                    values[2] = elems.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                } else {
                    for (i, slot) in values.iter_mut().enumerate() {
                        if let Some(v) = elems.get(i)
                            && let Some(n) = v.as_f64()
                        {
                            *slot = n;
                        }
                    }
                }
                Ok(HeatmapDataPoint::new(values[0], values[1], values[2]))
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                use serde::de::Error;
                let mut name: Option<String> = None;
                let mut value: Option<serde_json::Value> = None;
                let mut x: Option<f64> = None;
                let mut y: Option<f64> = None;

                while let Some(k) = map.next_key::<String>()? {
                    match k.as_str() {
                        "name" => name = Some(map.next_value()?),
                        "value" => value = Some(map.next_value()?),
                        "x" => x = Some(map.next_value()?),
                        "y" => y = Some(map.next_value()?),
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let (vx, vy, vv) = match value {
                    Some(serde_json::Value::Array(arr)) => {
                        let mut values = [0.0; 3];
                        if arr.first().is_some_and(|v| v.is_string()) {
                            values[2] = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        } else {
                            for (i, slot) in values.iter_mut().enumerate() {
                                if let Some(v) = arr.get(i)
                                    && let Some(n) = v.as_f64()
                                {
                                    *slot = n;
                                }
                            }
                        }
                        (values[0], values[1], values[2])
                    }
                    Some(serde_json::Value::Number(n)) => (0.0, 0.0, n.as_f64().unwrap_or(0.0)),
                    Some(other) => {
                        return Err(A::Error::custom(format!(
                            "heatmap value must be an array or number, got {:?}",
                            other
                        )));
                    }
                    None => (0.0, 0.0, 0.0),
                };

                Ok(HeatmapDataPoint {
                    x: x.unwrap_or(vx),
                    y: y.unwrap_or(vy),
                    value: vv,
                    name,
                })
            }
        }

        deserializer.deserialize_any(HeatmapDataPointVisitor)
    }
}

/// Scatter series configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatterSeriesOption {
    pub name: Option<String>,
    #[serde(default)]
    pub data: Vec<DataPoint>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    /// 坐标系类型（`"polar"` / `"cartesian2d"` 等）；`"polar"` 时自动路由到极坐标渲染
    pub coordinate_system: Option<String>,
    pub symbol: Option<SymbolType>,
    pub symbol_size: Option<SingleOrArray<LenientNumber>>,
    pub symbol_rotate: Option<f64>,
    pub symbol_keep_aspect: Option<bool>,
    pub symbol_offset: Option<Vec<f64>>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub sampling: Option<SamplingOption>,
    pub dataset_index: Option<usize>,
    pub series_layout_by: Option<String>,
    pub data_group_id: Option<String>,
    pub silent: Option<bool>,
    pub large: Option<bool>,
    pub large_threshold: Option<usize>,
}

impl ScatterSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn sampling(mut self, sampling: SamplingOption) -> Self {
        self.sampling = Some(sampling);
        self
    }
}

impl Default for ScatterSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            x_axis_index: None,
            y_axis_index: None,
            grid_index: None,
            coordinate_system: None,
            symbol: Some(SymbolType::Circle),
            symbol_size: Some(SingleOrArray::Single(LenientNumber::Number(10.0))),
            symbol_rotate: None,
            symbol_keep_aspect: None,
            symbol_offset: None,
            item_style: None,
            label: None,
            z: None,
            zlevel: None,
            encode: None,
            mark_point: None,
            mark_line: None,
            mark_area: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            sampling: None,
            dataset_index: None,
            series_layout_by: None,
            data_group_id: None,
            silent: None,
            large: None,
            large_threshold: None,
        }
    }
}

// ============================================================
// RadarOption - 雷达图配置
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct RadarIndicatorOption {
    pub name: Option<String>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarNameOption {
    pub show: Option<bool>,
    pub formatter: Option<String>,
    pub text_style: Option<TextStyleOption>,
}

impl Default for RadarNameOption {
    fn default() -> Self {
        Self {
            show: Some(true),
            formatter: None,
            text_style: None,
        }
    }
}

/// 单个值或数组（泛型版本），用于 ECharts 中接受单值或数组的字段。
///
/// 使用 untagged enum 自动处理单对象或数组，支持 T 为任意可反序列化类型
/// （包括结构体、字符串、数字等）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn as_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        match self {
            OneOrMany::One(v) => vec![v.clone()],
            OneOrMany::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarOption {
    pub indicator: Option<Vec<RadarIndicatorOption>>,
    pub center: Option<Vec<LenientNumber>>,
    pub radius: Option<SingleOrArray<LenientNumber>>,
    pub split_number: Option<usize>,
    pub name: Option<RadarNameOption>,
    pub shape: Option<String>,
    pub scale: Option<bool>,
    pub silent: Option<bool>,
    pub trigger_event: Option<bool>,
    pub axis_line: Option<AxisLineOption>,
    pub axis_tick: Option<AxisTickOption>,
    pub axis_label: Option<AxisLabelOption>,
    pub split_line: Option<SplitLineOption>,
    pub split_area: Option<SplitAreaOption>,
    pub split_area_color: Option<Vec<ColorOption>>,
    pub indicator_font_size: Option<f64>,
    pub radius_axis_index: Option<usize>,
    pub angle_axis_index: Option<usize>,
}

impl Default for RadarOption {
    fn default() -> Self {
        Self {
            indicator: None,
            center: Some(vec![
                LenientNumber::Percent(50.0),
                LenientNumber::Percent(50.0),
            ]),
            radius: Some(SingleOrArray::Array(vec![
                LenientNumber::Percent(0.0),
                LenientNumber::Percent(75.0),
            ])),
            split_number: Some(5),
            name: None,
            shape: None,
            scale: None,
            silent: None,
            trigger_event: None,
            axis_line: None,
            axis_tick: None,
            axis_label: None,
            split_line: None,
            split_area: None,
            split_area_color: None,
            indicator_font_size: None,
            radius_axis_index: None,
            angle_axis_index: None,
        }
    }
}

// ============================================================
// RadarSeriesOption - 雷达系列
// ============================================================

/// Radar series configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarSeriesOption {
    pub name: Option<String>,
    pub data: Vec<RadarDataOption>,
    pub radar_index: Option<usize>,
    pub item_style: Option<ItemStyleOption>,
    pub line_style: Option<LineStyleOption>,
    pub area_style: Option<AreaStyleOption>,
    pub label: Option<LabelOption>,
    pub symbol: Option<SymbolType>,
    pub symbol_size: Option<LenientNumber>,
    pub symbol_rotate: Option<f64>,
    pub symbol_keep_aspect: Option<bool>,
    pub symbol_offset: Option<Vec<f64>>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub silent: Option<bool>,
    pub data_group_id: Option<String>,
}

impl Default for RadarSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            radar_index: None,
            item_style: None,
            line_style: None,
            area_style: None,
            label: None,
            symbol: Some(SymbolType::Circle),
            symbol_size: Some(LenientNumber::Number(4.0)),
            symbol_rotate: None,
            symbol_keep_aspect: None,
            symbol_offset: None,
            z: None,
            zlevel: None,
            encode: None,
            mark_point: None,
            mark_line: None,
            mark_area: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            silent: None,
            data_group_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarDataOption {
    pub value: Vec<f64>,
    pub name: Option<String>,
}

// ============================================================
// PolarBarSeriesOption - 极坐标柱状图系列
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolarBarSeriesOption {
    pub name: Option<String>,
    #[serde(default)]
    pub data: Vec<DataPoint>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    /// 每个扇区的颜色，按数据索引
    pub color: Option<Vec<ColorOption>>,
    /// 扇区之间的间隔（角度，单位：度）
    pub pad_angle: Option<f64>,
    /// 起始角度（单位：度，0表示12点钟方向）
    pub start_angle: Option<f64>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub silent: Option<bool>,
    pub data_group_id: Option<String>,
}

impl Default for PolarBarSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            item_style: None,
            label: None,
            color: None,
            pad_angle: Some(2.0),
            start_angle: Some(0.0),
            z: None,
            zlevel: None,
            encode: None,
            mark_point: None,
            mark_line: None,
            mark_area: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            silent: None,
            data_group_id: None,
        }
    }
}

impl PolarBarSeriesOption {
    pub fn new(
        name: impl Into<String>,
        data: impl IntoIterator<Item = impl Into<DataPoint>>,
    ) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

// ============================================================
// PolarScatterSeriesOption - 极坐标散点图系列
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolarScatterSeriesOption {
    pub name: Option<String>,
    /// 数据格式：[角度, 半径] 或 [角度, 半径, 大小]
    pub data: Vec<PolarScatterDataPoint>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub symbol: Option<SymbolType>,
    /// 默认符号大小
    pub symbol_size: Option<SingleOrArray<LenientNumber>>,
    pub symbol_rotate: Option<f64>,
    pub symbol_keep_aspect: Option<bool>,
    pub symbol_offset: Option<Vec<f64>>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub silent: Option<bool>,
    pub data_group_id: Option<String>,
    pub large: Option<bool>,
    pub large_threshold: Option<usize>,
}

impl Default for PolarScatterSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            item_style: None,
            label: None,
            symbol: Some(SymbolType::Circle),
            symbol_size: Some(SingleOrArray::Single(LenientNumber::Number(10.0))),
            symbol_rotate: None,
            symbol_keep_aspect: None,
            symbol_offset: None,
            z: None,
            zlevel: None,
            encode: None,
            mark_point: None,
            mark_line: None,
            mark_area: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            silent: None,
            data_group_id: None,
            large: None,
            large_threshold: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarScatterDataPoint {
    /// 角度（单位：度，0表示12点钟方向，顺时针）
    pub angle: f64,
    /// 半径值
    pub radius: f64,
    /// 可选的符号大小（覆盖 series 的 symbol_size）
    pub symbol_size: Option<f64>,
    /// 可选的名称
    pub name: Option<String>,
}

// ============================================================
// BubbleSeriesOption - 气泡图系列
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BubbleSeriesOption {
    pub name: Option<String>,
    /// 数据格式：[x, y, size] 或 [x, y]
    pub data: Vec<BubbleDataPoint>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
    pub grid_index: Option<usize>,
    /// 气泡大小缩放因子
    pub symbol_size_scale: Option<f64>,
    pub item_style: Option<ItemStyleOption>,
    pub label: Option<LabelOption>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub encode: Option<SeriesEncodeOption>,
    pub mark_point: Option<MarkPointOption>,
    pub mark_line: Option<MarkLineOption>,
    pub mark_area: Option<MarkAreaOption>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub silent: Option<bool>,
    pub data_group_id: Option<String>,
}

impl Default for BubbleSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: Vec::new(),
            x_axis_index: None,
            y_axis_index: None,
            grid_index: None,
            symbol_size_scale: Some(1.0),
            item_style: None,
            label: None,
            z: None,
            zlevel: None,
            encode: None,
            mark_point: None,
            mark_line: None,
            mark_area: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            silent: None,
            data_group_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleDataPoint {
    pub x: f64,
    pub y: f64,
    /// 气泡大小（可选，默认使用固定大小）
    pub size: Option<f64>,
    /// 可选的名称
    pub name: Option<String>,
}

// ============================================================
// GaugeSeriesOption - 仪表盘系列
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeSeriesOption {
    pub name: Option<String>,
    /// 当前值
    pub data: Vec<GaugeDataPoint>,
    /// 最小值
    pub min: Option<f64>,
    /// 最大值
    pub max: Option<f64>,
    /// 中心位置，支持 number | string | Array
    pub center: Option<Vec<LenientNumber>>,
    /// 半径，支持 number | string | Array
    pub radius: Option<SingleOrArray<LenientNumber>>,
    /// 起始角度（默认-225度，即7:30方向）
    pub start_angle: Option<f64>,
    /// 结束角度（默认45度，即4:30方向）
    pub end_angle: Option<f64>,
    /// 分割段数
    pub split_number: Option<usize>,
    /// 轴线样式
    pub axis_line: Option<GaugeAxisLineOption>,
    /// 进度条样式
    pub progress: Option<GaugeProgressOption>,
    /// 指针样式
    pub pointer: Option<GaugePointerOption>,
    /// 刻度样式
    pub axis_tick: Option<GaugeAxisTickOption>,
    /// 刻度标签
    pub axis_label: Option<GaugeAxisLabelOption>,
    /// 分隔线
    pub split_line: Option<GaugeSplitLineOption>,
    /// 标题
    pub title: Option<GaugeTitleOption>,
    /// 详情（数值显示）
    pub detail: Option<GaugeDetailOption>,
    /// 渐变色配置
    pub gradient_colors: Option<Vec<GradientColorStopOption>>,
    pub z: Option<f64>,
    pub zlevel: Option<f64>,
    pub tooltip: Option<TooltipOption>,
    pub animation: Option<LenientBool>,
    pub animation_duration: Option<f64>,
    pub animation_delay: Option<f64>,
    pub silent: Option<bool>,
    pub data_group_id: Option<String>,
    pub item_style: Option<ItemStyleOption>,
    pub title_label: Option<String>,
}

impl Default for GaugeSeriesOption {
    fn default() -> Self {
        Self {
            name: None,
            data: vec![GaugeDataPoint {
                value: 0.0,
                name: None,
            }],
            min: Some(0.0),
            max: Some(100.0),
            center: Some(vec![
                LenientNumber::Percent(50.0),
                LenientNumber::Percent(50.0),
            ]),
            radius: Some(SingleOrArray::Single(LenientNumber::Percent(75.0))),
            start_angle: Some(-225.0),
            end_angle: Some(45.0),
            split_number: Some(10),
            axis_line: None,
            progress: None,
            pointer: None,
            axis_tick: None,
            axis_label: None,
            split_line: None,
            title: None,
            detail: None,
            gradient_colors: None,
            z: None,
            zlevel: None,
            tooltip: None,
            animation: None,
            animation_duration: None,
            animation_delay: None,
            silent: None,
            data_group_id: None,
            item_style: None,
            title_label: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeDataPoint {
    pub value: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeAxisLineOption {
    pub show: Option<bool>,
    pub line_style: Option<LineStyleOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GaugeProgressOption {
    pub show: Option<bool>,
    pub width: Option<f64>,
    pub round_cap: Option<bool>,
    pub clip: Option<bool>,
    pub item_style: Option<ItemStyleOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GaugePointerOption {
    pub show: Option<bool>,
    pub length: Option<String>,
    pub width: Option<LenientNumber>,
    pub item_style: Option<ItemStyleOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeAxisTickOption {
    pub show: Option<bool>,
    pub length: Option<f64>,
    pub line_style: Option<LineStyleOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeAxisLabelOption {
    pub show: Option<bool>,
    pub distance: Option<f64>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeSplitLineOption {
    pub show: Option<bool>,
    pub length: Option<f64>,
    pub line_style: Option<LineStyleOption>,
}

/// Gauge title label options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GaugeTitleOption {
    pub show: Option<bool>,
    pub offset_center: Option<Vec<LenientNumber>>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GaugeDetailOption {
    pub show: Option<bool>,
    pub formatter: Option<String>,
    pub offset_center: Option<Vec<LenientNumber>>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradientColorStopOption {
    pub offset: f64,
    pub color: String,
}

// ============================================================
// DataPoint
// ============================================================

/// A resolved data point in one of three forms:
///
/// | Variant | Meaning | JSON repr |
/// |---------|---------|-----------|
/// | `Value(v)` | value only, key from category index | `1.0` |
/// | `Named(n, v)` | named data point | `["label", 1.0]` |
/// | `XY(x, v)` | x-y data point | `[-1.0, 1.0]` |
///
/// Use [`Into<DataPoint>`] conversions to create instances conveniently:
/// ```
/// # use liecharts::option::DataPoint;
/// let a: DataPoint = 42.0.into();           // Value
/// let b: DataPoint = ("Jan", 30.0).into();  // Named
/// let c: DataPoint = (-1.0, 1.0).into();    // XY
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum DataPoint {
    /// Value-only data point; x position is derived from category index.
    Value(f64),
    /// Named data point: `(category_name, value)`.
    Named(String, f64),
    /// X-Y data point: `(x, y)` for value-axis plots.
    XY(f64, f64),
}

// ── helper accessors ──

impl DataPoint {
    /// If this is a `Value`, returns the contained number.
    pub fn as_value(&self) -> Option<f64> {
        match self {
            DataPoint::Value(n) => Some(*n),
            _ => None,
        }
    }

    /// If this is a `Named`, returns the name and value.
    pub fn as_named(&self) -> Option<(&str, f64)> {
        match self {
            DataPoint::Named(n, v) => Some((n.as_str(), *v)),
            _ => None,
        }
    }

    /// If this is an `XY`, returns the x and y.
    pub fn as_xy(&self) -> Option<(f64, f64)> {
        match self {
            DataPoint::XY(x, y) => Some((*x, *y)),
            _ => None,
        }
    }
}

// ── From impls ──

impl From<f64> for DataPoint {
    fn from(v: f64) -> Self {
        DataPoint::Value(v)
    }
}

impl From<(f64, f64)> for DataPoint {
    fn from((x, y): (f64, f64)) -> Self {
        DataPoint::XY(x, y)
    }
}

impl From<(&str, f64)> for DataPoint {
    fn from((name, value): (&str, f64)) -> Self {
        DataPoint::Named(name.to_string(), value)
    }
}

impl From<(String, f64)> for DataPoint {
    fn from((name, value): (String, f64)) -> Self {
        DataPoint::Named(name, value)
    }
}

// ── Serialize ──

impl Serialize for DataPoint {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DataPoint::Value(v) => v.serialize(serializer),
            DataPoint::Named(n, v) => (n.as_str(), v).serialize(serializer),
            DataPoint::XY(x, y) => (x, y).serialize(serializer),
        }
    }
}

// ── Deserialize ──

struct DataPointVisitor;

impl<'de> Visitor<'de> for DataPointVisitor {
    type Value = DataPoint;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a number, null, a [key, value] array, or a {name, value} object")
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<DataPoint, E> {
        Ok(DataPoint::Value(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<DataPoint, E> {
        Ok(DataPoint::Value(v as f64))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<DataPoint, E> {
        Ok(DataPoint::Value(v as f64))
    }

    /// JSON `null` 数据点 → NaN（在 line 图中会断线，符合 ECharts connect_nulls=false 语义）。
    fn visit_unit<E: de::Error>(self) -> Result<DataPoint, E> {
        Ok(DataPoint::Value(f64::NAN))
    }

    fn visit_none<E: de::Error>(self) -> Result<DataPoint, E> {
        Ok(DataPoint::Value(f64::NAN))
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<DataPoint, A::Error> {
        use serde::de::Error;
        // 收集所有元素，支持 1、2、3+ 元素数组
        let mut elems: Vec<serde_json::Value> = Vec::new();
        while let Some(e) = seq.next_element::<serde_json::Value>()? {
            elems.push(e);
        }
        if elems.is_empty() {
            return Err(A::Error::custom("expected at least 1 element in array"));
        }
        // 1 元素数组：直接作为 Value
        if elems.len() == 1 {
            let v = elems.into_iter().next().unwrap();
            if let Some(n) = v.as_f64() {
                return Ok(DataPoint::Value(n));
            }
            if let serde_json::Value::Null = v {
                return Ok(DataPoint::Value(f64::NAN));
            }
            return Err(A::Error::custom(
                "single element array must be a number or null",
            ));
        }
        // 2+ 元素：取前两个进行解析，第三个及之后（如 bubble 的 size）忽略
        let first = elems.swap_remove(0);
        let second = elems.swap_remove(0);

        match first {
            serde_json::Value::String(s) => {
                let value = second
                    .as_f64()
                    .ok_or_else(|| A::Error::custom("second element must be a number"))?;
                Ok(DataPoint::Named(s, value))
            }
            serde_json::Value::Number(n) => {
                let x = n
                    .as_f64()
                    .ok_or_else(|| A::Error::custom("first element must be a valid number"))?;
                let value = second
                    .as_f64()
                    .ok_or_else(|| A::Error::custom("second element must be a number"))?;
                Ok(DataPoint::XY(x, value))
            }
            // null 第一个元素降级为 Named("", value)，不报错。
            serde_json::Value::Null => {
                let value = second
                    .as_f64()
                    .ok_or_else(|| A::Error::custom("second element must be a number"))?;
                Ok(DataPoint::Named(String::new(), value))
            }
            other => Err(A::Error::custom(format!(
                "unexpected array element type: {:?}",
                other
            ))),
        }
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<DataPoint, A::Error> {
        use serde::de::Error;
        let mut name: Option<String> = None;
        // value 用 serde_json::Value 接受任意类型，后续按类型分支解析，
        // 兼容 ECharts 的 {value: 10} / {value: [x, y]} / {value: [x, y, z]} / {value: null} 等形态。
        let mut value: Option<serde_json::Value> = None;
        let mut x: Option<f64> = None;

        while let Some(k) = map.next_key::<String>()? {
            match k.as_str() {
                "name" => name = Some(map.next_value()?),
                "value" => value = Some(map.next_value()?),
                "x" => x = Some(map.next_value()?),
                _ => {
                    let _: serde_json::Value = map.next_value()?;
                }
            }
        }

        // 解析 value：可能是数字、[x, y] 数组、[x, y, z] 数组、null 或缺失。
        // 数组中的第三个及之后元素（如 bubble 的 z）丢弃以保持兼容。
        let (vx_from_value, value_num) = match value {
            Some(serde_json::Value::Number(n)) => {
                let v = n
                    .as_f64()
                    .ok_or_else(|| A::Error::custom("value must be a valid number"))?;
                (None, v)
            }
            Some(serde_json::Value::Array(arr)) => {
                let mut iter = arr.into_iter();
                match (iter.next(), iter.next()) {
                    (Some(f), Some(s)) => {
                        let xv = f.as_f64().ok_or_else(|| {
                            A::Error::custom("array value first element must be a number")
                        })?;
                        let yv = s.as_f64().ok_or_else(|| {
                            A::Error::custom("array value second element must be a number")
                        })?;
                        (Some(xv), yv)
                    }
                    (Some(f), None) => {
                        let v = f.as_f64().ok_or_else(|| {
                            A::Error::custom("array value single element must be a number")
                        })?;
                        (None, v)
                    }
                    _ => (None, f64::NAN),
                }
            }
            // null 或缺失 → NaN（line 图会断线）。
            Some(serde_json::Value::Null) | None => (None, f64::NAN),
            Some(other) => {
                return Err(A::Error::custom(format!(
                    "unsupported value type: {:?}",
                    other
                )));
            }
        };

        // 优先使用从 value 数组提取的 x；其次用显式 x 字段。
        let xv = vx_from_value.or(x);
        match xv {
            Some(xv) => Ok(DataPoint::XY(xv, value_num)),
            None => Ok(DataPoint::Named(name.unwrap_or_default(), value_num)),
        }
    }
}

impl<'de> Deserialize<'de> for DataPoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(DataPointVisitor)
    }
}

/// Data label configuration for series items.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LabelOption {
    pub show: Option<bool>,
    pub position: Option<LabelPosition>,
    pub formatter: Option<String>,
    pub color: Option<ColorOption>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<String>,
    pub font_style: Option<String>,
    pub rotate: Option<f64>,
    pub distance: Option<f64>,
    pub offset: Option<Vec<f64>>,
    pub align: Option<String>,
    pub vertical_align: Option<String>,
    pub line_height: Option<f64>,
    pub background_color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub border_radius: Option<LenientNumber>,
    pub padding: Option<LenientPadding>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub text_border_color: Option<ColorOption>,
    pub text_border_width: Option<f64>,
    pub text_shadow_blur: Option<f64>,
    pub text_shadow_color: Option<ColorOption>,
    pub text_shadow_offset_x: Option<f64>,
    pub text_shadow_offset_y: Option<f64>,
    pub overflow: Option<String>,
    pub ellipsis: Option<String>,
    pub line_over: Option<String>,
    pub bleed_margin: Option<f64>,
    pub rich: Option<serde_json::Value>,
}

/// Item style configuration (fill color, stroke color, opacity, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemStyleOption {
    pub color: Option<ColorOption>,
    pub border_color: Option<ColorOption>,
    pub border_width: Option<LenientNumber>,
    pub border_type: Option<LineType>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
    pub opacity: Option<f64>,
    pub decal: Option<String>,
}

impl Default for ItemStyleOption {
    fn default() -> Self {
        Self {
            color: None,
            border_color: None,
            border_width: None,
            border_type: None,
            shadow_blur: None,
            shadow_color: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            opacity: Some(1.0),
            decal: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaStyleOption {
    pub color: Option<OneOrMany<ColorOption>>,
    pub opacity: Option<f64>,
    pub origin: Option<String>,
    pub shadow_blur: Option<f64>,
    pub shadow_color: Option<ColorOption>,
    pub shadow_offset_x: Option<f64>,
    pub shadow_offset_y: Option<f64>,
}

impl Default for AreaStyleOption {
    fn default() -> Self {
        Self {
            color: None,
            opacity: Some(0.5),
            origin: None,
            shadow_blur: None,
            shadow_color: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
        }
    }
}

/// 文本对齐配置 - 用于 option 层的序列化
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum TextAlignOption {
    #[default]
    Left,
    Center,
    Right,
}

impl From<TextAlignOption> for crate::visual::TextAlign {
    fn from(option: TextAlignOption) -> Self {
        match option {
            TextAlignOption::Left => crate::visual::TextAlign::Left,
            TextAlignOption::Center => crate::visual::TextAlign::Center,
            TextAlignOption::Right => crate::visual::TextAlign::Right,
        }
    }
}

// ============================================================
// Position 枚举 - 支持预设值、像素值、百分比值
// ============================================================

/// 预设位置值
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PositionPreset {
    Auto,
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

/// 位置枚举 - 支持预设值、像素值或百分比值
#[derive(Debug, Clone, PartialEq)]
pub enum PositionOption {
    Preset(PositionPreset),
    Pixel(f64),
    Percent(f64),
}

impl PositionOption {
    pub fn auto() -> Self {
        PositionOption::Preset(PositionPreset::Auto)
    }
    pub fn center() -> Self {
        PositionOption::Preset(PositionPreset::Center)
    }
    pub fn left() -> Self {
        PositionOption::Preset(PositionPreset::Left)
    }
    pub fn right() -> Self {
        PositionOption::Preset(PositionPreset::Right)
    }
    pub fn top() -> Self {
        PositionOption::Preset(PositionPreset::Top)
    }
    pub fn bottom() -> Self {
        PositionOption::Preset(PositionPreset::Bottom)
    }
    pub fn px(value: f64) -> Self {
        PositionOption::Pixel(value)
    }
    pub fn percent(value: f64) -> Self {
        PositionOption::Percent(value)
    }
}

impl Default for PositionOption {
    fn default() -> Self {
        PositionOption::Preset(PositionPreset::Auto)
    }
}

impl Serialize for PositionOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PositionOption::Preset(p) => {
                let s = match p {
                    PositionPreset::Auto => "auto",
                    PositionPreset::Center => "center",
                    PositionPreset::Left => "left",
                    PositionPreset::Right => "right",
                    PositionPreset::Top => "top",
                    PositionPreset::Bottom => "bottom",
                };
                serializer.serialize_str(s)
            }
            PositionOption::Pixel(v) => serializer.serialize_f64(*v),
            PositionOption::Percent(v) => serializer.serialize_str(&format!("{}%", v)),
        }
    }
}

impl<'de> Deserialize<'de> for PositionOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PositionVisitor;

        impl<'de> Visitor<'de> for PositionVisitor {
            type Value = PositionOption;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a position value: preset string, number, or percentage string")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<PositionOption, E> {
                if value.ends_with('%') {
                    let v = value
                        .trim_end_matches('%')
                        .parse::<f64>()
                        .map_err(|_| de::Error::custom(format!("invalid percentage: {}", value)))?;
                    Ok(PositionOption::Percent(v))
                } else if value.ends_with("px") {
                    let v = value.trim_end_matches("px").parse::<f64>().map_err(|_| {
                        de::Error::custom(format!("invalid pixel value: {}", value))
                    })?;
                    Ok(PositionOption::Pixel(v))
                } else {
                    match value {
                        "auto" => Ok(PositionOption::Preset(PositionPreset::Auto)),
                        "center" | "middle" => Ok(PositionOption::Preset(PositionPreset::Center)),
                        "left" => Ok(PositionOption::Preset(PositionPreset::Left)),
                        "right" => Ok(PositionOption::Preset(PositionPreset::Right)),
                        "top" => Ok(PositionOption::Preset(PositionPreset::Top)),
                        "bottom" => Ok(PositionOption::Preset(PositionPreset::Bottom)),
                        _ => {
                            if let Ok(v) = value.parse::<f64>() {
                                Ok(PositionOption::Pixel(v))
                            } else {
                                Err(de::Error::custom(format!("invalid position: {}", value)))
                            }
                        }
                    }
                }
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<PositionOption, E> {
                Ok(PositionOption::Pixel(value))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<PositionOption, E> {
                Ok(PositionOption::Pixel(value as f64))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<PositionOption, E> {
                Ok(PositionOption::Pixel(value as f64))
            }
        }

        deserializer.deserialize_any(PositionVisitor)
    }
}

// ============================================================
// ColorOption - 颜色类型，支持从 "#RRGGBB" / "#RRGGBBAA" 解析
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorOption {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorOption {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(Self::new(r * 17, g * 17, b * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::new(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::with_alpha(r, g, b, a))
            }
            _ => None,
        }
    }

    /// 从 rgb() / rgba() 字符串解析颜色
    pub fn from_rgba(s: &str) -> Option<Self> {
        let s = s.trim();
        if !(s.starts_with("rgb(") || s.starts_with("rgba(")) {
            return None;
        }
        let start = if s.starts_with("rgba(") { 5 } else { 4 };
        let end = s.len() - 1;
        let inner = &s[start..end];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            Some(Self::new(r, g, b))
        } else if parts.len() == 4 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let a = parts[3].parse::<f64>().ok()?;
            let a_u8 = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
            Some(Self::with_alpha(r, g, b, a_u8))
        } else {
            None
        }
    }

    /// 从 CSS 颜色关键字解析（如 `red`、`blue`、`transparent`）。
    /// 输入需为小写。返回 None 表示不是已知关键字。
    pub fn from_css_keyword(keyword: &str) -> Option<Self> {
        // 仅小写的关键字。返回静态 (r, g, b) 元组以避免分配。
        let (r, g, b): (u8, u8, u8) = match keyword {
            "black" => (0, 0, 0),
            "white" => (255, 255, 255),
            "red" => (255, 0, 0),
            "green" => (0, 128, 0),
            "blue" => (0, 0, 255),
            "yellow" => (255, 255, 0),
            "cyan" | "aqua" => (0, 255, 255),
            "magenta" | "fuchsia" => (255, 0, 255),
            "gray" | "grey" => (128, 128, 128),
            "silver" => (192, 192, 192),
            "maroon" => (128, 0, 0),
            "olive" => (128, 128, 0),
            "navy" => (0, 0, 128),
            "teal" => (0, 128, 128),
            "purple" => (128, 0, 128),
            "orange" => (255, 165, 0),
            "pink" => (255, 192, 203),
            "brown" => (165, 42, 42),
            "lime" => (0, 255, 0),
            _ => return None,
        };
        Some(Self::new(r, g, b))
    }

    fn hex_string(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Default for ColorOption {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Serialize for ColorOption {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.hex_string())
    }
}

impl<'de> Deserialize<'de> for ColorOption {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = ColorOption;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str("a hex/rgb/rgba color string, a CSS keyword, or a gradient object")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<ColorOption, E> {
                // 1. 直接 hex / rgb / rgba 解析
                if let Some(c) = ColorOption::from_hex(value) {
                    return Ok(c);
                }
                if let Some(c) = ColorOption::from_rgba(value) {
                    return Ok(c);
                }
                // 2. CSS 关键字
                let lower = value.trim().to_ascii_lowercase();
                match lower.as_str() {
                    "transparent" | "none" => return Ok(ColorOption::with_alpha(0, 0, 0, 0)),
                    "inherit" | "initial" | "unset" | "revert" | "auto" => {
                        // 这些值在当前渲染器中没有具体语义，降级为黑色 sentinel
                        return Ok(ColorOption::new(0, 0, 0));
                    }
                    _ => {}
                }
                if let Some(c) = ColorOption::from_css_keyword(&lower) {
                    return Ok(c);
                }
                Err(de::Error::custom(format!("invalid color: {}", value)))
            }

            fn visit_string<E: de::Error>(self, value: String) -> Result<ColorOption, E> {
                self.visit_str(&value)
            }

            /// 处理渐变对象：{"type":"linear","colorStops":[{"offset":0,"color":"#xxx"},...]}
            /// 取首个 colorStop 的 color 作为降级色（暂不支持真实渐变渲染）
            fn visit_map<A>(self, mut map: A) -> Result<ColorOption, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                use serde_json::Value;

                let mut color_stops: Option<Value> = None;
                let mut first_color: Option<Value> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "colorStops" | "color_stops" => {
                            color_stops = Some(map.next_value::<Value>()?);
                        }
                        "color" => {
                            first_color = Some(map.next_value::<Value>()?);
                        }
                        _ => {
                            let _: Value = map.next_value()?;
                        }
                    }
                }

                // 优先从 colorStops 取首色
                if let Some(Value::Array(stops)) = color_stops {
                    for stop in &stops {
                        if let Some(c) = stop.get("color").or_else(|| stop.get("Color")) {
                            if let Some(s) = c.as_str() {
                                if let Some(parsed) =
                                    ColorOption::from_hex(s).or_else(|| ColorOption::from_rgba(s))
                                {
                                    return Ok(parsed);
                                }
                            } else if let Value::Object(_) = c {
                                // 嵌套渐变 colorStop，递归降级
                                let nested = serde_json::from_value::<ColorOption>(c.clone()).ok();
                                if let Some(parsed) = nested {
                                    return Ok(parsed);
                                }
                            }
                        }
                    }
                }
                // 没有 colorStops，但对象本身有 color 字段
                if let Some(Value::String(s)) = first_color
                    && let Some(parsed) =
                        ColorOption::from_hex(&s).or_else(|| ColorOption::from_rgba(&s))
                {
                    return Ok(parsed);
                }
                // 渐变对象但解析失败：降级为黑色 sentinel，避免报错
                Ok(ColorOption::new(0, 0, 0))
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}

// ============================================================
// NameLocation 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum NameLocation {
    Start,
    Middle,
    Center,
    #[default]
    End,
}

// ============================================================
// Orient 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum Orient {
    #[default]
    Horizontal,
    Vertical,
}

// ============================================================
// LineType 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum LineType {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

// ============================================================
// FontWeight 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FontWeightNamed {
    Normal,
    Bold,
    Bolder,
    Lighter,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FontWeight {
    Named(FontWeightNamed),
    Numeric(u16),
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Named(FontWeightNamed::Normal)
    }
}

// ============================================================
// SymbolType 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum SymbolType {
    #[default]
    Circle,
    EmptyCircle,
    Rect,
    RoundRect,
    Triangle,
    Diamond,
    Pin,
    Arrow,
    None,
}

// ============================================================
// LabelPosition 枚举
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum LabelPosition {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
    Inside,
    Outside,
    Center,
    Start,
    Middle,
    End,
}

// ============================================================
// AxisType 枚举
// ============================================================

/// Axis type: category, value, or time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum AxisType {
    #[default]
    Category,
    Value,
    Time,
    Log,
}

// ============================================================
// AxisPosition 枚举 - 坐标轴位置
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum AxisPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

// ============================================================
// FontStyle 枚举 - 字体风格
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

// ============================================================
// LabelAlign 枚举 - 标签水平对齐
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum LabelAlign {
    Left,
    #[default]
    Center,
    Right,
}

// ============================================================
// LabelVerticalAlign 枚举 - 标签垂直对齐
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum LabelVerticalAlign {
    Top,
    #[default]
    Middle,
    Bottom,
}
