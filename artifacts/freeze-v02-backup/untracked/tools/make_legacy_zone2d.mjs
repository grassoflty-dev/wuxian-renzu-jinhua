/* make_legacy_zone2d.mjs — 由当前 zone2d.js 还原出「改动前」的程序化梯形腿版本，
 * 仅写到 tools/_zone2d_legacy.js（不进仓库源码），供 A/B 对照渲染。 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const SRC = path.join(ROOT, "server-rs", "ui", "js", "zone2d.js");
const DST = path.join(ROOT, "tools", "_zone2d_legacy.js");

const NEW_CALL = [
  "    // 腿部：立绘下半身像素蒙皮（取自同一张立绘，与上身同像素管线，非几何梯形）。",
  "    // 先画腿再画躯干 → 髋关节被躯干盖住，摆腿时看不到接缝。",
  "    drawLegs(g, now, w, h);",
  ""
].join("\n");

const ANCHOR = '      g.fillStyle = "#e8b98a"; g.fillRect(-6, 0, 12, 12);\n    }\n';

const OLD = `    // 腿部动画：双腿两段式（大腿+小腿）剪刀式迈步，带提膝弯曲（拳皇/魂斗罗/我的世界式行走）
    if (state === "run") {
      const t = now / 90;                          // 步频相位（90ms 一弧度）
      const hipY = Math.round(h * 0.54), thighLen = 24, shinLen = 24;
      // 锥形腿段：从(x0,y0)到(x1,y1)，起始宽 w0 → 末端宽 w1（髋宽膝窄踝更窄，非方块）
      const seg = (x0, y0, x1, y1, w0, w1, color) => {
        const ang = Math.atan2(x1 - x0, y1 - y0);
        const len = Math.hypot(x1 - x0, y1 - y0) || 1;
        g.save(); g.translate(x0, y0); g.rotate(ang);
        g.fillStyle = color;
        g.beginPath();
        g.moveTo(-w0 / 2, 0); g.lineTo(w0 / 2, 0);
        g.lineTo(w1 / 2, len); g.lineTo(-w1 / 2, len);
        g.closePath(); g.fill();
        // 前缘高光（细亮带，去"平块"感，带圆柱体积）
        g.fillStyle = "rgba(255,255,255,0.10)";
        g.beginPath();
        g.moveTo(-w0 / 2 + 1, 1); g.lineTo(-w0 / 2 + 1 + Math.max(1.5, w0 * 0.25), 1);
        g.lineTo(-w1 / 2 + 1 + Math.max(1.5, w1 * 0.25), len - 1); g.lineTo(-w1 / 2 + 1, len - 1);
        g.closePath(); g.fill();
        g.restore();
      };
      // 前腿（近侧）与后腿（远侧），相位差 π 反相摆动
      for (const [hx, ph, body, shoe] of [
        [-9, t + Math.PI, "#34434f", "#293742"],   // 后腿（远侧，深裤色）
        [1, t, "#47596c", "#384859"],              // 前腿（近侧，略亮辨前后，贴近上身亮度）
      ]) {
        const thighAng = Math.sin(ph) * 0.55;      // 大腿摆角 ±0.55 rad
        const kneeX = hx + Math.sin(thighAng) * thighLen;
        const kneeY = hipY + Math.cos(thighAng) * thighLen;
        const kneeBend = Math.max(0, Math.sin(ph + Math.PI / 2)) * 0.7;   // 提膝弯曲（略柔和）
        const shinAng = thighAng - kneeBend;       // 小腿相对大腿后折
        const footX = kneeX + Math.sin(shinAng) * shinLen;
        const footY = kneeY + Math.cos(shinAng) * shinLen;
        // 大腿（髋13→膝7）+ 小腿（膝7→踝4）：细长锥形，贴近真腿
        seg(hx, hipY, kneeX, kneeY, 13, 7, body);
        seg(kneeX, kneeY, footX, footY, 7, 4, body);
        // 膝盖（略亮小圆，突出膝盖弯曲）
        g.fillStyle = shoe;
        g.beginPath(); g.arc(kneeX, kneeY, 3.5, 0, Math.PI * 2); g.fill();
        // 鞋：沿小腿方向前伸，鞋头朝前（非方块）
        g.fillStyle = shoe;
        g.save(); g.translate(footX, footY); g.rotate(shinAng);
        g.beginPath();
        g.moveTo(-3, -2); g.lineTo(10, -2); g.lineTo(13, 2); g.lineTo(-4, 2);
        g.closePath(); g.fill(); g.restore();
      }
    }
`;

let s = fs.readFileSync(SRC, "utf8");
if (!s.includes(NEW_CALL)) throw new Error("找不到新的 drawLegs 调用点");
if (!s.includes(ANCHOR)) throw new Error("找不到躯干绘制锚点");
s = s.replace(NEW_CALL, "");
s = s.replace(ANCHOR, ANCHOR + OLD);
fs.writeFileSync(DST, s, "utf8");
console.log("written", DST, s.length);
