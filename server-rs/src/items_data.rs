//! 战斗体系数据层（包 A）：已按游戏元素拆分为 weapons / gear / treasures / items 四模块。
//! 本文件保留为 re-export 空壳，保证全项目 `crate::items_data::xxx` 旧引用不破坏。

pub use crate::gear::*;
pub use crate::items::*;
pub use crate::treasures::*;
pub use crate::weapons::*;
