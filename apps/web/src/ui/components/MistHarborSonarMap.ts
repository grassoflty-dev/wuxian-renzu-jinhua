import { signalWraithInterference, invalidSignalWraithProjection } from "../../protocol/PresentationEventValidation.js";
import type { ExploredMapProjection, Vec3, WorldSnapshotV3 } from "../../protocol/types.js";

const SVG_NS = "http://www.w3.org/2000/svg";
const LOCAL_MAP = "information.local_map_i";
const SIGNAL_HAZARD = "mh_signal_interference_region";

interface Point { x: number; z: number }
interface Room { id: string; outline: Point[]; center: Point }
interface Connection { from: Point; to: Point }
interface Marker { kind: "objective" | "signal_region"; position: Point }
interface SonarGeometry { rooms: Room[]; connections: Connection[]; player: Point; objectives: Point[] }

export interface MistHarborSonarView {
  status: "unauthorized" | "unexplored" | "ready";
  rooms: readonly Room[];
  connections: readonly Connection[];
  player: Point | null;
  markers: readonly Marker[];
}

const empty = (status: "unauthorized" | "unexplored"): MistHarborSonarView =>
  ({ status, rooms: [], connections: [], player: null, markers: [] });

function point(value: Vec3 | undefined): Point | null {
  return value && [value.xM, value.yM, value.zM].every(Number.isFinite)
    ? { x: value.xM, z: value.zM } : null;
}

function readGeometry(map: ExploredMapProjection): SonarGeometry | null {
  if (!Array.isArray(map.rooms) || map.rooms.length === 0 ||
      !Array.isArray(map.connections) || !Array.isArray(map.objectives)) return null;
  const player = point(map.playerPositionM);
  if (!player) return null;
  const rooms: Room[] = [];
  const roomIds = new Set<string>();
  for (const room of map.rooms) {
    if (!room || typeof room.roomId !== "string" || !room.roomId || roomIds.has(room.roomId) ||
        !Array.isArray(room.outlineM) || room.outlineM.length < 3) return null;
    const outline = room.outlineM.map(vertex => point(vertex));
    if (outline.some(vertex => vertex === null)) return null;
    const vertices = outline as Point[];
    const twiceArea = vertices.reduce((sum, vertex, index) => {
      const next = vertices[(index + 1) % vertices.length]!;
      return sum + vertex.x * next.z - next.x * vertex.z;
    }, 0);
    if (!Number.isFinite(twiceArea) || Math.abs(twiceArea) <= 1e-9) return null;
    const center = { x: vertices.reduce((sum, vertex) => sum + vertex.x, 0) / vertices.length,
      z: vertices.reduce((sum, vertex) => sum + vertex.z, 0) / vertices.length };
    if (!Number.isFinite(center.x) || !Number.isFinite(center.z)) return null;
    roomIds.add(room.roomId);
    rooms.push({ id: room.roomId, outline: vertices, center });
  }
  const byId = new Map(rooms.map(room => [room.id, room.center]));
  const connections: Connection[] = [];
  const connectionIds = new Set<string>();
  for (const link of map.connections) {
    if (!link || typeof link.connectionId !== "string" || !link.connectionId ||
        connectionIds.has(link.connectionId) || link.fromRoomId === link.toRoomId) return null;
    const from = byId.get(link.fromRoomId);
    const to = byId.get(link.toRoomId);
    if (!from || !to) return null;
    connectionIds.add(link.connectionId);
    connections.push({ from, to });
  }
  const objectives: Point[] = [];
  const objectiveIds = new Set<string>();
  for (const objective of map.objectives) {
    if (!objective || typeof objective.objectiveId !== "string" || !objective.objectiveId ||
        objectiveIds.has(objective.objectiveId)) return null;
    const position = point(objective.positionM);
    if (!position) return null;
    objectiveIds.add(objective.objectiveId);
    objectives.push(position);
  }
  const all = rooms.flatMap(room => room.outline);
  const span = Math.max(
    Math.max(...all.map(vertex => vertex.x)) - Math.min(...all.map(vertex => vertex.x)),
    Math.max(...all.map(vertex => vertex.z)) - Math.min(...all.map(vertex => vertex.z)),
  );
  return Number.isFinite(span) && span > 0 ? { rooms, connections, player, objectives } : null;
}

function signalPrecision(snapshot: WorldSnapshotV3): { kind: "exact" | "unknown" | "area"; position?: Point } {
  if (invalidSignalWraithProjection(snapshot)) return { kind: "unknown" };
  const source = signalWraithInterference(snapshot);
  if (source) return { kind: "area", position: { x: source.transform.positionM.xM, z: source.transform.positionM.zM } };
  if (snapshot.sceneId !== "mh_signal_yard") return { kind: "exact" };
  const matches = Array.isArray(snapshot.hazards)
    ? snapshot.hazards.filter(hazard => hazard?.entityId === SIGNAL_HAZARD) : [];
  if (matches.length !== 1) return { kind: "unknown" };
  const hazard = matches[0]!;
  const transform = hazard.transform;
  const position = point(transform?.positionM);
  if (hazard.kind !== "signal_interference_zone" || typeof hazard.active !== "boolean" ||
      !position || typeof transform?.yawRad !== "number" || !Number.isFinite(transform.yawRad)) {
    return { kind: "unknown" };
  }
  return hazard.active ? { kind: "area", position } : { kind: "exact" };
}

/** A skin over Rust-authorized map knowledge; no scene or route data fills gaps. */
export function deriveMistHarborSonarMap(snapshot: WorldSnapshotV3): MistHarborSonarView | null {
  if (snapshot.worldId !== "mist_harbor") return null;
  const localMapRows = Array.isArray(snapshot.capabilities?.items)
    ? snapshot.capabilities.items.filter(item => item?.capabilityId === LOCAL_MAP) : [];
  const selectedLocalMap = localMapRows.length === 1 && localMapRows[0]?.granted === true && localMapRows[0]?.selected === true;
  const effectiveAuthorization = snapshot.capabilities?.mapTopologyAuthorized;
  const authorized = effectiveAuthorization === undefined
    ? selectedLocalMap
    : effectiveAuthorization === true;
  if (!authorized) {
    return empty("unauthorized");
  }
  const map = snapshot.capabilities?.exploredMap;
  if (!map || map.worldId !== snapshot.worldId) return empty("unexplored");
  const geometry = readGeometry(map);
  if (!geometry) return empty("unexplored");
  const precision = signalPrecision(snapshot);
  const markers: Marker[] = geometry.objectives.length === 0 && !selectedLocalMap ? []
    : precision.kind === "area" && precision.position
    ? [{ kind: "signal_region", position: precision.position }]
    : precision.kind === "exact"
      ? geometry.objectives.map(position => ({ kind: "objective", position })) : [];
  return { status: "ready", rooms: geometry.rooms, connections: geometry.connections,
    player: geometry.player, markers };
}

function svgElement<K extends keyof SVGElementTagNameMap>(tag: K, className: string): SVGElementTagNameMap[K] {
  const element = document.createElementNS(SVG_NS, tag);
  element.setAttribute("class", className);
  return element;
}

function renderRadar(view: MistHarborSonarView): SVGSVGElement {
  const vertices = view.rooms.flatMap(room => room.outline);
  const minX = Math.min(...vertices.map(vertex => vertex.x));
  const maxX = Math.max(...vertices.map(vertex => vertex.x));
  const minZ = Math.min(...vertices.map(vertex => vertex.z));
  const maxZ = Math.max(...vertices.map(vertex => vertex.z));
  const span = Math.max(maxX - minX, maxZ - minZ);
  const centerX = minX + (maxX - minX) / 2;
  const centerZ = minZ + (maxZ - minZ) / 2;
  const xy = (value: Point) => ({ x: 50 + (value.x - centerX) / span * 76,
    y: 50 - (value.z - centerZ) / span * 76 });
  const svg = svgElement("svg", "mh-sonar-radar");
  svg.setAttribute("viewBox", "0 0 100 100");
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", "雾港声呐海图：仅显示 Rust 授权的地图与导航信息");
  for (const radius of [20, 39, 48]) {
    const ring = svgElement("circle", "mh-sonar-ring");
    ring.setAttribute("cx", "50"); ring.setAttribute("cy", "50"); ring.setAttribute("r", String(radius));
    svg.append(ring);
  }
  for (const [x1, y1, x2, y2] of [[50, 2, 50, 10], [90, 50, 98, 50],
    [50, 90, 50, 98], [2, 50, 10, 50]]) {
    const tick = svgElement("line", "mh-sonar-bearing");
    tick.setAttribute("x1", String(x1)); tick.setAttribute("y1", String(y1));
    tick.setAttribute("x2", String(x2)); tick.setAttribute("y2", String(y2));
    svg.append(tick);
  }
  for (const room of view.rooms) {
    const shape = svgElement("polygon", "mh-sonar-room");
    shape.setAttribute("points", room.outline.map(vertex => {
      const projected = xy(vertex);
      return `${projected.x},${projected.y}`;
    }).join(" "));
    svg.append(shape);
  }
  for (const connection of view.connections) {
    const line = svgElement("line", "mh-sonar-link");
    const from = xy(connection.from);
    const to = xy(connection.to);
    line.setAttribute("x1", String(from.x)); line.setAttribute("y1", String(from.y));
    line.setAttribute("x2", String(to.x)); line.setAttribute("y2", String(to.y));
    svg.append(line);
  }
  for (const marker of view.markers) {
    const location = xy(marker.position);
    const dot = svgElement("circle", marker.kind === "signal_region" ? "mh-sonar-signal" : "mh-sonar-objective");
    dot.setAttribute("cx", String(location.x)); dot.setAttribute("cy", String(location.y));
    dot.setAttribute("r", marker.kind === "signal_region" ? "7" : "2.2");
    svg.append(dot);
  }
  if (view.player) {
    const location = xy(view.player);
    const player = svgElement("circle", "mh-sonar-player");
    player.setAttribute("cx", String(location.x)); player.setAttribute("cy", String(location.y));
    player.setAttribute("r", "2.8");
    svg.append(player);
  }
  const north = svgElement("text", "mh-sonar-north");
  north.setAttribute("x", "50"); north.setAttribute("y", "7");
  north.setAttribute("text-anchor", "middle");
  north.textContent = "N";
  svg.append(north);
  return svg;
}

export function renderMistHarborSonarMap(host: HTMLElement, snapshot: WorldSnapshotV3): void {
  host.replaceChildren();
  const view = deriveMistHarborSonarMap(snapshot);
  if (!view) return;
  host.className = "mh-sonar-panel";
  const title = document.createElement("h3");
  title.textContent = "声呐海图";
  const status = document.createElement("p");
  status.className = "mh-sonar-status";
  status.textContent = view.status === "unauthorized" ? "地图信息未授权"
    : view.status === "unexplored" ? "尚无已探索投影"
      : "已授权 · 仅显示获准地图信息";
  host.append(title, status);
  if (view.status !== "ready") return;
  host.append(renderRadar(view));
  const legend = document.createElement("p");
  legend.className = "mh-sonar-legend";
  legend.textContent = view.markers.some(marker => marker.kind === "signal_region")
    ? "● 玩家 · ◯ 信号区域（方向提示，非目标坐标）"
    : "● 玩家 · N 北向 · 已知通路";
  host.append(legend);
}
