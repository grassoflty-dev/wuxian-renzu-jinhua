//! 任务系统数据层（任务列表需求 §六.40/41）。
//! 任务 = 主线 + 支线；完成条件用 flag 判断（done_when）。
//! 已铺装全部世界的主线任务（完成判定用 worlds::clear_flag 对应的真实通关旗标）。

use serde::Serialize;

use crate::state::GameState;

/// 任务类别
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskKind {
    Main,
    Side,
}

impl TaskKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskKind::Main => "main",
            TaskKind::Side => "side",
        }
    }
}

/// 任务定义（静态表 TASKS 条目）
pub struct TaskDef {
    pub id: &'static str,
    pub world_id: &'static str,
    pub kind: TaskKind,
    pub title: &'static str,
    /// 完成判定：flag 达成即完成
    pub done_when: fn(&GameState) -> bool,
}

/// 任务视图（下发前端）
#[derive(Serialize, Clone)]
pub struct TaskView {
    pub id: &'static str,
    pub kind: &'static str,
    pub title: &'static str,
    pub done: bool,
}

pub static TASKS: &[TaskDef] = &[
    TaskDef {
        id: "bh_main",
        world_id: crate::worlds::WORLD_BIOHAZARD,
        kind: TaskKind::Main,
        title: "主线 · 进入蜂巢，关闭红后，活着回到地面",
        done_when: |st| st.flag("bh_cleared"),
    },
    TaskDef {
        id: "bh_A",
        world_id: crate::worlds::WORLD_BIOHAZARD,
        kind: TaskKind::Side,
        title: "支线A · 与蕾恩建立信任",
        done_when: |st| st.flag("A"),
    },
    TaskDef {
        id: "bh_B1",
        world_id: crate::worlds::WORLD_BIOHAZARD,
        kind: TaskKind::Side,
        title: "支线B1 · 洞察丧尸行动规律",
        done_when: |st| st.flag("B1"),
    },
    TaskDef {
        id: "bh_B2",
        world_id: crate::worlds::WORLD_BIOHAZARD,
        kind: TaskKind::Side,
        title: "支线B2 · 参透激光通道机关",
        done_when: |st| st.flag("B2"),
    },
    TaskDef {
        id: "bh_C",
        world_id: crate::worlds::WORLD_BIOHAZARD,
        kind: TaskKind::Side,
        title: "支线C · 为蕾恩注射肾上腺素",
        done_when: |st| st.flag("C"),
    },
    TaskDef {
        id: "bh_decon",
        world_id: crate::worlds::WORLD_BIOHAZARD,
        kind: TaskKind::Side,
        title: "隐藏 · 窥破消毒事故真相",
        done_when: |st| st.flag("decon_truth"),
    },
    TaskDef {
        id: "bh_cooling",
        world_id: crate::worlds::WORLD_BIOHAZARD,
        kind: TaskKind::Side,
        title: "隐藏 · 修复冷却回路并读取散热情报",
        done_when: |st| st.flag("server_cooling"),
    },
    TaskDef {
        id: "bh_nav",
        world_id: crate::worlds::WORLD_BIOHAZARD,
        kind: TaskKind::Side,
        title: "隐藏 · 参透蜂巢垂直捷径",
        done_when: |st| st.flag("nav_manual_cross"),
    },
    TaskDef {
        id: "xinghe_main",
        world_id: "xinghe",
        kind: TaskKind::Main,
        title: "主线 · 深入脑虫巢穴，歼灭脑虫",
        done_when: |st| st.flag("xh_brain_down"),
    },
    TaskDef {
        id: "tianting_main",
        world_id: "tianting",
        kind: TaskKind::Main,
        title: "主线 · 完成封神试炼",
        done_when: |st| st.flag("tt_cleared"),
    },
    TaskDef {
        id: "yinse_dadi_main",
        world_id: "yinse_dadi",
        kind: TaskKind::Main,
        title: "主线 · 探明机界遗迹真相",
        done_when: |st| st.flag("ys_cleared"),
    },
    TaskDef {
        id: "moruiya_main",
        world_id: "moruiya",
        kind: TaskKind::Main,
        title: "主线 · 穿越摩瑞亚矿坑的黑暗",
        done_when: |st| st.flag("mo_cleared"),
    },
    TaskDef {
        id: "poxiao_main",
        world_id: "poxiao",
        kind: TaskKind::Main,
        title: "主线 · 突破破晓封锁区",
        done_when: |st| st.flag("px_cleared"),
    },
    TaskDef {
        id: "zhuyuan_main",
        world_id: "zhuyuan",
        kind: TaskKind::Main,
        title: "主线 · 通关「咒怨」",
        done_when: |st| st.flag("zhuyuan_cleared"),
    },
    TaskDef {
        id: "moshi_shoucheng_main",
        world_id: "moshi_shoucheng",
        kind: TaskKind::Main,
        title: "主线 · 通关「末世死城」",
        done_when: |st| st.flag("moshi_shoucheng_cleared"),
    },
    TaskDef {
        id: "yiying_main",
        world_id: "yiying",
        kind: TaskKind::Main,
        title: "主线 · 通关「异形4·奥瑞迦号」",
        done_when: |st| st.flag("yiying_cleared"),
    },
    TaskDef {
        id: "tianshe_main",
        world_id: "tianshe",
        kind: TaskKind::Main,
        title: "主线 · 通关「天蛇族实验室」",
        done_when: |st| st.flag("dungeon_cleared"),
    },
    TaskDef {
        id: "jiguancheng_main",
        world_id: "jiguancheng",
        kind: TaskKind::Main,
        title: "主线 · 通关「机关城」",
        done_when: |st| st.flag("jiguancheng_cleared"),
    },
    TaskDef {
        id: "cangjingge_main",
        world_id: "cangjingge",
        kind: TaskKind::Main,
        title: "主线 · 通关「藏经阁」",
        done_when: |st| st.flag("cangjingge_cleared"),
    },
    TaskDef {
        id: "jianzhong_main",
        world_id: "jianzhong",
        kind: TaskKind::Main,
        title: "主线 · 通关「剑冢」",
        done_when: |st| st.flag("jianzhong_cleared"),
    },
    TaskDef {
        id: "tongqu_main",
        world_id: "tongqu",
        kind: TaskKind::Main,
        title: "主线 · 通关「通衢古镇」",
        done_when: |st| st.flag("tongqu_cleared"),
    },
    TaskDef {
        id: "juluoji_main",
        world_id: "juluoji",
        kind: TaskKind::Main,
        title: "主线 · 通关「侏罗纪公园」",
        done_when: |st| st.flag("juluoji_cleared"),
    },
    TaskDef {
        id: "sishen_main",
        world_id: "sishen",
        kind: TaskKind::Main,
        title: "主线 · 通关「死神来了」",
        done_when: |st| st.flag("sishen_cleared"),
    },
    TaskDef {
        id: "mumiyi_main",
        world_id: "mumiyi",
        kind: TaskKind::Main,
        title: "主线 · 通关「木乃伊」",
        done_when: |st| st.flag("mumiyi_cleared"),
    },
    TaskDef {
        id: "mojiao_main",
        world_id: "mojiao",
        kind: TaskKind::Main,
        title: "主线 · 通关「魔教总坛」",
        done_when: |st| st.flag("mojiao_cleared"),
    },
    TaskDef {
        id: "wulin_main",
        world_id: "wulin",
        kind: TaskKind::Main,
        title: "主线 · 通关「武林大会」",
        done_when: |st| st.flag("wulin_cleared"),
    },
    TaskDef {
        id: "hezi_main",
        world_id: "hezi",
        kind: TaskKind::Main,
        title: "主线 · 通关「盒壁层」",
        done_when: |st| st.flag("hezi_cleared"),
    },
    TaskDef {
        id: "shaqiu_main",
        world_id: "shaqiu",
        kind: TaskKind::Main,
        title: "主线 · 通关「沙丘魔海」",
        done_when: |st| st.flag("shaqiu_cleared"),
    },
    TaskDef {
        id: "yize_main",
        world_id: "yize",
        kind: TaskKind::Main,
        title: "主线 · 通关「远古遗迹」",
        done_when: |st| st.flag("yize_cleared"),
    },
    TaskDef {
        id: "tiexue_main",
        world_id: "tiexue",
        kind: TaskKind::Main,
        title: "主线 · 通关「铁血金字塔」",
        done_when: |st| st.flag("tiexue_cleared"),
    },
    TaskDef {
        id: "yecheng_main",
        world_id: "yecheng",
        kind: TaskKind::Main,
        title: "主线 · 通关「夜城」",
        done_when: |st| st.flag("yecheng_cleared"),
    },
    TaskDef {
        id: "tiexue2_main",
        world_id: "tiexue2",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("tiexue2_cleared"),
    },
    TaskDef {
        id: "xingjichuanqi_main",
        world_id: "xingjichuanqi",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("xingjichuanqi_cleared"),
    },
    TaskDef {
        id: "xinhuangfang_main",
        world_id: "xinhuangfang",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("xinhuangfang_cleared"),
    },
    TaskDef {
        id: "huanxiongshi_main",
        world_id: "huanxiongshi",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("huanxiongshi_cleared"),
    },
    TaskDef {
        id: "mengguijie_main",
        world_id: "mengguijie",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("mengguijie_cleared"),
    },
    TaskDef {
        id: "siwuzhen_main",
        world_id: "siwuzhen",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("siwuzhen_cleared"),
    },
    TaskDef {
        id: "jingjiling_main",
        world_id: "jingjiling",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("jingjiling_cleared"),
    },
    TaskDef {
        id: "shenmiao_main",
        world_id: "shenmiao",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("shenmiao_cleared"),
    },
    TaskDef {
        id: "shuangbai_main",
        world_id: "shuangbai",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("shuangbai_cleared"),
    },
    TaskDef {
        id: "dashengtang_main",
        world_id: "dashengtang",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("dashengtang_cleared"),
    },
    TaskDef {
        id: "daliexi_main",
        world_id: "daliexi",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("daliexi_cleared"),
    },
    TaskDef {
        id: "poxu_main",
        world_id: "poxu",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("poxu_cleared"),
    },
    TaskDef {
        id: "hangu_main",
        world_id: "hangu",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("hangu_cleared"),
    },
    TaskDef {
        id: "panbu_main",
        world_id: "panbu",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("panbu_cleared"),
    },
    TaskDef {
        id: "diweidu_main",
        world_id: "diweidu",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("diweidu_cleared"),
    },
    TaskDef {
        id: "sanlian_main",
        world_id: "sanlian",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("sanlian_cleared"),
    },
    TaskDef {
        id: "wujin_main",
        world_id: "wujin",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("wujin_cleared"),
    },
    TaskDef {
        id: "yizhong_main",
        world_id: "yizhong",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("yizhong_cleared"),
    },
    TaskDef {
        id: "jishengqianye_main",
        world_id: "jishengqianye",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("jishengqianye_cleared"),
    },
    TaskDef {
        id: "miwu_main",
        world_id: "miwu",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("miwu_cleared"),
    },
    TaskDef {
        id: "xingchen_main",
        world_id: "xingchen",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("xingchen_cleared"),
    },
    TaskDef {
        id: "yinxiang_main",
        world_id: "yinxiang",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("yinxiang_cleared"),
    },
    TaskDef {
        id: "nuoya_main",
        world_id: "nuoya",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("nuoya_cleared"),
    },
    TaskDef {
        id: "lanshan_main",
        world_id: "lanshan",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("lanshan_cleared"),
    },
    TaskDef {
        id: "shourongsuo_main",
        world_id: "shourongsuo",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("shourongsuo_cleared"),
    },
    TaskDef {
        id: "tianwang_main",
        world_id: "tianwang",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("tianwang_cleared"),
    },
    TaskDef {
        id: "xingjijianchuan_main",
        world_id: "xingjijianchuan",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("xingjijianchuan_cleared"),
    },
    TaskDef {
        id: "jialebi_main",
        world_id: "jialebi",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("jialebi_cleared"),
    },
    TaskDef {
        id: "jishujing_main",
        world_id: "jishujing",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("jishujing_cleared"),
    },
    TaskDef {
        id: "baisun_main",
        world_id: "baisun",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("baisun_cleared"),
    },
    TaskDef {
        id: "bihai_main",
        world_id: "bihai",
        kind: TaskKind::Main,
        title: "主线 · 通关并返回主神空间",
        done_when: |st| st.flag("bihai_cleared"),
    },
    TaskDef {
        id: "xingjichuanqi2_main",
        world_id: "xingjichuanqi2",
        kind: TaskKind::Main,
        title: "主线 · 通关「星际传奇·灰雾之心」",
        done_when: |st| st.flag("xingjichuanqi2_cleared"),
    },
    TaskDef {
        id: "shenghua3_main",
        world_id: "shenghua3",
        kind: TaskKind::Main,
        title: "主线 · 通关「浣熊市地下」",
        done_when: |st| st.flag("shenghua3_cleared"),
    },
];

/// 当前世界任务视图（含完成状态）。主神空间等无任务世界返回空列表。
pub fn task_view(st: &GameState) -> Vec<TaskView> {
    TASKS
        .iter()
        .filter(|t| t.world_id == st.world_id.as_str())
        .map(|t| TaskView {
            id: t.id,
            kind: t.kind.as_str(),
            title: t.title,
            done: (t.done_when)(st),
        })
        .collect()
}

/// 当前任务：当前世界第一个未完成的主线任务标题（无则 None）。
pub fn current_task(st: &GameState) -> Option<&'static str> {
    TASKS
        .iter()
        .filter(|t| {
            t.world_id == st.world_id.as_str() && t.kind == TaskKind::Main && !(t.done_when)(st)
        })
        .map(|t| t.title)
        .next()
}
