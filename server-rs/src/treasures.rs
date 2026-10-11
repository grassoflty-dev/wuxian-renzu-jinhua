//! 法宝表（修真 §2.4，装备格 slot 0=本命 1=护身 2=辅助）
//! 由 items_data 拆分而来，仅数据铺装与 treasure_def 查询 helper。

use crate::defs::TreasureDef;

pub const TRESURE_DEFS: &[TreasureDef] = &[
    TreasureDef { id: "cu_bab_benming_fejian", name: "本命飞剑·青锋", desc: "以心血温养的本命飞剑，剑在人在，剑亡身殒。它与你性命相连，也是你唯一不许背弃的「我」。", slot: 0, dmg_reduce: 0, atk_flat: 8, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.1, ignore_death: false, price: 4000, need_grade: Some('C') },
    TreasureDef { id: "cu_bab_hudun_fu", name: "护体符印", desc: "一枚刻进肉里的旧符，危急时自会绽开护主。符纹越磨越淡，像你越来越短的命，也像最后一点不肯认输的执拗。", slot: 1, dmg_reduce: 6, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 2500, need_grade: Some('D') },
    TreasureDef { id: "cu_bab_hunyuan_lu", name: "混元炉", desc: "一炉混元，炼化万物浊气返上清。炉中焰温吞，却足以把游走心头的魔念，熬成一线安定。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 5, qi_max: 0, hp_max: 0, per_turn_qi: 3, dmg_mult: 1.0, ignore_death: false, price: 9000, need_grade: Some('B') },
    TreasureDef { id: "cu_bab_wufen_bazhan", name: "五方幡", desc: "五方旗幡，替一次必死之劫。据说幡起时，能听见阎罗殿前那句——「此人命不该绝」的叹息。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: true, price: 16000, need_grade: Some('A') },
    TreasureDef { id: "cu_bab_qiankun_jie", name: "乾坤袋", desc: "袖里乾坤，可纳山河。它吞得下整座城池，却塞不回那个没能带走的人。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 40, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 45000, need_grade: Some('S') },
    TreasureDef { id: "cu_bab_qiushui_jian", name: "秋水神剑", desc: "剑名秋水，澄澈见底，映得出一个人心里最暗的阴影。它不藏锋，它只照。", slot: 0, dmg_reduce: 0, atk_flat: 12, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.2, ignore_death: false, price: 30000, need_grade: Some('S') },
    // ---- 增量扩充（第 2 批 · 攻击/防御/辅助三型）----
    TreasureDef { id: "tr_zhuxian_calendar", name: "诛仙剑意图", desc: "一方残卷，载尽诛仙剑阵的杀伐之机。展开即见万千剑光，只是执图之人，终要拿自己的心去喂它。", slot: 0, dmg_reduce: 0, atk_flat: 16, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.25, ignore_death: false, price: 32000, need_grade: Some('S') },
    TreasureDef { id: "tr_blood_banner", name: "血煞战旗", desc: "战旗浸透厮杀的血煞，扬起时，千军万马的吼声都往你一人身上压来。旗不倒，命不绝。", slot: 0, dmg_reduce: 0, atk_flat: 10, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.15, ignore_death: false, price: 15000, need_grade: Some('A') },
    TreasureDef { id: "tr_taixu_shield", name: "太虚玄光镜", desc: "镜面悬着太虚，映出的不是你的脸，是下一劫将至的预兆。它以虚挡实，护你一时安稳。", slot: 1, dmg_reduce: 12, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 30, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 20000, need_grade: Some('A') },
    TreasureDef { id: "tr_shenlei_pendant", name: "神雷辟邪佩", desc: "一枚雷纹古佩，邪祟近身时自会噼啪作响，似有雷神在侧替你喝退不干净的东西。", slot: 1, dmg_reduce: 8, atk_flat: 0, dodge: 0.0, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 13000, need_grade: Some('B') },
    TreasureDef { id: "tr_danxin_mirror", name: "锻心明镜", desc: "镜可锻心，照见的每一丝杂念，都会被磨成澄明。你越恐惧，镜面越亮，也就越无处可逃。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 6, qi_max: 60, hp_max: 0, per_turn_qi: 5, dmg_mult: 1.0, ignore_death: false, price: 22000, need_grade: Some('A') },
    TreasureDef { id: "tr_undo_pillowstone", name: "逆转生死盘", desc: "一方能拨回死生的罗盘，转一圈，就把你从鬼门关前拽回来。可有些代价，连它也不肯替你算清。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: true, price: 60000, need_grade: Some('S') },
    // ==================== 增量扩充（第 3 批 · +20 法宝 · 动漫/小说） ====================
    // ---- 本命（slot=0 攻击）----
    TreasureDef { id: "tr_shengbei_shengtian", name: "圣杯·神圣权柄", desc: "杯中盛满「愿」，也盛满「代价」。谁能许愿，谁就得先填进自己最珍贵的那一块命。", slot: 0, dmg_reduce: 0, atk_flat: 18, dodge: 0.0, san_resist: 6, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.3, ignore_death: false, price: 36000, need_grade: Some('S') },
    TreasureDef { id: "tr_mo_jie_jiujie", name: "魔戒·至尊戒", desc: "至尊魔戒，统御诸戒。戴上它的那一刻，你就分不清——是它戴着你，还是你戴着它。", slot: 0, dmg_reduce: 0, atk_flat: 20, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.35, ignore_death: false, price: 40000, need_grade: Some('S') },
    TreasureDef { id: "tr_guaili_yuantu", name: "乖离剑·原质图谱", desc: "图谱记载世界最初的原质，乖离剑便是以它「划开」天地的。看久了，你会忘记自己只是后来者。", slot: 0, dmg_reduce: 0, atk_flat: 16, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.25, ignore_death: false, price: 30000, need_grade: Some('S') },
    TreasureDef { id: "tr_yinyang_jing", name: "阴阳宝镜", desc: "镜分阴阳，可窥两界。它照得见生，也照得见死，照得最清楚的，是你此刻站在这条界线上的模样。", slot: 0, dmg_reduce: 0, atk_flat: 14, dodge: 0.0, san_resist: 4, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.2, ignore_death: false, price: 22000, need_grade: Some('A') },
    TreasureDef { id: "tr_bahuang_longyin", name: "八荒龙印", desc: "八荒龙气凝成的印，戾气与王气同在。执印者威加四海，也终将被这四海，一点点吞没。", slot: 0, dmg_reduce: 0, atk_flat: 12, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.15, ignore_death: false, price: 16000, need_grade: Some('A') },
    TreasureDef { id: "tr_leishen_xianglu", name: "雷神锤·神威", desc: "神威一锤，雷霆万钧。它只认「配得上」它的人，你若心生退意，锤便重得再也举不起来。", slot: 0, dmg_reduce: 0, atk_flat: 15, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 2, dmg_mult: 1.2, ignore_death: false, price: 24000, need_grade: Some('A') },
    // ---- 护身（slot=1 防御）----
    TreasureDef { id: "tr_mo_jing_xianshi", name: "贤者之镜·洞察", desc: "镜名洞察，照见的都是「本相」。它不怜你，只把真相摊在你眼前，让你无处躲闪。", slot: 1, dmg_reduce: 14, atk_flat: 0, dodge: 0.0, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 26000, need_grade: Some('A') },
    TreasureDef { id: "tr_duantou_mojing", name: "魔镜·破碎之握", desc: "镜已破碎，裂痕里伸出谁的手也看不清。它护身，却总想把什么从镜那头，也一并拽过来。", slot: 1, dmg_reduce: 13, atk_flat: 4, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 20, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 21000, need_grade: Some('A') },
    TreasureDef { id: "tr_ahnidun_shield", name: "埃葵斯神盾", desc: "神盾埃葵斯，嵌着美杜莎之颅。它挡下刀剑，却挡不住那个直视它、从此不再移开的目光。", slot: 1, dmg_reduce: 18, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 40, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 32000, need_grade: Some('S') },
    TreasureDef { id: "tr_longnei_xiangu", name: "龙内丹", desc: "一枚温热的龙丹，藏着千年道行。它在你体内慢慢化开，也把龙的孤寂，一并种进你的气海。", slot: 1, dmg_reduce: 10, atk_flat: 6, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 3, dmg_mult: 1.0, ignore_death: false, price: 18000, need_grade: Some('A') },
    TreasureDef { id: "tr_shenlingsan_yu", name: "神陵山玉符", desc: "神陵山传下的玉符，刻着安魂的云纹。贴身戴着，连噩梦都不敢轻易近前。", slot: 1, dmg_reduce: 8, atk_flat: 0, dodge: 0.10, san_resist: 6, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 14000, need_grade: Some('B') },
    TreasureDef { id: "tr_xuanhuang_taibao", name: "玄黄太宝", desc: "天地玄黄，孕此至宝。它浑朴无华，只在生死之际才肯显出一线护你的神异。", slot: 1, dmg_reduce: 15, atk_flat: 0, dodge: 0.0, san_resist: 10, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 28000, need_grade: Some('S') },
    // ---- 辅助（slot=2 回气/生存/SAN）----
    TreasureDef { id: "tr_xianzhe_ziliao", name: "贤者之石·点金", desc: "贤者之石，点石成金。它成金的不是石头，是你那点所剩无几的、想活下来的念。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 8, dmg_mult: 1.0, ignore_death: false, price: 30000, need_grade: Some('S') },
    TreasureDef { id: "tr_sisin_luandao", name: "死神镰刀·摄魂", desc: "镰刀专摄生魂，挥过之处，亡者皆静。它不夺命，它只是替死神，把迷路的魂引回原处。", slot: 2, dmg_reduce: 0, atk_flat: 8, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.1, ignore_death: false, price: 20000, need_grade: Some('A') },
    TreasureDef { id: "tr_longzu_shengyi", name: "龙珠·七龙珠", desc: "集齐七颗，便能召唤神龙，实现一个愿望。可每个轮回的「愿望」，都早在你开口前，被谁先许过了。", slot: 2, dmg_reduce: 0, atk_flat: 10, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 30, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 24000, need_grade: Some('A') },
    TreasureDef { id: "tr_mishen_zhi_tong", name: "三千世界神瞳", desc: "一瞳可窥三千世界，浮光掠影皆入眼。看得越多，越分不清哪个才是你真正该回去的那一个。", slot: 2, dmg_reduce: 0, atk_flat: 6, dodge: 0.08, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 21000, need_grade: Some('A') },
    TreasureDef { id: "tr_tianlu_pa", name: "天书残卷", desc: "天书只余残卷，字迹半漫。可就是这半页，也不知困住了多少窥天机、却回不了头的人。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 12, qi_max: 40, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: false, price: 18000, need_grade: Some('A') },
    TreasureDef { id: "tr_huanjing_luo", name: "幻境罗盘", desc: "罗盘指针不指南北，只指人心最脆的那一处。转一圈，前路是幻，也未必不是生天。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 0, qi_max: 30, hp_max: 0, per_turn_qi: 5, dmg_mult: 1.0, ignore_death: false, price: 16000, need_grade: Some('B') },
    TreasureDef { id: "tr_shengwg_huanghun", name: "圣域星芒", desc: "星芒自圣域坠落，落在掌心像一枚不肯熄灭的星火。它照不亮整片黑夜，却总能护你走出这一段。", slot: 2, dmg_reduce: 3, atk_flat: 6, dodge: 0.0, san_resist: 8, qi_max: 0, hp_max: 0, per_turn_qi: 2, dmg_mult: 1.0, ignore_death: false, price: 17000, need_grade: Some('A') },
    TreasureDef { id: "tr_mengjing_guiji", name: "梦境诡计宝匣", desc: "匣中藏着梦境的诡计，开合之间，能在某次必死之场上偷换因果。只是醒来时，别问梦去了哪里。", slot: 2, dmg_reduce: 0, atk_flat: 0, dodge: 0.0, san_resist: 10, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.0, ignore_death: true, price: 35000, need_grade: Some('S') },
    // ==================== 增量扩充（第 4 批 · 数据子代理追加 1 件） ====================
    TreasureDef { id: "tr_xianfeng_ling", name: "先锋灵幡", desc: "幡上绣着半面冲锋的图腾，扬起时连风声都替你先喊了一嗓子。它不守，它只问你还敢不敢先上。", slot: 0, dmg_reduce: 0, atk_flat: 12, dodge: 0.0, san_resist: 0, qi_max: 0, hp_max: 0, per_turn_qi: 0, dmg_mult: 1.15, ignore_death: false, price: 15000, need_grade: Some('A') },
];

/// 按法宝 id 查表
pub fn treasure_def(id: &str) -> Option<&'static TreasureDef> {
    TRESURE_DEFS.iter().find(|t| t.id == id)
}
