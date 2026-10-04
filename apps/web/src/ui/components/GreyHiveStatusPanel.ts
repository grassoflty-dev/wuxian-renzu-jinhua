import type { ExploredMapProjection, Vec3, WorldSnapshotV3 } from "../../protocol/types.js";

const SVG_NS = "http://www.w3.org/2000/svg";
const POWER_OBJECTIVE = "gh_restore_main_power";
const CONTAINMENT_OBJECTIVE = "gh_lockdown_objective";
const MAP_CAPABILITY = "information.local_map_i";

interface Point { x: number; z: number }
interface MapRoom { id: string; outline: Point[]; center: Point }
interface MapView {
  rooms: MapRoom[];
  connections: Array<{ from: Point; to: Point }>;
  objectives: Point[];
  player: Point | null;
}

export interface GreyHiveStatusView {
  power: string;
  containment: string;
  mapStatus: string;
  map: MapView | null;
}

function point(value: Vec3 | undefined): Point | null {
  return value && Number.isFinite(value.xM) && Number.isFinite(value.zM)
    ? { x: value.xM, z: value.zM } : null;
}

function objectiveStatus(snapshot: WorldSnapshotV3, id: string): string {
  const objective = Array.isArray(snapshot.objectives)
    ? snapshot.objectives.find(item => item?.objectiveId === id) : undefined;
  if (objective?.state === "complete") return "已完成 · 当前场景权威目标";
  if (objective?.state === "active") return "待完成 · 当前场景权威目标";
  return "当前场景未投影此状态";
}

function completedEvents(snapshot: WorldSnapshotV3): ReadonlySet<string> | null {
  if (snapshot.progression?.currentWorldId !== snapshot.worldId ||
      !Array.isArray(snapshot.progression.worlds)) return null;
  const world = snapshot.progression.worlds.find(row =>
    !!row && typeof row === "object" && (row as Record<string, unknown>).worldId === "grey_hive") as Record<string, unknown> | undefined;
  if (!world || !Array.isArray(world.completedEvents) ||
      !world.completedEvents.every(event => typeof event === "string")) return null;
  return new Set(world.completedEvents);
}

function readMap(map: ExploredMapProjection): MapView | null {
  if (!Array.isArray(map.rooms) || !Array.isArray(map.connections) || !Array.isArray(map.objectives)) return null;
  const rooms = map.rooms.flatMap(room => {
    if (!room || typeof room.roomId !== "string" || !Array.isArray(room.outlineM)) return [];
    const outline = room.outlineM.map(vertex => point(vertex)).filter((vertex): vertex is Point => vertex !== null);
    if (outline.length < 3 || outline.length !== room.outlineM.length) return [];
    const center = {
      x: outline.reduce((sum, vertex) => sum + vertex.x, 0) / outline.length,
      z: outline.reduce((sum, vertex) => sum + vertex.z, 0) / outline.length,
    };
    return [{ id: room.roomId, outline, center }];
  });
  if (rooms.length === 0) return null;
  const byId = new Map(rooms.map(room => [room.id, room.center]));
  const connections = map.connections.flatMap(link => {
    const from = byId.get(link?.fromRoomId);
    const to = byId.get(link?.toRoomId);
    return from && to ? [{ from, to }] : [];
  });
  const objectives = map.objectives.map(item => point(item?.positionM)).filter((value): value is Point => value !== null);
  return { rooms, connections, objectives, player: point(map.playerPositionM) };
}

/** Prefer Rust route events; older snapshots fall back to current-scene objectives. */
export function deriveGreyHiveStatus(snapshot: WorldSnapshotV3): GreyHiveStatusView | null {
  if (snapshot.worldId !== "grey_hive") return null;
  const events = completedEvents(snapshot);
  const authorized = Array.isArray(snapshot.capabilities?.items) && snapshot.capabilities.items.some(item =>
    item?.capabilityId === MAP_CAPABILITY && item.granted === true && item.selected === true);
  const projection = snapshot.capabilities?.exploredMap;
  const map = authorized && projection?.worldId === snapshot.worldId ? readMap(projection) : null;
  return {
    power: events ? events.has("hive_power") ? "已恢复 · Rust 权威进度" : "待恢复 · Rust 权威进度"
      : objectiveStatus(snapshot, POWER_OBJECTIVE),
    containment: events ? events.has("hive_lockdown") ? "封锁已解除 · Rust 权威进度" : "封锁待解除 · Rust 权威进度"
      : objectiveStatus(snapshot, CONTAINMENT_OBJECTIVE),
    mapStatus: !authorized ? "局部地图未授予或未选中"
      : map ? "已授权 · 当前世界探索投影" : "已授权 · 当前无有效探索投影",
    map,
  };
}

function mapSvg(view: MapView): SVGSVGElement {
  const allPoints = view.rooms.flatMap(room => room.outline);
  const minX = Math.min(...allPoints.map(vertex => vertex.x));
  const maxX = Math.max(...allPoints.map(vertex => vertex.x));
  const minZ = Math.min(...allPoints.map(vertex => vertex.z));
  const maxZ = Math.max(...allPoints.map(vertex => vertex.z));
  const span = Math.max(maxX - minX, maxZ - minZ, 1);
  const midX = (minX + maxX) / 2;
  const midZ = (minZ + maxZ) / 2;
  const xy = (vertex: Point) => ({ x: 50 + (vertex.x - midX) / span * 90, y: 50 - (vertex.z - midZ) / span * 90 });
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("viewBox", "0 0 100 100");
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", "已授权的灰巢设施探索地图");
  for (const room of view.rooms) {
    const shape = document.createElementNS(SVG_NS, "polygon");
    shape.setAttribute("class", "gh-map-room");
    shape.setAttribute("points", room.outline.map(vertex => {
      const projected = xy(vertex);
      return `${projected.x},${projected.y}`;
    }).join(" "));
    svg.append(shape);
  }
  for (const link of view.connections) {
    const line = document.createElementNS(SVG_NS, "line");
    const from = xy(link.from);
    const to = xy(link.to);
    line.setAttribute("class", "gh-map-link");
    line.setAttribute("x1", String(from.x));
    line.setAttribute("y1", String(from.y));
    line.setAttribute("x2", String(to.x));
    line.setAttribute("y2", String(to.y));
    svg.append(line);
  }
  for (const objective of view.objectives) {
    const marker = document.createElementNS(SVG_NS, "circle");
    const projected = xy(objective);
    marker.setAttribute("class", "gh-map-objective");
    marker.setAttribute("cx", String(projected.x));
    marker.setAttribute("cy", String(projected.y));
    marker.setAttribute("r", "1.5");
    svg.append(marker);
  }
  if (view.player) {
    const marker = document.createElementNS(SVG_NS, "circle");
    const projected = xy(view.player);
    marker.setAttribute("class", "gh-map-player");
    marker.setAttribute("cx", String(projected.x));
    marker.setAttribute("cy", String(projected.y));
    marker.setAttribute("r", "2");
    svg.append(marker);
  }
  return svg;
}

export function renderGreyHiveStatusPanel(host: HTMLElement, snapshot: WorldSnapshotV3): void {
  host.replaceChildren();
  const view = deriveGreyHiveStatus(snapshot);
  if (!view) return;
  host.className = "gh-status-panel";
  for (const [heading, value] of [["电力状态", view.power], ["隔离封锁", view.containment], ["设施地图", view.mapStatus]] as const) {
    const card = document.createElement("section");
    card.className = "gh-status-card";
    const title = document.createElement("h3");
    title.textContent = heading;
    const detail = document.createElement("p");
    detail.textContent = value;
    card.append(title, detail);
    if (heading === "设施地图" && view.map) {
      card.append(mapSvg(view.map));
      const rooms = document.createElement("p");
      rooms.className = "gh-map-caption";
      rooms.textContent = `已探索：${view.map.rooms.map(room => room.id).join("、")}`;
      card.append(rooms);
    }
    host.append(card);
  }
}
