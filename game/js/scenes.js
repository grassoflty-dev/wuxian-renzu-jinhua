/* 无限轮回 · 第一章 生化蜂巢 —— 场景剧本（取材《无限恐怖》第一部「生化危机」） */
"use strict";

/* ================= 结算辅助 ================= */
function buildSettleHtml(st) {
  const sideNames = { A: "支线A · 与蕾恩建立信任", B1: "支线B1 · 洞察丧尸行动规律", B2: "支线B2 · 参透激光通道机关", C: "支线C · 为蕾恩注射肾上腺素" };
  const aliveKeys = Object.keys(S.team).filter(k => !S.deadTeam.includes(k));
  const aliveBonus = aliveKeys.length * 100;
  const sides = Object.keys(sideNames).filter(k => S.flags[k]);
  const sideBonus = sides.length * 200;
  const total = S.points + aliveBonus + sideBonus;
  const rank = total >= 1600 ? "S" : total >= 1300 ? "A" : total >= 1000 ? "B" : total >= 700 ? "C" : "D";
  st.settleTotal = total; st.settleRank = rank;
  return `<h2 class="good">任 务 完 成</h2>
  <p style="text-align:center;color:#9a958a">「生化危机」世界通关 · 轮回结算</p>
  <table class="statTable">
    <tr><td>累计奖励点数（击杀·救援·探索）</td><td>+${S.points}</td></tr>
    <tr><td>存活队友加成 × ${aliveKeys.length} 人</td><td>+${aliveBonus}</td></tr>
    <tr><td>支线剧情达成 × ${sides.length} 条</td><td>+${sideBonus}</td></tr>
    <tr><td>剩余理智</td><td>${Math.max(0, S.san)}</td></tr>
    <tr><td style="color:#ffd76a"><b>综合评价</b></td><td style="color:#ffd76a;font-size:18px"><b>${rank}</b></td></tr>
  </table>
  <p>${sides.length ? "已达成：" + sides.map(k => sideNames[k]).join("；") : "未达成任何隐藏支线。下一世，试着多看、多问、多想。"}</p>`;
}

const SCENES = {

/* ================= 序章 ================= */
s_office: {
  bg: "img_office.png", loc: "现实世界 · 深夜办公室", mood: "calm",
  voice: "vo_question.wav", speaker: "",
  text: [
    "晚上十一点四十，写字楼里只剩你一个人。\n\n报表、加班、房租……日子像一台不会停的复印机，把你的人生一张张印成灰色。",
    "你揉了揉眼睛，正准备关机——\n\n显示器忽然自己亮了。漆黑的屏幕中央，浮现出两行<b>猩红色的字</b>：",
    "<em>「想明白生命的意义吗？想真正的……活着吗？」</em>\n\n字的下面是一个输入框，光标一跳一跳，像心脏的搏动。\n\n你明明没有联网。机箱的风扇声不知何时停了，整个世界安静得可怕。",
  ],
  choices: [
    { t: "输入 YES", sub: "指尖比理智更快", go: "s_yes" },
    { t: "拔掉电源，输入 NO", sub: "这一定是恶作剧", go: "e_mediocre" },
    { t: "抓起手机想报警", sub: "屏幕上的红字突然开始蠕动……", go: "s_office_phone" },
  ],
},
s_office_phone: {
  bg: "img_office.png", loc: "现实世界 · 深夜办公室", mood: "danger", speaker: "",
  text: "你抓起手机——没有信号。日期在跳动：23:59…23:59…23:59…\n\n抬起头时，红字已经占满整块屏幕，像有生命一样缓缓呼吸：\n\n<em>「犹豫，也是一种回答。但机会，只给真正想活的人。」</em>",
  choices: [
    { t: "深吸一口气，输入 YES", go: "s_yes" },
    { t: "砸了电脑夺门而逃", go: "e_mediocre" },
  ],
},
s_yes: {
  bg: "img_office.png", mood: "danger", speaker: "", loc: "？？？？",
  text: [
    "回车键落下的瞬间，世界熄灭了。\n\n没有黑暗，没有声音，甚至没有「你」的感觉。仿佛过了一秒，又仿佛过了一万年——",
    "<em>「确认。轮回编号 0001，载入中……」</em>",
  ],
  choices: [{ t: "……", go: "s_nexus" }],
},

/* ================= 主神空间 ================= */
s_nexus: {
  bg: "img_nexus.png", loc: "主神空间 · 半圆形广场", mood: "calm",
  voice: "vo_rules.wav", speaker: "？？？",
  text: [
    "冷。最先恢复的是触觉——冰冷的金属地面。\n\n你睁开眼：一座<b>巨大得超越常识的半圆形广场</b>。灰黑色金属墙壁向上延伸百米消失在昏暗中，穹顶洒下惨白光柱，照亮聚集在广场中央的十几个人影。",
    "有人穿着西装，有人穿着睡衣，一个大学生模样的男孩在原地发抖。所有人脸上都是同样的茫然。",
    "一个<b>寸头男人</b>走到你面前。黑色作战服，左脸一道浅疤，双手各挎一把银色手枪。他的眼神平静得不像人类。",
    "「新人，欢迎来到<em>主神空间</em>。」他环视众人，「我叫<em>张杰</em>。你们可以理解为——老兵，或者引导者。」",
    "「规则很简单：主神把我们丢进一部又一部的恐怖片世界。完成主线任务、达成隐藏支线，就能获得<em>奖励点数</em>和<em>支线剧情</em>评价。活着回来的人可以用点数兑换任何东西——治疗、武器、血统、超能力，甚至复活死去的同伴。」",
  ],
  choices: [{ t: "「恐怖片世界……是什么意思？」", go: "s_nexus2" }],
},
s_nexus2: {
  bg: "img_nexus.png", loc: "主神空间 · 半圆形广场", mood: "danger", speaker: "张杰",
  text: [
    "「意思是——」他咧嘴笑了笑，笑容里没有温度，「下一秒你们就会明白。」\n\n话音未落，一道血红色光柱从天而降笼罩全场。无数信息直接烙进脑海：",
    "<em>【下一部恐怖片：《生化危机》】\n【主线任务：跟随佣兵小队进入蜂巢，关闭超级电脑「红后」，并活着回到地面。】\n【任务失败：抹杀。】</em>",
  ],
  choices: [
    { t: "「等等！我们没有任何装备——」", sub: "恐慌开始在人群中蔓延", go: "s_weapon" },
    { t: "强迫自己冷静，向张杰要武器", sub: "既然是规则，就有规则的用法", go: "s_weapon" },
  ],
},
s_weapon: {
  bg: "img_nexus.png", loc: "主神空间 · 武器架", mood: "calm", speaker: "张杰",
  text: "张杰打了个响指。广场边缘的金属墙面裂开，三件武器悬浮在半空。\n\n「新人第一次进入，主神允许各选一件基础武器。别挑花眼——<em>能让你活下来的才是好武器。</em>」",
  choices: [
    { t: "◆ 消防斧", sub: "沉重 · 高伤害 · 每一次挥砍都需要直面死亡的勇气", do: st => { st.weapon = "axe"; }, go: "s_warning" },
    { t: "◆ 9mm 手枪", sub: "6发子弹 · 远程稳定 · 打空后就是一块砖头", do: st => { st.weapon = "gun"; }, go: "s_warning" },
    { t: "◆ 军用刺刀军刀", sub: "迅捷 · 连击 · 需要贴身搏命", do: st => { st.weapon = "sword"; }, go: "s_warning" },
  ],
},
s_warning: {
  bg: "img_nexus.png", loc: "主神空间 · 半圆形广场", mood: "calm",
  voice: "vo_warning.wav", speaker: "张杰",
  text: [
    "选好武器的同时，张杰的声音沉了下来。\n\n「最后说一遍规矩，给我记进骨头里：<em>在恐怖片世界里死了，就是真的死了。</em>没有读档，没有重生。主神不会复活新人——一次都不会。」",
    "「还有——」他看着你，疤痕在白光下泛着青色，「跟紧剧情人物。他们是主角，有主角气运护体。离他们太远的新人，通常活不到片尾字幕。」\n\n白光暴涨，吞没一切——",
  ],
  choices: [{ t: "……", go: "s_train" }],
},

/* ================= 地下列车 ================= */
s_train: {
  bg: "img_train.png", loc: "《生化危机》 · 地下列车", mood: "calm", speaker: "",
  text: [
    "柴油味。车轮撞击铁轨的哐当声。\n\n你在一辆行驶中的地下列车里醒来。车厢另一端坐着几名全副武装的佣兵——防弹背心、冲锋枪、战术手电。这不是电影画面：气味、颠簸、枪油味，真实得可怕。",
    "「目标：地下研究所『蜂巢』入口。」戴战术手套的女人利落地检查弹匣，她是<em>蕾恩</em>，「红后把整个蜂巢锁死了。里面五百多名员工……生死不明。」",
    "角落里缩着几个和你一样茫然的新人。张杰靠在车门边，冲你微微点头——认命吧，开始了。",
  ],
  choices: [
    { t: "【支线A】主动坐到蕾恩身边搭话", sub: "了解剧情人物 = 掌握活下去的信息", do: st => { st.flags.A = true; }, go: "s_train_rain" },
    { t: "安静待在角落观察众人", do: () => applySan(-3, "压抑的车厢"), go: "s_mission" },
  ],
},
s_train_rain: {
  bg: "img_train.png", loc: "《生化危机》 · 地下列车", mood: "calm", speaker: "蕾恩",
  text: [
    "「新来的？坐。」蕾恩拍了拍身边的座位，把一支手电扔给你，「待会儿跟着我的背影走。记住三条：<em>别乱跑，别掉队，听到我喊趴下就立刻趴下。</em>」",
    "她顿了顿，压低声音：「说真的……这次任务邪门。五百多人失联，却连一个求救信号都没有。我不喜欢这种安静。」\n\n你获得了她的初步信任。<em>【支线A · 已记录达成条件】</em>",
  ],
  choices: [{ t: "列车减速，抵达蜂巢站台", go: "s_mission" }],
},
s_mission: {
  bg: "img_train.png", loc: "《生化危机》 · 废弃站台", mood: "danger",
  voice: "vo_mission.wav", speaker: "【主神】",
  text: [
    "列车停稳。站台空旷得诡异——应急灯全亮着，却没有一个人影。行李箱东倒西歪，一只婴儿鞋孤零零躺在自动扶梯口。\n\n脑海中响起没有感情的机械音：",
    "<em>【主线任务发布：跟随佣兵小队进入蜂巢，关闭超级电脑「红后」，活着回到地面。】\n【提示：支线剧情隐藏于剧情节点之中，达成可获得额外奖励点数。】</em>",
    "队长<em>一号·马修·艾迪森</em>打出手势，小队呈战术队形展开。「新人跟中间，别碰任何东西。走！」",
  ],
  choices: [{ t: "跟随队伍踏入蜂巢深处", go: "s_corridor" }],
},

/* ================= 蜂巢走廊 · B餐厅 ================= */
s_corridor: {
  bg: "img_corridor.png", loc: "蜂巢 · B区走廊", mood: "danger", speaker: "",
  text: [
    "惨白的日光灯管有一半在闪烁。走廊两侧全是玻璃幕墙，幕墙后是漆黑的实验室，培养槽里泡着说不清的阴影。\n\n空气里有消毒水的味道——以及一丝若有若无的、铁锈般的腥味。",
    "「前面是B餐厅。」一号压低声音，「五百人的基地……不可能连个值班的都没有。都警醒点。」",
  ],
  choices: [
    { t: "【支线B1】凑近幕墙观察实验室内部", sub: "恐惧，但信息就是生命", do: st => { st.flags.B1 = true; }, go: "s_observe_lab" },
    { t: "紧跟队伍前进", go: "s_bhall" },
  ],
},
s_observe_lab: {
  bg: "img_corridor.png", loc: "蜂巢 · B区走廊", mood: "danger", speaker: "",
  text: [
    "你把手电贴上玻璃，强忍寒意照进去——\n\n地板上有大片被拖拽过的暗褐色痕迹，一直延伸进最里面的通风管道。一件白大褂挂在椅背上，胸前工作牌上溅着干涸的血点。",
    "<em>拖拽痕迹的方向性很强……它们不是乱走的。有什么东西在「搬运」。而且——它们行动迟缓，只会直线扑抓。</em>\n\n你默默记下了这些特征。<em>【支线B1 达成：掌握敌人行动规律，战斗命中率提升。】</em>",
  ],
  choices: [{ t: "追上队伍", do: () => applySan(-5, "目睹拖拽血痕"), go: "s_bhall" }],
},
s_bhall: {
  bg: "img_corridor.png", loc: "蜂巢 · B餐厅", mood: "danger", speaker: "",
  text: [
    "B餐厅的门虚掩着。一号用枪尖挑开的瞬间，一股浓烈的腐臭扑面而来。\n\n几十份没吃完的饭菜长满霉斑，餐桌上地上散落着撕碎的工作服……却没有尸体。\n\n「全体戒备。」一号的声音绷紧了。",
    "就在此时——<em>「哐！！」</em>厨房双开门被撞开，一个穿厨师服的<em>东西</em>跌跌撞撞扑出来！灰白的脸，浑浊如死鱼的眼珠，半截舌头耷拉在下巴外——\n\n它直直扑向正在调试设备的<em>卡普兰</em>！",
  ],
  choices: [
    { t: "冲上去救卡普兰！", sub: "近战缠斗 · 保护剧情人物 · 赢得信任", do: st => { st.flags.savedKaplan = true; }, go: "s_fight_zombie1_save" },
    { t: "保持距离出手", sub: "更安全 · 但卡普兰可能挂彩", go: "s_fight_zombie1_far" },
  ],
},
s_fight_zombie1_save: {
  bg: "img_corridor.png", loc: "蜂巢 · B餐厅", mood: "danger", speaker: "⚔ 初次遭遇",
  text: "没有时间思考。你的身体先于大脑动了——<em>这就是恐怖片世界，没有NG，没有重播。</em>",
  fightCfg: {
    id: "zombie1", name: "厨师丧尸", hp: 34, dmg: [7, 13],
    reward: 10, rewardWhy: "首次击杀丧尸",
    intro: "丧尸的头骨以不自然的角度歪着，却依然朝卡普兰的方向嘶吼挣扎。",
    winScene: "s_after_zombie1_save", deathScene: "e_zombies",
  },
},
s_fight_zombie1_far: {
  bg: "img_corridor.png", loc: "蜂巢 · B餐厅", mood: "danger", speaker: "⚔ 初次遭遇",
  text: "你和那东西之间隔着两张餐桌——这份距离是你全部的安全感。<em>稳住，瞄准头部。</em>",
  fightCfg: {
    id: "zombie1", name: "厨师丧尸", hp: 34, dmg: [7, 13],
    reward: 10, rewardWhy: "首次击杀丧尸",
    intro: "丧尸撞翻桌椅，踩着满地餐具朝你挪来。",
    winScene: "s_after_zombie1_far", deathScene: "e_zombies",
  },
},
s_after_zombie1_save: {
  bg: "img_corridor.png", loc: "蜂巢 · B餐厅", mood: "danger", speaker: "卡普兰",
  text: [
    "你从侧面狠狠一击，那东西应声倒地，抽搐两下不再动弹。\n\n「呼……呼……谢了兄弟！」卡普兰瘫坐在地，脸色惨白，又猛地抬头盯着你，「等等——你刚才根本没犹豫。你以前是干什么的？」",
    "蕾恩走过来踢了踢尸体，若有所思地看了你一眼：「下手干净利落……新兵可做不到。」\n\n<em>队伍对你的信任提升了。</em>但你近距离看清了它的脸——那真的是「人」。胃里一阵翻涌。",
  ],
  choices: [{ t: "压下恶心，检查厨房", do: () => applySan(-6, "近距离直面丧尸"), go: "s_find_adrenaline" }],
},
s_after_zombie1_far: {
  bg: "img_corridor.png", loc: "蜂巢 · B餐厅", mood: "danger", speaker: "",
  text: [
    "那东西倒了下去。但就在倒下前的瞬间，它的指甲擦过了卡普兰的小臂——划出一道血痕。\n\n「该死！没事，我没事！」卡普兰咬牙包扎，看向你的眼神复杂，「下次……能再快一点吗？」",
    "一号皱眉环视：「都听好了，这地方的东西<em>不怕疼、不死透</em>。瞄准头部，确认摧毁。」",
  ],
  choices: [{ t: "搜查厨房", go: "s_find_adrenaline" }],
},
s_find_adrenaline: {
  bg: "img_corridor.png", loc: "蜂巢 · B餐厅厨房", mood: "danger", speaker: "",
  text: [
    "厨房操作台上翻倒着急救箱，大部分药品已经过期。你在抽屉夹层摸到一支<em>肾上腺素注射器</em>和半瓶碘伏。\n\n标签还清晰可见：抗休克用。你把它收进口袋——不知道为什么，直觉告诉你它会救某个人的命。",
  ],
  choices: [
    { t: "收好肾上腺素", sub: "【道具获得】肾上腺素注射器", do: st => { st.flags.adrenaline = true; }, go: "s_to_redqueen" },
    { t: "没敢多拿，跟上队伍", go: "s_to_redqueen" },
  ],
},

/* ================= 红后机房 · 激光通道 ================= */
s_to_redqueen: {
  bg: "img_redqueen.png", loc: "蜂巢 · 中央机房", mood: "danger", speaker: "",
  text: [
    "队伍深入到蜂巢最核心处。一座巨大的<em>球形机房</em>出现——环状金属走廊悬在深渊般的中庭之上。中央平台上，幽蓝色的全息投影微微闪烁。",
    "那是一个小女孩的形象。蓝裙，长发，眼睛一眨不眨地「看」着每一个闯入者。\n\n「<em>红后</em>。」一号吐出一口气，「超级电脑。关掉她，封锁才会解除。」",
    "「你们正在犯一个错误。」童声突然响起，温柔得毛骨悚然，「<em>我只是在执行隔离协议。放我出去，外面的世界会毁灭。</em>」\n\n没人回应。卡普兰接上破解终端，手指翻飞。",
  ],
  choices: [
    { t: "【支线B2】趁破解间隙研究那条玻璃通道的结构图", sub: "墙上的检修示意图……为什么让人不安", do: st => { st.flags.B2 = true; }, go: "s_laser_observed" },
    { t: "持枪警戒四周", go: "s_shutdown" },
  ],
},
s_laser_observed: {
  bg: "img_redqueen.png", loc: "蜂巢 · 机房外 · 检修通道口", mood: "danger", speaker: "",
  text: [
    "你盯着通往配电室的<em>玻璃幕墙通道</em>——地面导轨上有规律的灼烧痕，天花板嵌着一格一格的暗格，四壁玻璃厚得不正常。",
    "<em>这不是普通通道。防御系统的攻击必然分层推进——低位、高位、全覆盖……</em>\n\n你在心里把推演练了三遍，直到卡普兰喊道：「搞定！红后系统关——等等，为什么备用电源还在给什么东西供电？！」\n\n<em>【支线B2 达成：你看懂了激光通道的机关规律。】</em>",
  ],
  choices: [{ t: "「大家小心！那条通道有问题——」", go: "s_shutdown" }],
},
s_shutdown: {
  bg: "img_redqueen.png", loc: "蜂巢 · 中央机房", mood: "danger", speaker: "红后",
  text: [
    "「求你们了。」红后的投影剧烈闪烁，「<em>不要关掉我。</em>」\n\n「关掉她。」一号下令。\n\n卡普兰按下回车。全息影像凝固、碎裂成数据雪花。机房陷入短暂的寂静——",
    "然后，整座蜂巢的灯光转为暗红。<em>断电导致所有门禁同时失效。</em>\n\n耳机里传来外围组惊恐的呼叫：「B区的门全开了！它们在往上爬——大量接触！重复，大量接触——啊！！」\n\n「必须去配电室手动重启隔离闸门！」一号当机立断，「全体跟我来！唯一的路径是那条玻璃通道——快！」",
  ],
  choices: [{ t: "跟着队伍冲进玻璃通道", go: "s_laser_cine" }],
},
s_laser_cine: {
  video: "vid_laser.mp4", cineLabel: "过场 · 激光通道（MiniMax H3 本地生成）", mood: "danger",
  bg: "img_laser.png", loc: "蜂巢 · 激光通道", speaker: "",
  text: [
    "蓝白色的光在你视网膜上留下残影。刚才那一幕快得不真实——仿佛整条通道本身活了过来，用光做刀，把空气切成一段一段。",
    "队伍被迫停在通道中段。前方还有二十米就是配电室的门。空气里有细微的电流嗡鸣声，像毒蛇吐信前的嘶声。",
  ],
  choices: [{ t: "握紧武器，继续前进", go: "s_laser" }],
},
s_laser: {
  bg: "img_laser.png", loc: "蜂巢 · 激光通道", mood: "danger", speaker: "",
  text: [
    "走到一半时，走在最前的<em>一号</em>突然举起拳头——全员急停。\n\n配电室的安全门近在咫尺。但嗡鸣声陡然拔高——<em>第一段防御启动了。</em>",
  ],
  choices: [{ t: "▸ 判断攻击模式并应对！", sub: "连续三波 · 错两次 = 死亡", go: "s_laser_q1" }],
},
s_laser_q1: {
  bg: "img_laser.png", loc: "蜂巢 · 激光通道 · 第一波", mood: "danger", speaker: "【防御启动 · 第一波】",
  text: S => (S.flags.B2
    ? "你的瞳孔骤缩——天花板暗格缩开的方式、导轨灼痕的高度……<em>你在结构图前推演过这一幕！低位扫掠层！</em>\n\n一道蓝色光刃贴着地面<em>水平横扫而来</em>——"
    : "天花板暗格缩开，一道蓝色光刃贴着地面<em>水平横扫而来</em>——快得几乎看不见！")
    + "\n\n<em>一瞬间，你该怎么做？</em>",
  choices: [
    { t: "向上跳跃，扒住侧壁管道", sub: "赌它扫的是低位", go: S => laserJudge("jump", "s_laser_q2") },
    { t: "整个人贴地趴下", sub: "赌它扫的是高位", go: S => laserJudge("duck", "s_laser_q2") },
    { t: "拉着身边人向后急退", sub: "先脱离扇面再说", go: S => laserJudge("back", "s_laser_q2") },
  ],
},
s_laser_q2: {
  bg: "img_laser.png", loc: "蜂巢 · 激光通道 · 第二波", mood: "danger", speaker: "【防御启动 · 第二波】",
  text: S => {
    const base = "第一道光刃擦着<em>J.D.</em>的发梢掠过，在身后墙上切出一条焦黑切口。所有人都僵住了——还没完。\n\n嗡鸣声再次变调：<em>第二波，从头顶暗格里斜劈而下的交叉网刀！</em>";
    return base + "\n\n<em>趴下？跳开？还是贴住墙角死角？！</em>";
  },
  choices: [
    { t: "贴地滑铲从网刀下方穿过", go: S => laserJudge("slide", "s_laser_q3") },
    { t: "原地起跳抓住顶部导轨", go: S => laserJudge("hang", "s_laser_q3") },
    { t: "缩进两面墙夹角的死角", go: S => laserJudge("corner", "s_laser_q3") },
  ],
},
s_laser_q3: {
  bg: "img_laser.png", loc: "蜂巢 · 激光通道 · 最终波", mood: "danger", speaker: "【防御启动 · 最终波】",
  text: S => "两次闪避耗光了所有人的运气。配电室的白光已经从门缝里漏进来——就差一步。\n\n嗡鸣声这一次沉闷得可怕，整个通道的暗格<em>同时</em>缩开：<em>最终波，全网格收拢。无差别覆盖。</em>\n\n一号猛然回头，眼睛死死盯着通道中段一根凸出的承重梁——和梁后那一小块、唯一一小块阴影。",
  choices: [
    { t: "冲向承重梁后的阴影", go: S => laserJudge("beam", "s_laser_end") },
    { t: "赌一把，全速冲刺向安全门", go: S => laserJudge("dash", "s_laser_end") },
    { t: "转身向来路狂奔", go: S => laserJudge("flee", "s_laser_end") },
  ],
},
s_laser_end: {
  bg: "img_laser.png", loc: "蜂巢 · 配电室", mood: "danger", speaker: "",
  text: S => {
    let extra = "";
    if (S.laserFails === 0 && S.flags.B2) {
      extra = "因为你的预警和判断，J.D. 和另一名佣兵全程毫发无伤。<em>【支线B2 完美达成：额外保全一名队员】</em>";
      S.flags.savedExtra = true;
    } else if (S.laserFails === 0) {
      extra = "全员惊魂未定，但都活着。一号深深看了你一眼：「反应不错。」";
    }
    return "隔离闸门轰然落下，把嘶吼挡在了外面。配电室里只剩下众人粗重的喘息声。" + extra;
  },
  choices: [{
    t: "重启隔离系统", go: "s_after_laser",
    do: () => {
      applySan(-12, "直面激光通道的死亡");
      if (S.flags.savedExtra) gainPts(50, "保全额外队员");
    },
  }],
},
/* ---- 激光失误分支 ---- */
s_fail_q1: {
  bg: "img_laser.png", loc: "蜂巢 · 激光通道 · 第一波", mood: "danger", speaker: "",
  text: S => [
    "<em>你判断错了。</em>\n\n光刃从完全意想不到的角度掠过——剧痛！你被掀翻在水一样的冷光里，左肋像被烧红的铁尺贯穿。",
    "混乱中，新人<em>J.D.</em>惊慌失措地跑进了第二个扇面。蓝光闪过。\n\n他没有再叫出声。所有人都看见了那两半缓缓滑落的影子。<em>【J.D. 阵亡】</em>",
    "「别停下！！」蕾恩嘶吼着把你拽起来。主神不会给哀悼的时间——<em>第二波已经启动了。</em>（体力 -35 · 理智 -8）",
  ].join("\n\n"),
  choices: [{ t: "咬牙跟上，应对第二波！", do: () => { hurtPlayer(35, "e_laser"); applySan(-8); killTeam("jd"); }, go: "s_laser_q2" }],
},
s_fail_q2: {
  bg: "img_laser.png", loc: "蜂巢 · 激光通道 · 第二波", mood: "danger", speaker: "",
  text: S => [
    "<em>又错了！</em>\n\n交叉网刀的轨迹和你预判的差了半米。你侧身翻滚躲过要害，肩背还是被燎出三道深可见骨的焦痕。（体力 -35 · 理智 -8）\n\n<em>你已经没有第三次犯错的余地了。</em>",
  ].join("\n\n"),
  choices: [{ t: "撑住！最后一波！", do: () => { hurtPlayer(35, "e_laser"); applySan(-8); }, go: "s_laser_q3" }],
},
s_fail_q3: {
  bg: "img_laser.png", loc: "蜂巢 · 激光通道 · 最终波", mood: "danger", speaker: "",
  text: S => [
    "你在最后关头扑偏了——光网贴着头皮收拢，发梢和衣角化作飞灰，小腿被灼出一道血槽！（体力 -30）\n\n<em>系统正在充能第二次收拢。这是最后一次机会——看清楚承重梁的位置！</em>",
  ].join("\n\n"),
  choices: [{ t: "重新判断！", do: () => { hurtPlayer(30, "e_laser"); applySan(-6); }, go: "s_laser_q3" }],
},
s_after_laser: {
  bg: "img_redqueen.png", loc: "蜂巢 · 机房回廊", mood: "danger", speaker: "",
  text: S => {
    killTeam("one");
    return [
      "系统重启的提示音响起时，异变突生——<em>残余的一套防御模块突然激活。</em>最后一道网格无声无息地从侧面封死了回廊！",
      "网格收拢的方向正对着毫无察觉、还在操作终端的<em>卡普兰</em>。\n\n「小心——！」\n\n一号几乎没有犹豫。他一把推开卡普兰，自己却被光网罩了个正着。",
      "没有惨叫。光刃过处，一切都太安静了。\n\n「队长！！」蕾恩的怒吼在金属回廊里回荡，惊起一片嗡鸣的电子音。你攥紧武器，指节发白——这就是<em>剧情的力量</em>吗？该发生的，终究会发生……",
      "一号·马修·艾迪森，阵亡。<em>【剧情角色死亡：主线不受影响，但你亲眼见证了「剧情修正力」。】</em>",
    ].join("\n\n");
  },
  choices: [{ t: "「我们还得继续……走水道撤向列车！」", do: () => applySan(-8, "一号之死"), go: "s_waterway" }],
},

/* ================= 水道逃亡 · 丧尸群 ================= */
s_waterway: {
  bg: "img_corridor.png", loc: "蜂巢 · 实验室水道", mood: "danger", speaker: "",
  text: [
    "齐踝深的冷却水里，一行人蹚过幽暗的设备层。手电的光柱扫过之处，水面漂着碎裂的培养槽玻璃。\n\n「听——」蕾恩突然停步。",
    "黑暗深处传来此起彼伏的、湿漉漉的脚步声。<em>越来越多。</em>\n\n手电光尽头，白大褂的影子一个接一个从管道后转出来，浑浊的眼珠反射着光——<em>丧尸群，至少几十只，堵住了唯一的退路。</em>",
  ],
  choices: [
    { t: "「我从正面开路——你们掩护！」", go: "s_fight_horde" },
    { t: "「炸掉管线！制造混乱冲过去！」", sub: "需要有人断后……", go: "s_fight_horde" },
  ],
},
s_fight_horde: {
  bg: "img_corridor.png", loc: "蜂巢 · 水道遭遇战", mood: "danger", speaker: "⚔ 群体战斗",
  text: "没有退路了。身后的铁栅栏已经被咬得咯咯作响——<em>要么撕开一条血路，要么留在这里。</em>",
  fightCfg: {
    id: "horde", name: "丧尸群（先锋）", hp: 55, dmg: [11, 17],
    reward: 20, rewardWhy: "突破丧尸群封锁",
    intro: "最前面的几只扑进水里溅起大片水花，后面源源不断……",
    rageAt: 25, rageText: "更多的丧尸从管道里涌出来了——它们被血腥味刺激得疯狂！",
    winScene: "s_rain_bitten", deathScene: "e_zombies",
  },
},
s_rain_bitten: {
  bg: "img_train.png", loc: "蜂巢 · 站台", mood: "danger", speaker: "",
  text: [
    "你踉跄着冲上站台时，队伍已经减员过半。幸存的佣兵用火力封住水道入口，铁门缓缓落下。\n\n「咳咳……都上来了吗？报数——」蕾恩靠在立柱上喘息，忽然动作一顿。",
    "她的目光落在自己的<b>小臂</b>上。\n\n一圈清晰的<em>齿痕</em>，正在渗血。不知道什么时候——哪一只——\n\n「……该死。」她扯下绷带开始草草包扎，声音异常平静，「看什么看？<em>被咬了就是被咬了。</em>继续走。」\n\n所有人都明白那意味着什么。车厢里死一般寂静。",
  ],
  choices: [
    { t: "掏出肾上腺素为她注射", sub: "【需要道具】延缓感染 · 支线C", if: st => st.flags.adrenaline,
      do: st => { st.flags.C = true; }, go: "s_adrenaline_used" },
    { t: "默默递上碘伏帮她处理伤口", do: () => applySan(-10, "无能为力的愧疚"), go: "s_final_station" },
    { t: "别过头假装没看见", do: () => applySan(-14, "逃避"), go: "s_final_station" },
  ],
},
s_adrenaline_used: {
  bg: "img_train.png", loc: "蜂巢 · 站台", mood: "danger", speaker: "蕾恩",
  text: [
    "「你什么时候拿的这个？」蕾恩愣了一下，随即自嘲地笑笑，撸起袖子伸出手，「行吧。死马当活马医。」\n\n针管推下去，她的脸色肉眼可见地好了几分，连呼吸都平稳下来。",
    "「……谢了。」她别过脸去，声音很低，「如果……我是说如果，我变成了那种东西——<em>别犹豫，直接开枪。</em>这是命令。」\n\n<em>【支线C 达成：感染进程被大幅延缓。蕾恩暂时安全了。】</em>",
  ],
  choices: [{ t: "远处传来一声不属于人类的尖啸——", go: "s_boss_intro" }],
},

/* ================= BOSS · 舔食者 ================= */
s_final_station: {
  bg: "img_train.png", loc: "蜂巢 · 站台", mood: "danger", speaker: "",
  text: "列车就停在五十米外，车头的灯刺破黑暗。只要跑过去，就结束了——\n\n然后你听见了那个声音。<em>金属被利爪划开的、令人牙酸的锐响。</em>\n\n它来自头顶。",
  choices: [{ t: "抬起头", go: "s_boss_intro" }],
},
s_boss_intro: {
  video: "vid_licker.mp4", cineLabel: "过场 · 它来了（MiniMax H3 本地生成）", mood: "danger",
  bg: "img_licker.png", loc: "站台 · 天花板上的东西", speaker: "",
  text: [
    "手电光柱扫过天花板的瞬间，所有人都僵住了。\n\n<em>它</em>倒挂在管道上——赤红的肌腱代替了皮肤，暴露的脑组织随着呼吸搏动，一条细长的舌头垂下来，在空气中缓慢地品鉴着你们的气味。",
    "「舔食者……」卡普兰的声音在抖，「T病毒对它不是杀死，是<em>重塑</em>……」\n\n它松开了爪子。",
  ],
  choices: [{ t: "⚔ 迎战！", go: "s_boss" }],
},
s_boss: {
  bg: "img_licker.png", loc: "决战 · 舔食者", mood: "danger", speaker: "⚔ BOSS 战",
  text: S => `它落地时砸出一个凹坑，肌肉纤维如活蛇般蠕动。蕾恩举枪掩护，卡普兰在远处大喊着为你指示它的动向——\n\n<em>生死，在此一战。</em>${S.geneLockUsed ? "" : "\n\n（若体力濒危且理智尚存，或将触发某种觉醒……）"}`,
  fightCfg: {
    id: "licker", name: "舔食者", hp: 112, dmg: [15, 22],
    reward: 500, rewardWhy: "击杀变异体「舔食者」",
    intro: "它的长舌「啪」地抽碎了你脚边的水泥碴。",
    rageAt: 55, rageText: "舔食者发出刺耳的尖啸，暴起发难——速度陡然翻倍！",
    onRage: (st, f) => {
      if (st.weapon === "gun") {
        WEAPONS.gun.ammo += 4;
        flog(`卡普兰从掩体后扔来一个备用弹匣！<span class="hit">（弹药 +4）</span>`);
      }
      if (st.weapon === "axe") {
        flog(`蕾恩的点射逼得它侧身一滞——<span class="hit">机会！</span>`);
      }
      if (st.weapon === "sword") {
        flog(`它在管道间借力反弹，攻势愈发刁钻！`);
      }
    },
    finisherName: S => S.weapon === "axe" ? "斧劈头颅" : "致命一击",
    finisherIf: (st, f) => f.hp <= 28,
    finisherDesc: S => S.weapon === "axe"
      ? "你踩着它扑击的势头腾身跃起，消防斧带着全身的重量与不甘、恐惧、愤怒，狠狠噼进那裸露的大脑深处——直到斧刃穿透，钉进水泥地。它抽搐着，不动了。"
      : "你瞅准它长舌回收的刹那欺身而上，将全部力量灌进最后一击——直贯那暴露在外的脑组织深处。巨大的身躯轰然倒塌。",
    winScene: "s_escape_train", deathScene: "e_licker",
  },
},
s_escape_train: {
  bg: "img_train.png", loc: "逃离蜂巢", mood: "calm", speaker: "",
  text: [
    "车门合拢的汽笛声，是你听过最美妙的声音。\n\n列车冲破封锁驶向地面。车厢里，幸存者们东倒西歪，没有人说话。蕾恩靠着车壁闭目养神——她的呼吸依然平稳。卡普兰抱着头盔，肩膀微微耸动。",
    "你摊开手掌，发现掌心被斧柄（枪身／刀柄）硌出了血痕，居然到现在才觉得疼。\n\n窗外，黑暗的地道飞速倒退。然后——<em>白光暴涨。</em>",
  ],
  choices: [{ t: "……", go: "s_settle" }],
},

/* ================= 结算 ================= */
s_settle: {
  overlay: {
    voice: "vo_settle.wav",
    html: S => buildSettleHtml(S),
    buttons: S => [
      {
        t: "查 看 主 神 空 间", do: () => showCard(
          `<h2>主 神 空 间</h2>
           <p>白光散去，半圆形广场依旧冰冷。张杰抱着手臂等你，嘴角挂着意味不明的笑。</p>
           <p style="color:#ffd76a">「不错，第一次就能活着回来，还清掉了那只舔食者。」他上下打量你，「知道吗，你这批新人里，<em>回来的不到一半</em>。」</p>
           <p>他随手一挥，广场上浮现出巨大的兑换目录光影：</p>
           <table class="statTable">
             <tr><td>细胞活力强化（体质提升）</td><td>800 点</td></tr>
             <tr><td>基因锁第一阶段·自主开启权</td><td>2000 点 + D级支线×1</td></tr>
             <tr><td>血统类：初级吸血鬼血统</td><td>3000 点 + C级支线×1</td></tr>
             <tr><td>复活一名本次阵亡的同伴</td><td>4000 点</td></tr>
           </table>
           <p style="color:#ff8a8a">「攒够点数再来吧，新人。」张杰转身走向黑暗，声音飘回来，「对了——我叫张杰。<em>记住这个名字。</em>以后你会明白，引导者的意思……呵呵。」</p>
           <p>他的背影消失在光柱之外。那句轻笑让你莫名觉得脊背发凉——他知道的，远比他说出来的多。</p>
           <p style="text-align:center;color:#888;margin-top:14px">休息期：20 天<br>—— 血色光柱再次亮起 ——<br><b style="color:var(--blood);font-size:18px">下一部恐怖片：《咒 怨》</b><br><span style="color:#555">无限的世界，无限的轮回。第一章 · 完</span></p>`,
          [{ t: "进 入 下 一 次 轮 回 ▶", do: () => backToTitle() }]
        ),
      },
    ],
  },
},

/* ================= 死亡结局 ================= */
e_mediocre: {
  overlay: {
    death: { title: "平庸之死", cause: "拒绝了血色的邀请，回到了灰色的日常" },
    html: () => `<h2>平 庸 之 死</h2>
      <p>屏幕熄灭了。红字消失了，就像从未出现过。</p>
      <p>你瘫在工位上笑了很久，笑到眼泪都出来——第二天照常上班，照常加班，照常在深夜的写字楼里枯坐。<em>只是从此，每当深夜屏幕反光的瞬间，你都会心悸。</em></p>
      <p style="color:#666">许多年后你会在某个平凡的黄昏忽然想起那个夜晚，想起自己曾经离「真正的活着」只有一次点击的距离。<br>——但主神不给第二次机会。</p>`,
    buttons: () => [{ t: "重 新 开 始", do: () => backToTitle() }],
  },
},
e_laser: {
  overlay: {
    death: { title: "光中之刃", cause: "激光通道判断失误，被防御网格切裂" },
    html: () => `<h2>光 中 之 刃</h2>
      <p>你只来得及看见一片刺目的蓝白色。</p>
      <p>没有痛觉。切割过于锋利时，神经甚至来不及传递任何东西。视野分成几块倾斜的碎片，每一块里都是同一张震惊的、逐渐远去的脸。</p>
      <p style="color:#666">【轮回记录】防御系统不分新人和主角。它只区分——反应快的人，和死人。</p>`,
    buttons: () => [{ t: "轮 回 重 启", do: () => backToTitle() }],
  },
},
e_zombies: {
  overlay: {
    death: { title: "噬咬之终", cause: "被丧尸拖入黑暗" },
    html: () => `<h2>噬 咬 之 终</h2>
      <p>最先失去的是武器，然后是平衡，最后是天空的视野。</p>
      <p>无数只手把你拖进冰冷的黑暗里。牙齿撕开皮肉的剧痛中，你听见蕾恩喊你的名字，听见枪声，听见很多很多的脚步声越来越远——</p>
      <p style="color:#666">【轮回记录】在恐怖片世界里，「差点救到」和「没救到」是同一个意思。</p>`,
    buttons: () => [{ t: "轮 回 重 启", do: () => backToTitle() }],
  },
},
e_licker: {
  overlay: {
    death: { title: "天花板上的眼睛", cause: "死于舔食者的利爪与长舌之下" },
    html: () => `<h2>天 花 板 上 的 眼 睛</h2>
      <p>长舌卷住脚踝的瞬间你就明白了结局。它把你提起来的时候，你看见了自己悬在半空的倒影——映在那颗搏动的、裸露的大脑下方的血泊里。</p>
      <p>最后的画面，是蕾恩红了眼的枪口和卡普兰绝望的脸。</p>
      <p style="color:#666">【轮回记录】基因锁会在绝境中开启——前提是，你还剩一口气去开启它。</p>`,
    buttons: () => [{ t: "轮 回 重 启", do: () => backToTitle() }],
  },
},
e_sancollapse: {
  overlay: {
    death: { title: "心碎而止", cause: "理智崩溃，精神先于肉体死亡" },
    html: () => `<h2>心 碎 而 止</h2>
      <p>恐惧是有额度的。当最后一点额度耗尽，大脑选择了一劳永逸的解决方案——关机。</p>
      <p>你笑着坐在血泊边，任凭谁喊都不回应。佣兵们不得不留下一人陪着你，错过了撤离时限。<br><em>警报响起的时候，你还在笑。</em></p>
      <p style="color:#666">【轮回记录】在这个世界里，理智和体力一样，是需要管理的战略资源。</p>`,
    buttons: () => [{ t: "轮 回 重 启", do: () => backToTitle() }],
  },
},
e_generic: {
  overlay: {
    death: { title: "无名之死", cause: "倒在了无人知晓的角落" },
    html: () => `<h2>无 名 之 死</h2><p>黑暗吞没了一切。主神的记录里，只多了一行小小的名字。</p>`,
    buttons: () => [{ t: "轮 回 重 启", do: () => backToTitle() }],
  },
},
};

/* ================= 激光判定逻辑 =================
   每题正确答案；q1 额外允许 back（B2 支线推演过退离扇面同样安全）
   失误1次：进入对应 s_fail_qN（重伤+J.D.阵亡/再给机会）
   累计失误2次：死亡结局 */
function laserJudge(answer, next) {
  const correct = { s_laser_q1: "jump", s_laser_q2: "slide", s_laser_q3: "beam" }[S.sceneId];
  if (answer === correct || (S.sceneId === "s_laser_q1" && answer === "back")) {
    return next;
  }
  if (S.laserFails >= 1) { S.laserFails++; return "e_laser"; }
  S.laserFails++;
  if (S.sceneId === "s_laser_q1") return "s_fail_q1";
  if (S.sceneId === "s_laser_q2") return "s_fail_q2";
  return "s_fail_q3";
}
function backToTitle() {
  try { localStorage.removeItem("whc1_save"); } catch (e) {}
  location.reload();
}
