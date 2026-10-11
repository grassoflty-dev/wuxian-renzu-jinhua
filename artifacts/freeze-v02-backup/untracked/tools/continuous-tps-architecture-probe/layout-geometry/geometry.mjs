import fs from "node:fs";
import path from "node:path";

export const GEOMETRY_DIR = path.dirname(new URL(import.meta.url).pathname.replace(/^\/[A-Za-z]:/, (match) => match.slice(1)));
export const PROBE_ROOT = path.resolve(GEOMETRY_DIR, "..");
export const INPUTS = {
  layout: path.join(PROBE_ROOT, "layout", "layout-v4.json"),
  layoutResult: path.join(PROBE_ROOT, "results", "layout-v4.json"),
  coordinates: path.join(PROBE_ROOT, "results", "coordinates-ag05.json"),
  physics: path.join(PROBE_ROOT, "results", "physics-ag06.json"),
};

export const EPS = 1e-7;
export const CAPSULE = {
  radius_m: 0.35,
  height_m: 2.2,
  source: "AG-06 physics-ag06.json platform_contract (0.349999994... / 2.200000047... rounded conservatively)",
};

export function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

export function round(value, places = 3) {
  const factor = 10 ** places;
  return Math.round(value * factor) / factor;
}

export function vsub(a, b) { return [a[0] - b[0], a[1] - b[1]]; }
export function vadd(a, b) { return [a[0] + b[0], a[1] + b[1]]; }
export function vmul(a, scalar) { return [a[0] * scalar, a[1] * scalar]; }
export function dot(a, b) { return a[0] * b[0] + a[1] * b[1]; }
export function cross(a, b) { return a[0] * b[1] - a[1] * b[0]; }
export function length(a) { return Math.hypot(a[0], a[1]); }
export function distance(a, b) { return length(vsub(a, b)); }
export function normalize(a) {
  const len = length(a);
  return len < EPS ? [0, 0] : [a[0] / len, a[1] / len];
}
export function leftNormal(direction) { return [-direction[1], direction[0]]; }

export function point2(point) { return point.length === 2 ? point : [point[0], point[2]]; }
export function floorY(room) { return room.floor_y_m; }
export function roomBounds(room) {
  const [x, , z] = room.center;
  const [width, depth] = room.size;
  return { minX: x - width / 2, maxX: x + width / 2, minZ: z - depth / 2, maxZ: z + depth / 2 };
}
export function roomPolygon(room) {
  const b = roomBounds(room);
  return [[b.minX, b.minZ], [b.maxX, b.minZ], [b.maxX, b.maxZ], [b.minX, b.maxZ]];
}
export function roomContains(room, point, margin = 0) {
  const p = point2(point);
  const b = roomBounds(room);
  return p[0] >= b.minX + margin - EPS && p[0] <= b.maxX - margin + EPS && p[1] >= b.minZ + margin - EPS && p[1] <= b.maxZ - margin + EPS;
}
export function roomHeight(room) { return room.size[2]; }

export function polygonArea(polygon) {
  let area = 0;
  for (let i = 0; i < polygon.length; i += 1) {
    const a = polygon[i];
    const b = polygon[(i + 1) % polygon.length];
    area += a[0] * b[1] - b[0] * a[1];
  }
  return area / 2;
}

export function orientation(a, b, c) { return cross(vsub(b, a), vsub(c, a)); }
export function onSegment(a, b, p) {
  return Math.abs(orientation(a, b, p)) <= EPS && p[0] >= Math.min(a[0], b[0]) - EPS && p[0] <= Math.max(a[0], b[0]) + EPS && p[1] >= Math.min(a[1], b[1]) - EPS && p[1] <= Math.max(a[1], b[1]) + EPS;
}
export function segmentsIntersect(a, b, c, d, includeTouch = true) {
  const o1 = orientation(a, b, c);
  const o2 = orientation(a, b, d);
  const o3 = orientation(c, d, a);
  const o4 = orientation(c, d, b);
  if (includeTouch && Math.abs(o1) <= EPS && onSegment(a, b, c)) return true;
  if (includeTouch && Math.abs(o2) <= EPS && onSegment(a, b, d)) return true;
  if (includeTouch && Math.abs(o3) <= EPS && onSegment(c, d, a)) return true;
  if (includeTouch && Math.abs(o4) <= EPS && onSegment(c, d, b)) return true;
  return ((o1 > EPS && o2 < -EPS) || (o1 < -EPS && o2 > EPS)) && ((o3 > EPS && o4 < -EPS) || (o3 < -EPS && o4 > EPS));
}
export function pointInPolygon(point, polygon) {
  let inside = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i += 1) {
    const a = polygon[i];
    const b = polygon[j];
    if (onSegment(a, b, point)) return true;
    const crossesRay = ((a[1] > point[1]) !== (b[1] > point[1])) && point[0] < ((b[0] - a[0]) * (point[1] - a[1])) / (b[1] - a[1]) + a[0];
    if (crossesRay) inside = !inside;
  }
  return inside;
}
export function polygonSelfIntersections(polygon) {
  const hits = [];
  for (let i = 0; i < polygon.length; i += 1) {
    const a1 = polygon[i];
    const a2 = polygon[(i + 1) % polygon.length];
    for (let j = i + 1; j < polygon.length; j += 1) {
      if (j === i || (j + 1) % polygon.length === i || (i + 1) % polygon.length === j) continue;
      const b1 = polygon[j];
      const b2 = polygon[(j + 1) % polygon.length];
      if (segmentsIntersect(a1, a2, b1, b2, false)) hits.push({ edge_a: i, edge_b: j });
    }
  }
  return hits;
}

function miterPoint(point, prevDirection, nextDirection, halfWidth, side) {
  const prevNormal = leftNormal(prevDirection);
  const nextNormal = leftNormal(nextDirection);
  const bisector = normalize(vadd(prevNormal, nextNormal));
  const denominator = dot(bisector, prevNormal);
  if (Math.abs(denominator) < 0.2) return vadd(point, vmul(nextNormal, halfWidth * side));
  const miterLength = Math.min(halfWidth / Math.abs(denominator), halfWidth * 4);
  return vadd(point, vmul(bisector, miterLength * side));
}

export function corridorPolygon(centerline, width) {
  const halfWidth = width / 2;
  const left = [];
  const right = [];
  for (let i = 0; i < centerline.length; i += 1) {
    const point = centerline[i];
    const previous = centerline[Math.max(0, i - 1)];
    const next = centerline[Math.min(centerline.length - 1, i + 1)];
    const prevDirection = normalize(vsub(point, previous));
    const nextDirection = normalize(vsub(next, point));
    const direction = length(prevDirection) < EPS ? nextDirection : length(nextDirection) < EPS ? prevDirection : normalize(vadd(prevDirection, nextDirection));
    const normal = leftNormal(direction);
    if (i === 0 || i === centerline.length - 1) {
      left.push(vadd(point, vmul(normal, halfWidth)));
      right.push(vadd(point, vmul(normal, -halfWidth)));
    } else {
      left.push(miterPoint(point, prevDirection, nextDirection, halfWidth, 1));
      right.push(miterPoint(point, prevDirection, nextDirection, halfWidth, -1));
    }
  }
  return [...left, ...right.reverse()];
}

export function pathLength(pathPoints) {
  let total = 0;
  for (let i = 1; i < pathPoints.length; i += 1) total += distance(point2(pathPoints[i - 1]), point2(pathPoints[i]));
  return total;
}

export function polylineSelfIntersections(pathPoints) {
  const hits = [];
  for (let i = 0; i < pathPoints.length - 1; i += 1) {
    const a1 = point2(pathPoints[i]);
    const a2 = point2(pathPoints[i + 1]);
    for (let j = i + 2; j < pathPoints.length - 1; j += 1) {
      if (j === i + 1) continue;
      const b1 = point2(pathPoints[j]);
      const b2 = point2(pathPoints[j + 1]);
      if (segmentsIntersect(a1, a2, b1, b2, false)) hits.push({ segment_a: i, segment_b: j });
    }
  }
  return hits;
}

export function segmentDistance(point, a, b) {
  const ab = vsub(b, a);
  const denom = dot(ab, ab);
  const t = denom < EPS ? 0 : Math.max(0, Math.min(1, dot(vsub(point, a), ab) / denom));
  return distance(point, vadd(a, vmul(ab, t)));
}
export function polylineDistance(point, pathPoints) {
  let min = Infinity;
  for (let i = 1; i < pathPoints.length; i += 1) min = Math.min(min, segmentDistance(point, point2(pathPoints[i - 1]), point2(pathPoints[i])));
  return min;
}

export function pathSample(pathPoints, maxStep = 0.5) {
  const samples = [point2(pathPoints[0])];
  for (let i = 1; i < pathPoints.length; i += 1) {
    const a = point2(pathPoints[i - 1]);
    const b = point2(pathPoints[i]);
    const segmentLength = distance(a, b);
    const steps = Math.max(1, Math.ceil(segmentLength / maxStep));
    for (let step = 1; step <= steps; step += 1) samples.push(vadd(a, vmul(vsub(b, a), step / steps)));
  }
  return samples;
}

export function closestBoundaryPoint(point, room) {
  const b = roomBounds(room);
  const p = point2(point);
  const candidates = [[b.minX, p[1]], [b.maxX, p[1]], [p[0], b.minZ], [p[0], b.maxZ]].map(([x, z]) => [Math.max(b.minX, Math.min(b.maxX, x)), Math.max(b.minZ, Math.min(b.maxZ, z))]);
  return candidates.sort((a, c) => distance(p, a) - distance(p, c))[0];
}

export function boundaryCrossings(pathPoints, room) {
  const crossings = [];
  const sampleCount = 100;
  for (let segment = 0; segment < pathPoints.length - 1; segment += 1) {
    const a = point2(pathPoints[segment]);
    const b = point2(pathPoints[segment + 1]);
    let previousT = 0;
    let previousInside = roomContains(room, a, 0);
    for (let i = 1; i <= sampleCount; i += 1) {
      const t = i / sampleCount;
      const p = vadd(a, vmul(vsub(b, a), t));
      const inside = roomContains(room, p, 0);
      if (inside !== previousInside) {
        let lo = previousT;
        let hi = t;
        for (let iteration = 0; iteration < 30; iteration += 1) {
          const mid = (lo + hi) / 2;
          const midPoint = vadd(a, vmul(vsub(b, a), mid));
          if (roomContains(room, midPoint, 0) === previousInside) lo = mid;
          else hi = mid;
        }
        const crossT = (lo + hi) / 2;
        const crossPoint = vadd(a, vmul(vsub(b, a), crossT));
        crossings.push({ segment, t: round(crossT, 6), point: crossPoint, from_inside: previousInside, to_inside: inside });
      }
      previousT = t;
      previousInside = inside;
    }
  }
  return crossings;
}

export function firstExitCrossing(pathPoints, room) { return boundaryCrossings(pathPoints, room).find((crossing) => crossing.from_inside && !crossing.to_inside) ?? null; }
export function lastEntryCrossing(pathPoints, room) {
  const crossings = boundaryCrossings(pathPoints, room).filter((crossing) => !crossing.from_inside && crossing.to_inside);
  return crossings.at(-1) ?? null;
}

export function connectorLevel(data, edge) {
  const from = data.rooms.find((room) => room.id === edge.from);
  const to = data.rooms.find((room) => room.id === edge.to);
  return {
    from_floor_y_m: from.floor_y_m,
    to_floor_y_m: to.floor_y_m,
    delta_y_m: round(to.floor_y_m - from.floor_y_m),
    same_floor: Math.abs(from.floor_y_m - to.floor_y_m) <= EPS,
    from_level: from.level,
    to_level: to.level,
  };
}

export function roomAtPoint(data, point, floor) {
  return data.rooms.filter((room) => Math.abs(room.floor_y_m - floor) <= EPS && roomContains(room, point, 0));
}

export function corridorContains(point, connectorGeometry, margin = 0) {
  return connectorGeometry.centerline.some((_, index) => index < connectorGeometry.centerline.length - 1 && segmentDistance(point, connectorGeometry.centerline[index], connectorGeometry.centerline[index + 1]) <= connectorGeometry.width_m / 2 - margin + EPS);
}

export function navigablePoint(data, point, floor, connectorGeometries, margin = CAPSULE.radius_m) {
  const room = data.rooms.some((candidate) => Math.abs(candidate.floor_y_m - floor) <= EPS && roomContains(candidate, point, margin) && roomHeight(candidate) >= CAPSULE.height_m);
  if (room) return true;
  return connectorGeometries.some((connector) => connector.same_floor && Math.abs(connector.from_floor_y_m - floor) <= EPS && connector.height_m >= CAPSULE.height_m && corridorContains(point, connector, margin));
}

export function shortestPath(adjacency, start, goal) {
  const distances = new Map([...adjacency.keys()].map((key) => [key, Infinity]));
  const previous = new Map();
  const open = new Set(adjacency.keys());
  distances.set(start, 0);
  while (open.size) {
    let current = null;
    for (const candidate of open) if (current === null || distances.get(candidate) < distances.get(current)) current = candidate;
    if (current === null || distances.get(current) === Infinity) break;
    open.delete(current);
    if (current === goal) break;
    for (const edge of adjacency.get(current) ?? []) {
      if (!open.has(edge.to)) continue;
      const alternate = distances.get(current) + edge.weight;
      if (alternate < distances.get(edge.to)) {
        distances.set(edge.to, alternate);
        previous.set(edge.to, current);
      }
    }
  }
  if (distances.get(goal) === Infinity) return { reachable: false, distance_m: null, nodes: [] };
  const nodes = [];
  for (let cursor = goal; cursor !== undefined; cursor = previous.get(cursor)) nodes.unshift(cursor);
  return { reachable: true, distance_m: distances.get(goal), nodes };
}

export function edgeState(data, edge, stateName) {
  return data.gate_states[stateName]?.[edge.door.id] ?? edge.door.state;
}
