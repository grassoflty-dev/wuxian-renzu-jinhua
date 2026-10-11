//! 无限曙光 · 蓝山保卫战 副本（黄金模板骨架 → 精致副本润色）。
//! 主线：孤山要塞被围 → 世界展示调查（3 处奇观 + 2 位居民）→ BOSS 战前铺垫 → 攻城巨魔督军 → 结算 / 死亡。
//! 保留原 start_boss / lanshan_figths / 结算卡 ls_card / 死亡卡 ls_death / BOSS 遭遇 ls_01 与回合 ls_round 结构与 id 前缀不变；
//! 仅新增开场、奇观调查点、NPC、BOSS 铺垫、开放结局等场景，并把原 ls_00 入口改路由进扩充链。
use crate::defs::*;
use crate::state::GameState;

static NO_EFF: [Eff; 0] = [];
static NO_CH: [ChoiceDef; 0] = [];

fn rage_none(_st: &mut GameState, _log: &mut Vec<String>) {}

// ===== 选择驱动 BOSS（原逻辑，保留不动）=====
fn start_boss(st: &mut GameState) -> String {
    if st.fight.is_none() {
        if let Some(cfg) = crate::scenes::fight_cfg("ls_boss") {
            st.fight = Some(crate::power::scaled_fight("ls_boss", cfg, st, vec![cfg.intro.to_string()]));
        }
    }
    "ls_round".to_string()
}
fn boss_act(st: &mut GameState, dmg: i32, guard: bool) -> String {
    if dmg > 0 { if let Some(f) = st.fight.as_mut() { f.hp = (f.hp - dmg).max(0); } }
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) { return boss_win(st); }
    let raw = if st.fight.as_ref().map(|f| f.raged).unwrap_or(false) { 22 } else { 16 };
    if !guard { st.hp = (st.hp - raw).max(0); }
    if st.hp <= 0 { return "ls_death".to_string(); }
    "ls_round".to_string()
}
fn boss_win(st: &mut GameState) -> String {
    st.points += 500; st.set_flag("ls_boss_down"); st.sp_grade = Some('D');
    crate::world::add_item(st, "ls_reward");
    "ls_card".to_string()
}

pub static LANSHAN_SCENES: &[SceneDef] = &[
    /* ================= 入口（原场景 ls_00，仅改路由进扩充链） ================= */
    SceneDef {
        id: "ls_00", bg: Some("lanshan_bg.png"), loc: Some("蓝山要塞 · 南门"), mood: "calm",
        speaker: None, voice: Some("vo_lanshan_2"),
        text: TextSpec::Static(&[
            "你踏入了「无限曙光 · 蓝山保卫战」。",
            "孤山在晨曦里黑得像一块铁，要塞的旗帜被风扯得哗哗作响。山脚下，黑压压的攻城营盘一眼望不到头，篝火在雾里烧成一片红。",
            "「一城，一山，」老卒在你身后瓮声说，「这一仗要是输，城里的人就再没见过山了。」",
        ]),
        choices: &[
            ChoiceDef { label: "随老卒登上要塞城墙", sub: "San(+5) · 守山之人", cond: None, effects: &[Eff::San(5)], route: Route::To("ls_open_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕一 · 开场扩充（钩子 + 氛围） ================= */
    SceneDef {
        id: "ls_open_1", bg: Some("lanshan_bg.png"), loc: Some("南城墙 · 直面攻城营盘"), mood: "awe",
        speaker: Some("哨兵（嘶哑）"), voice: None,
        text: TextSpec::Static(&[
            "你顺城墙垛口望下去。营地里的攻城槌、冲车、梯子密得像收割前的稻田，巨怪的号角一声接一声，把清晨的空气震得发颤。",
            "「他们三天前就到了，就一直这么围，不攻，也不退。」哨兵抹了把脸，「他们是在等我们……害怕到先开口投降。」",
            "城垛上的守卒一个赛一个平静。他们太明白了：这仗的胜负，从来不在兵力，而在『这座山还想不想让人活下去』。",
        ]),
        choices: &[
            ChoiceDef { label: "接过一袋城砖压上垛口", sub: "+10 点 · 一同守卫", cond: None, effects: &[Eff::Points(10)], route: Route::To("ls_open_2") },
            ChoiceDef { label: "先静立听攻城号角辨意图", sub: "San(-5) · 感受被围", cond: None, effects: &[Eff::San(-5)], route: Route::To("ls_open_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_open_2", bg: Some("lanshan_bg.png"), loc: Some("城内 · 被山抱住的街巷"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你绕进城墙下的街巷。巷子里没人慌张，妇人把仅剩的一缸水舀给了守卒，孩童举着木刀学大人比划，老人坐在门槛上，把一篮栗子一粒粒数给即将上墙的年轻人。",
            "「山在，城就在。」一个老太婆把热栗子塞进你手里，「娃儿，别怕。山认得咱，舍不得塌。」",
            "你抬头——蓝山的主峰就静静地立在城后方，像一只巨大的、沉默的手掌，替整座城挡住了一切来路。",
        ]),
        choices: &[
            ChoiceDef { label: "顺石板路走向城中广场", sub: "进入世界展示区", cond: None, effects: &NO_EFF, route: Route::To("ls_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕二 · 世界展示（奇观调查点 + 居民对话） ================= */
    SceneDef {
        id: "ls_hub", bg: Some("lanshan_bg.png"), loc: Some("城中广场 · 誓师与炊烟"), mood: "calm",
        speaker: Some("老卒"), voice: None,
        text: TextSpec::Static(&[
            "城中广场中央立着一尊凿进山体半身的石像——『守山者』，一手拄城，一手抬着山，在晨曦里像活着一样倔。石匠们正在它脚下，把每一块能用的条石都码成垛口。",
            "广场四周，几处石凿的纪念台在供着先人的名字；一条登山的石阶从广场直通向山腰的烽火台。",
            "有人递来一碗热汤，有人往你怀里塞干粮——这座被围住的城，竟一丝溃散的气象也没有。",
        ]),
        choices: &[
            ChoiceDef { label: "登上石阶 · 山腰烽火台", sub: "奇观① 望山之顶", cond: None, effects: &NO_EFF, route: Route::To("ls_pt_fire") },
            ChoiceDef { label: "供读先人 · 纪念台的永夜", sub: "奇观② 名字之墙", cond: None, effects: &NO_EFF, route: Route::To("ls_pt_mem") },
            ChoiceDef { label: "查看石匠 · 凿不断的山脊", sub: "奇观③ 石匠营", cond: None, effects: &NO_EFF, route: Route::To("ls_pt_stone") },
            ChoiceDef { label: "与老卒攀谈", sub: "与首位居民对话", cond: None, effects: &NO_EFF, route: Route::To("ls_n_veteran") },
            ChoiceDef { label: "走向西门 · 瓮城入口", sub: "BOSS 战前 · 慎入", cond: None, effects: &NO_EFF, route: Route::To("ls_pre_1") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_pt_fire", bg: Some("lanshan_bg.png"), loc: Some("山腰烽火台 · 蓝山之主峰"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你沿石阶爬上山腰烽火台。整座被围的城在你脚下摊开，而烽火台里还堆着一夜未熄的柴，暗红的余烬把石壁烘得温热。",
            "从这里看出去，城外是燃烧的营盘与黑压压的巨怪，城内却是此起彼伏的炊烟，一列列守卒在墙头轮换，像山的脉搏一下一下跳动。",
            "「山不塌，城就散不了。」你终于理解这句话的重量——这座山，是这座城里每个人心里那根不肯倒的骨头。",
        ]),
        choices: &[
            ChoiceDef { label: "在这座山上立誓 · 至少看至天明", sub: "+15 点 · 望山之盟", cond: None,
                effects: &[Eff::SetFlag("ls_saw_mountain"), Eff::Points(15)], route: Route::To("ls_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_pt_mem", bg: Some("lanshan_bg.png"), loc: Some("纪念台 · 名字之墙"), mood: "revelation",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "纪念台的石墙上，一行行名字被凿进山体，新名字叠在旧名字下，从墙面一路凿到墙脚。没有墓，没有碑文，只有名字——和名字旁简短的几笔：守者、妇、孩童。",
            "你伸手去触摸最旧的一层，指尖正落在一条被风雨磨钝的刻痕旁，却仍能辨出那行字：",
            "「山记住的，不算失落。山忘了的——」下面没有写完。铁凿的主人大概知道，这面墙会替他写完。",
        ]),
        choices: &[
            ChoiceDef { label: "把自己的名字也凿上墙脚", sub: "San(-5) · +20 点 · 与山同在", cond: None,
                effects: &[Eff::SetFlag("ls_read_names"), Eff::San(-5), Eff::Points(20)], route: Route::To("ls_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_pt_stone", bg: Some("lanshan_bg.png"), loc: Some("石匠营 · 凿不断的山脊"), mood: "awe",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "石匠营里，满地的条石还带着新凿的齿痕。老石匠蹲在他刚凿好的那块胸石前，用指腹反复抚过每一道纹路，像在安抚一匹受惊的马。",
            "「攻城之前先凿山的，我只服这一手——」他抬头看你，眼里有火，「他们想把山凿成梯子攻上来。我就把山凿成盾，让他们爬一辈子也爬不到顶。」",
            "你注意到，他掌心的老茧比山石还硬，而那把凿子，已经豁了三个口。",
        ]),
        choices: &[
            ChoiceDef { label: "帮他把最后一块棱石凿圆", sub: "+15 点 · 一同凿盾", cond: None,
                effects: &[Eff::SetFlag("ls_stone_cast"), Eff::Points(15)], route: Route::To("ls_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_n_veteran", bg: Some("lanshan_bg.png"), loc: Some("城中广场 · 与老卒对谈"), mood: "tension",
        speaker: Some("老卒"), voice: None,
        text: TextSpec::Static(&[
            "老卒在守山者石像下席地而坐，面前摆着半壶茶。他见你坐下，先给你斟满一杯，才开口：",
            "「守城啊，最怕的不是强敌，是城里先泄了气。」他呷一口，「我守了一辈子，就悟出一件事——山不是我们的盾，我们是山的根。根在，它就不倒。」",
            "「你要上瓮城，会撞见那个督军。别的攻城头目只想要城，他不一样——」老卒顿了顿，「他……想证明哪怕最硬的山，也终归要输给『要攻打』三个字。别跟他争这座山该不该由人守。」",
        ]),
        choices: &[
            ChoiceDef { label: "问他为何这座山值得守", sub: "San(+5) · 守山的理由", cond: None,
                effects: &[Eff::San(5), Eff::Points(10)], route: Route::To("ls_n_veteran2") },
            ChoiceDef { label: "为他满上那杯茶", sub: "+10 点 · 敬守者", cond: None,
                effects: &[Eff::Points(10)], route: Route::To("ls_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_n_veteran2", bg: Some("lanshan_bg.png"), loc: Some("守者 · 攻城之前的一句话"), mood: "choice",
        speaker: Some("老卒"), voice: None,
        text: TextSpec::Static(&[
            "「值得守？」老卒望着石像笑了，眼角的纹路挤在一起，「你往城底下看——那些小儿拿着木刀，老人把栗子分给上墙的人。这就是一座城。」",
            "「山是城里千千万万个人的命叠出来的。你今天护住它一寸，明儿它就能替你挡回十个想踏过去的人。」",
            "他正色看你：「上瓮城之前记住——攻城巨魔督军最恨别人说『山是不能被征服的』。你若真信这句，就别说出口，让他用刀来证实。」",
        ]),
        choices: &[
            ChoiceDef { label: "记住这句忠告与那个名号", sub: "+15 点 · BOSS 铺垫", cond: None,
                effects: &[Eff::SetFlag("ls_warn_tujun"), Eff::Points(15)], route: Route::To("ls_hub") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 幕三 · BOSS 战前铺垫 ================= */
    SceneDef {
        id: "ls_pre_1", bg: Some("lanshan_bg.png"), loc: Some("西门瓮城 · 至死方休的关"), mood: "fear",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "西门瓮城外，攻城槌的声音已经响了一夜。守城门吏死死抵着门，一条腿陷进泥里，脸上却挂着笑——那是一种『门后有人』才笑得出的笑。",
            "门缝里透进来一只攻城巨怪的脚，正一点点往缝里挤。门吏转身朝你咧嘴：",
            "「娃儿，进城去吧。山还在这儿——」他话没说完，门被撞开一道缝，「我就守到门开那一刻。」",
        ]),
        choices: &[
            ChoiceDef { label: "抵住这道门，与他一同撑到黎明", sub: "San(+5) · · 门最终被撞开", cond: None,
                effects: &[Eff::San(5)], route: Route::To("ls_pre_2") },
            ChoiceDef { label: "快步退进瓮城内壁", sub: "被逼至瓮城死角", cond: None, effects: &NO_EFF, route: Route::To("ls_pre_2") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_pre_2", bg: Some("lanshan_bg.png"), loc: Some("瓮城 · 攻城巨魔督军"), mood: "danger",
        speaker: Some("攻城巨魔督军"), voice: None,
        text: TextSpec::Static(&[
            "瓮城中央，攻城巨魔督军扛着被血磨亮的战锤，一步步走进来，脚步把石板震得簌簌掉灰。他停在城自身尽头，单手撑墙，俯视你。",
            "「听说——」他瓮声开口，「你们把这座山，说得跟命一样重。」他回头看了看那座蓝山，「我就是来把这话，一项一项砸烂的。」",
            "「你要护？我陪你护到天黑。你要让？我放你一命，从此你心里再没有山。」他没有拔锤子，可那双盯着山的眼睛，却比锤子更沉。",
        ]),
        choices: &[
            ChoiceDef { label: "【迎战 · 攻城巨魔督军】", sub: "进入原 BOSS 遭遇 ls_01", cond: None, effects: &NO_EFF, route: Route::To("ls_01") },
            ChoiceDef { label: "登烽火台最后望一眼这座山", sub: "结局 · 看景", cond: None, effects: &NO_EFF, route: Route::To("ls_end_view") },
            ChoiceDef { label: "带走一块守山石当纪念", sub: "结局 · 带纪念", cond: None, effects: &NO_EFF, route: Route::To("ls_end_souv") },
            ChoiceDef { label: "留下替这座城再守一夜", sub: "结局 · 停留", cond: None, effects: &NO_EFF, route: Route::To("ls_end_stay") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },

    /* ================= 原场景 ls_01 / ls_round / ls_card / ls_death（保留不动） ================= */
    SceneDef {
        id: "ls_01", bg: Some("lanshan_bg.png"), loc: Some("瓮城 · 决战处"), mood: "danger",
        speaker: Some("攻城巨魔督军"), voice: None,
        text: TextSpec::Static(&["攻城巨魔督军 一步跨来，战锤在地上拖出火星。一个城市，一座山，一场输不起的仗——他偏要在这山前，分出个高下。"]),
        choices: &[
            ChoiceDef { label: "迎战", sub: "", cond: None, effects: &NO_EFF, route: Route::Dyn(start_boss) },
            ChoiceDef { label: "先观察", sub: "", cond: None, effects: &[Eff::Points(5)], route: Route::To("ls_01") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_round", bg: Some("lanshan_bg.png"), loc: Some("决战处"), mood: "danger",
        speaker: None, voice: None,
        text: TextSpec::Dyn(|st| format!("攻城巨魔督军 剩余 {} 血，你 HP {}", st.fight.as_ref().map(|f| f.hp).unwrap_or(0), st.hp)),
        choices: &[
            ChoiceDef { label: "重击", sub: "高伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 30, false)) },
            ChoiceDef { label: "防御", sub: "免伤", cond: None, effects: &NO_EFF, route: Route::Dyn(|st| boss_act(st, 0, true)) },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_card", bg: None, loc: None, mood: "calm", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: None,
            card: |st| crate::state::Card {
                title: "结 算".into(), good: true,
                body_html: format!("<p>你守住了这座山，也守住了有一整个城市的后方。</p><table class='statTable'><tr><td>存活点数</td><td>{}</td></tr></table>", st.points),
                buttons: vec![("回 到 主 神 空 间 ▶".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },
    SceneDef {
        id: "ls_death", bg: None, loc: None, mood: "danger", speaker: None, voice: None,
        text: TextSpec::Static(&[]), choices: &NO_CH, fight_id: None, video: None, cine_label: None,
        overlay: Some(OverlayDef {
            voice: None, death: Some(("无限曙光 · 蓝山保卫战 · 殒命", "殒命于无限曙光 · 蓝山保卫战")),
            card: |_st| crate::state::Card {
                title: "死 亡".into(), good: false,
                body_html: r#"<p>你死在了这里。</p><p style='color:#ff8a8a'>【死亡档案】</p>"#.to_string(),
                buttons: vec![("回 主 神 空 间 · 复 活".into(), "__enter_nexus__".into())],
                voice: None,
            },
        }),
    },

    /* ================= 开放结局三分支（看景 / 带纪念 / 停留，route 到原结算卡 ls_card） ================= */
    SceneDef {
        id: "ls_end_view", bg: Some("lanshan_bg.png"), loc: Some("烽火台 · 结局 · 望山到天明"), mood: "calm",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你没有和督军在瓮城分生死，只是退回山腰烽火台，把这座被围的城与那座不肯倒的山，一起看到天明。",
            "号角声里，城中炊烟升起，守卒在墙头轮换，像一日复一日的活着本身，就是最小的胜利。",
            "山在，城在，人还在。你没有带兵刃入城，却把这『山离心脏最近的距离』，完完整整看了进去。（结局 · 看景）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 看见山在", cond: None,
                effects: &[Eff::SetFlag("ls_end_view"), Eff::PointsIfFlag("ls_saw_mountain", 30)], route: Route::To("ls_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_end_souv", bg: Some("lanshan_bg.png"), loc: Some("石匠营 · 结局 · 带走守山石"), mood: "choice",
        speaker: Some("旁白"), voice: None,
        text: TextSpec::Static(&[
            "你在石匠营里，拣起一块被他凿圆了棱角的守山石，握进掌心。石面还带着凿子的齿痕与手掌的余温。",
            "「带上它。」老石匠没抬头，「往后不论走多远，只要它还挺在腰里，就当这座山还替你把后路撑着。」",
            "你快步离开瓮城，石头的重量沉在怀里，像把一整座不肯输的山，也一并揣进了行囊。（结局 · 带走纪念）",
        ]),
        choices: &[
            ChoiceDef { label: "（回到主神空间 · 结算）", sub: "+40 点 · 带走『守山石』", cond: None,
                effects: &[Eff::SetFlag("ls_end_souv"), Eff::Points(40), Eff::AddItem("ls_souvenir_stone")], route: Route::To("ls_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
    SceneDef {
        id: "ls_end_stay", bg: Some("lanshan_bg.png"), loc: Some("瓮城 · 结局 · 留下守一夜"), mood: "fear",
        speaker: Some("攻城巨魔督军"), voice: None,
        text: TextSpec::Static(&[
            "你在督军面前放下兵器也没有退，而是退回瓮城城垛旁，拿起一支守军的矛，加入了对峙的行列。他站在原地，盯着你看了很久。",
            "「……很多人都说山值得守，可你是头一个真愿意留下来守的。」他收回战锤，「今夜，我不砸你的门。」",
            "他转身走进夜色。你握着那支滚烫的矛，陪这座城守完了这一夜——山在，你也在。（结局 · 停留）",
        ]),
        choices: &[
            ChoiceDef { label: "（天明后回到主神空间 · 结算）", sub: "+40 点 · 守夜者", cond: None,
                effects: &[Eff::SetFlag("ls_end_stay"), Eff::Points(40), Eff::PointsIfFlag("ls_stone_cast", 30)], route: Route::To("ls_card") },
        ],
        fight_id: None, video: None, cine_label: None, overlay: None,
    },
];

pub fn lanshan_figths() -> &'static [(&'static str, FightCfg)] {
    &[
        ("ls_boss", FightCfg {
            name: "攻城巨魔督军", hp: 260, dmg: (20, 32), reward: 500, reward_why: "击败 BOSS", intro: "BOSS 出现！",
            rage_at: Some(60), rage_text: "狂暴了！", on_rage: rage_none,
            finisher_if: |_st, _ehp| false, finisher_name: |_st| String::new(), finisher_desc: |_st| String::new(),
            win: |_st| "ls_card".to_string(), death: "ls_death",
        }),
    ]
}