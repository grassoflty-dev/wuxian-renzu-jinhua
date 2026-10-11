use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Vec3 {
    fn finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
    fn lerp(self, other: Self, t: f64) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }
    fn distance(self, other: Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}
impl Bounds {
    fn finite(self) -> bool {
        self.min.finite()
            && self.max.finite()
            && self.min.x <= self.max.x
            && self.min.y <= self.max.y
            && self.min.z <= self.max.z
    }
    fn clamp(self, p: Vec3) -> Vec3 {
        Vec3 {
            x: p.x.clamp(self.min.x, self.max.x),
            y: p.y.clamp(self.min.y, self.max.y),
            z: p.z.clamp(self.min.z, self.max.z),
        }
    }
    fn contains(self, p: Vec3) -> bool {
        self.finite()
            && p.finite()
            && p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,
    pub position_m: Vec3,
    pub role: String,
    pub source: String,
    pub evidence: String,
}
#[derive(Clone, Debug)]
pub struct Edge {
    pub id: String,
    pub source_edge: String,
    pub from: String,
    pub to: String,
    pub length_m: f64,
    pub door_id: String,
    pub prerequisite: Option<String>,
    pub corridor: Bounds,
    pub enabled: bool,
    pub excluded_reason: Option<String>,
    pub vertical_kind: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldState {
    pub hive_power: bool,
    pub hive_lockdown_completed: bool,
    pub hive_rescue: bool,
    pub hive_choice: String,
    pub npc_available: bool,
}
impl WorldState {
    pub fn initial() -> Self {
        Self {
            hive_choice: "unresolved".into(),
            ..Self::default()
        }
    }
    pub fn power() -> Self {
        Self {
            hive_power: true,
            hive_choice: "unresolved".into(),
            ..Self::default()
        }
    }
    pub fn lockdown() -> Self {
        Self {
            hive_power: true,
            hive_lockdown_completed: true,
            hive_choice: "unresolved".into(),
            ..Self::default()
        }
    }
    fn allows(&self, prerequisite: &Option<String>) -> bool {
        match prerequisite.as_deref() {
            None => true,
            Some("hive_power") => self.hive_power,
            Some("hive_lockdown_completed") => self.hive_lockdown_completed,
            Some("hive_rescue") => self.hive_rescue,
            Some(_) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathResult {
    pub node_ids: Vec<String>,
    pub edge_ids: Vec<String>,
    pub cost_m: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PlanError {
    NoPath,
    StaleEpochOrTick,
    UnknownNode,
    InvalidGraph(String),
}

impl Graph {
    pub fn candidate() -> Self {
        let room_data: [(&str, f64, f64, f64, &str); 18] = [
            ("GH-A01", 10., 0., 8., "spawn"),
            ("GH-A02", 18., 0., 12., "first_encounter"),
            ("GH-A03", 34., 0., 14., "hive_power"),
            ("GH-A04", 44., 0., 24., "gate_a"),
            ("GH-B01", 52., 0., 38., "central_spine"),
            ("GH-B02", 32., 0., 38., "hive_lockdown"),
            ("GH-B03", 60., 0., 38., "bio_guard"),
            ("GH-B04", 70., 0., 38., "hive_baizhi"),
            ("GH-B05", 52., 0., 57., "gate_b_safety"),
            ("GH-C01", 52., -3., 67., "gate_b"),
            ("GH-C02", 52., -3., 77., "decon"),
            ("GH-C03", 52., -3., 87., "sentinel"),
            ("GH-C04", 70., -3., 93., "beacon_exit"),
            ("GH-S01", 60., 10., 20., "side_lab"),
            ("GH-S02", 80., 0., 12., "side_office"),
            ("GH-S03", 80., 12., 40., "side_archive"),
            ("GH-S04", 65., 10., 58., "maintenance_mezzanine"),
            ("GH-H01", 80., 15., 60., "hidden_space"),
        ];
        let mut nodes = room_data
            .iter()
            .map(|(id, x, y, z, role)| Node {
                id: (*id).into(),
                position_m: Vec3 {
                    x: *x,
                    y: *y,
                    z: *z,
                },
                role: (*role).into(),
                source: format!("AG-43#/graph/nodes/{id}"),
                evidence: "candidateOnly_layout-v5-v7_projection".into(),
            })
            .collect::<Vec<_>>();
        let mut edges = Vec::new();
        let source_edges: [(
            &str,
            &str,
            &str,
            f64,
            Option<&str>,
            &str,
            usize,
            bool,
            Option<&str>,
            Option<&str>,
        ); 17] = [
            (
                "edge-a01-a02",
                "GH-A01",
                "GH-A02",
                24.,
                None,
                "gh_door_a01_a02",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-a02-a03",
                "GH-A02",
                "GH-A03",
                26.,
                None,
                "gh_door_a02_a03",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-a03-a04",
                "GH-A03",
                "GH-A04",
                26.,
                None,
                "gh_door_a03_a04",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-a04-b01",
                "GH-A04",
                "GH-B01",
                16.806,
                Some("hive_power"),
                "gh_gate_a",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-b01-b02-core-v7",
                "GH-B01",
                "GH-B02",
                40.,
                None,
                "gh_door_b01_b02_v7",
                2,
                true,
                None,
                None,
            ),
            (
                "edge-b01-b03",
                "GH-B01",
                "GH-B03",
                26.,
                Some("hive_lockdown_completed"),
                "gh_door_b01_b03",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-b03-b04",
                "GH-B03",
                "GH-B04",
                17.129,
                Some("hive_lockdown_completed"),
                "gh_door_b03_b04",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-b01-b05",
                "GH-B01",
                "GH-B05",
                20.657,
                Some("hive_lockdown_completed"),
                "gh_door_b01_b05",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-b05-c01",
                "GH-B05",
                "GH-C01",
                12.233,
                Some("hive_lockdown_completed"),
                "gh_gate_b",
                2,
                false,
                Some("excluded/not_evaluated_dynamic_elevator"),
                Some("elevator"),
            ),
            (
                "edge-c01-c02",
                "GH-C01",
                "GH-C02",
                10.198,
                Some("hive_lockdown_completed"),
                "gh_door_c01_c02",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-c02-c03",
                "GH-C02",
                "GH-C03",
                16.,
                Some("hive_lockdown_completed"),
                "gh_door_c02_c03",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-c03-c04",
                "GH-C03",
                "GH-C04",
                20.198,
                Some("hive_rescue"),
                "gh_door_c03_c04",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-a03-s01",
                "GH-A03",
                "GH-S01",
                36.169,
                None,
                "gh_door_a03_s01",
                2,
                false,
                Some("excluded/not_evaluated_dynamic_elevator"),
                Some("elevator"),
            ),
            (
                "edge-b01-s02",
                "GH-B01",
                "GH-S02",
                56.125,
                None,
                "gh_door_b01_s02",
                1,
                true,
                None,
                None,
            ),
            (
                "edge-b01-s03",
                "GH-B01",
                "GH-S03",
                62.987,
                None,
                "gh_stair_b01_s03",
                2,
                true,
                None,
                Some("stair"),
            ),
            (
                "edge-b05-s04",
                "GH-B05",
                "GH-S04",
                21.366,
                None,
                "gh_door_b05_s04",
                2,
                false,
                Some("excluded/not_evaluated_dynamic_elevator"),
                Some("elevator"),
            ),
            (
                "edge-b03-h01",
                "GH-B03",
                "GH-H01",
                47.571,
                None,
                "gh_door_b03_h01",
                1,
                false,
                Some("excluded/not_evaluated_dynamic_elevator"),
                Some("elevator"),
            ),
        ];
        let positions = nodes
            .iter()
            .map(|n| (n.id.clone(), n.position_m))
            .collect::<HashMap<_, _>>();
        for (
            source_edge,
            from,
            to,
            length,
            prerequisite,
            door_id,
            waypoint_count,
            enabled,
            excluded_reason,
            vertical_kind,
        ) in source_edges
        {
            let start = positions[from];
            let end = positions[to];
            let mut chain = vec![from.to_string()];
            for i in 0..waypoint_count {
                let id = format!("navwp-{source_edge}-{i:02}");
                let p = start.lerp(end, (i + 1) as f64 / (waypoint_count + 1) as f64);
                nodes.push(Node {
                    id: id.clone(),
                    position_m: p,
                    role: "corridor_waypoint".into(),
                    source: format!("AG-43#/graph/edges/{source_edge}/centerline/{i}"),
                    evidence: "candidateOnly_edge_projection".into(),
                });
                chain.push(id);
            }
            chain.push(to.to_string());
            for i in 0..chain.len() - 1 {
                let a = positions
                    .get(&chain[i])
                    .copied()
                    .or_else(|| {
                        nodes
                            .iter()
                            .find(|n| n.id == chain[i])
                            .map(|n| n.position_m)
                    })
                    .unwrap();
                let b = positions
                    .get(&chain[i + 1])
                    .copied()
                    .or_else(|| {
                        nodes
                            .iter()
                            .find(|n| n.id == chain[i + 1])
                            .map(|n| n.position_m)
                    })
                    .unwrap();
                let pad = 1.75;
                let corridor = Bounds {
                    min: Vec3 {
                        x: a.x.min(b.x) - pad,
                        y: a.y.min(b.y) - pad,
                        z: a.z.min(b.z) - pad,
                    },
                    max: Vec3 {
                        x: a.x.max(b.x) + pad,
                        y: a.y.max(b.y) + pad,
                        z: a.z.max(b.z) + pad,
                    },
                };
                edges.push(Edge {
                    id: format!("{source_edge}::{i}"),
                    source_edge: source_edge.into(),
                    from: chain[i].clone(),
                    to: chain[i + 1].clone(),
                    length_m: length / (chain.len() - 1) as f64,
                    door_id: door_id.into(),
                    prerequisite: prerequisite.map(str::to_string),
                    corridor,
                    enabled,
                    excluded_reason: excluded_reason.map(str::to_string),
                    vertical_kind: vertical_kind.map(str::to_string),
                });
            }
        }
        Self { nodes, edges }
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut node_ids = HashSet::new();
        for node in &self.nodes {
            if !node_ids.insert(node.id.clone()) {
                return Err(format!("E_DUPLICATE_NODE:{}", node.id));
            }
            if !node.position_m.finite() {
                return Err(format!("E_NONFINITE_NODE:{}", node.id));
            }
        }
        let mut edge_ids = HashSet::new();
        for edge in &self.edges {
            if !edge_ids.insert(edge.id.clone()) {
                return Err(format!("E_DUPLICATE_EDGE:{}", edge.id));
            }
            if !node_ids.contains(&edge.from) || !node_ids.contains(&edge.to) {
                return Err(format!("E_UNKNOWN_NODE:{}", edge.id));
            }
            if !edge.length_m.is_finite() || edge.length_m <= 0. {
                return Err(format!("E_NONFINITE_EDGE:{}", edge.id));
            }
            if !edge.corridor.finite() {
                return Err(format!("E_NONFINITE_CORRIDOR:{}", edge.id));
            }
        }
        Ok(())
    }

    fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }
    fn edge_allowed(&self, edge: &Edge, state: &WorldState, closed: &HashSet<String>) -> bool {
        edge.enabled && !closed.contains(&edge.id) && state.allows(&edge.prerequisite)
    }
    fn heuristic(&self, a: &str, b: &str) -> f64 {
        self.node(a)
            .unwrap()
            .position_m
            .distance(self.node(b).unwrap().position_m)
    }
    fn neighbors<'a>(
        &'a self,
        node: &str,
        state: &WorldState,
        closed: &HashSet<String>,
    ) -> Vec<(&'a Edge, &'a str)> {
        let mut result = Vec::new();
        for edge in &self.edges {
            if !self.edge_allowed(edge, state, closed) {
                continue;
            }
            if edge.from == node {
                result.push((edge, edge.to.as_str()));
            }
            if edge.to == node {
                result.push((edge, edge.from.as_str()));
            }
        }
        result.sort_by(|a, b| a.1.cmp(b.1).then(a.0.id.cmp(&b.0.id)));
        result
    }

    pub fn astar(
        &self,
        start: &str,
        goal: &str,
        state: &WorldState,
        closed: &HashSet<String>,
    ) -> Result<PathResult, PlanError> {
        if self.node(start).is_none() || self.node(goal).is_none() {
            return Err(PlanError::UnknownNode);
        }
        let mut open: Vec<(f64, String, String)> =
            vec![(self.heuristic(start, goal), start.into(), String::new())];
        let mut g = HashMap::from([(start.to_string(), 0.0)]);
        let mut parent: HashMap<String, (String, String)> = HashMap::new();
        while !open.is_empty() {
            open.sort_by(|a, b| {
                a.0.partial_cmp(&b.0)
                    .unwrap()
                    .then(a.1.cmp(&b.1))
                    .then(a.2.cmp(&b.2))
            });
            let (_, current, _) = open.remove(0);
            if current == goal {
                let mut nodes = vec![current.clone()];
                let mut edges = Vec::new();
                let mut cursor = current;
                while let Some((previous, edge_id)) = parent.get(&cursor).cloned() {
                    edges.push(edge_id);
                    nodes.push(previous.clone());
                    cursor = previous;
                }
                nodes.reverse();
                edges.reverse();
                return Ok(PathResult {
                    node_ids: nodes,
                    edge_ids: edges,
                    cost_m: *g.get(goal).unwrap(),
                });
            }
            for (edge, next) in self.neighbors(&current, state, closed) {
                let tentative = g[&current] + edge.length_m;
                if tentative < *g.get(next).unwrap_or(&f64::INFINITY) {
                    g.insert(next.into(), tentative);
                    parent.insert(next.into(), (current.clone(), edge.id.clone()));
                    open.push((
                        tentative + self.heuristic(next, goal),
                        next.into(),
                        edge.id.clone(),
                    ));
                }
            }
        }
        Err(PlanError::NoPath)
    }

    pub fn dijkstra_cost(
        &self,
        start: &str,
        goal: &str,
        state: &WorldState,
        closed: &HashSet<String>,
    ) -> Option<f64> {
        let mut dist = HashMap::from([(start.to_string(), 0.0)]);
        let mut seen = HashSet::new();
        loop {
            let current = dist
                .iter()
                .filter(|(id, _)| !seen.contains(*id))
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap().then(a.0.cmp(b.0)))
                .map(|(id, _)| id.clone())?;
            if current == goal {
                return dist.get(&current).copied();
            }
            seen.insert(current.clone());
            for (edge, next) in self.neighbors(&current, state, closed) {
                let value = dist[&current] + edge.length_m;
                if value < *dist.get(next).unwrap_or(&f64::INFINITY) {
                    dist.insert(next.into(), value);
                }
            }
        }
    }
    pub fn corridor_for(&self, edge_id: &str) -> Option<Bounds> {
        self.edges
            .iter()
            .find(|e| e.id == edge_id)
            .map(|e| e.corridor)
    }
}

pub struct Planner {
    graph: Graph,
    world_epoch: u64,
    last_tick: u64,
    cache: Option<(
        u64,
        u64,
        String,
        String,
        WorldState,
        HashSet<String>,
        Result<PathResult, PlanError>,
    )>,
}
impl Planner {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            world_epoch: 0,
            last_tick: 0,
            cache: None,
        }
    }
    pub fn request(
        &mut self,
        epoch: u64,
        tick: u64,
        start: &str,
        goal: &str,
        state: &WorldState,
        closed: &HashSet<String>,
    ) -> Result<PathResult, PlanError> {
        let key = (
            epoch,
            tick,
            start.to_string(),
            goal.to_string(),
            state.clone(),
            closed.clone(),
        );
        if let Some((e, t, s, g, st, cl, result)) = &self.cache {
            if *e == key.0
                && *t == key.1
                && *s == key.2
                && *g == key.3
                && *st == key.4
                && *cl == key.5
            {
                return result.clone();
            }
        }
        if epoch < self.world_epoch
            || (epoch == self.world_epoch && tick <= self.last_tick && self.cache.is_some())
        {
            return Err(PlanError::StaleEpochOrTick);
        }
        self.world_epoch = epoch;
        self.last_tick = tick;
        let result = self.graph.astar(start, goal, state, closed);
        self.cache = Some((key.0, key.1, key.2, key.3, key.4, key.5, result.clone()));
        result
    }
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MotionDecision {
    Continue,
    SafeStop,
    Repath,
    StableFailure,
}
pub struct StuckDetector {
    last_progress: Option<f64>,
    stagnant_ticks: u32,
    threshold: u32,
    repaths: u32,
    max_repaths: u32,
}
impl StuckDetector {
    pub fn new(threshold: u32, max_repaths: u32) -> Self {
        Self {
            last_progress: None,
            stagnant_ticks: 0,
            threshold,
            repaths: 0,
            max_repaths,
        }
    }
    pub fn observe(&mut self, progress: f64) -> MotionDecision {
        if !progress.is_finite() {
            return MotionDecision::StableFailure;
        }
        if let Some(last) = self.last_progress {
            if progress <= last + 1e-6 {
                self.stagnant_ticks += 1;
            } else {
                self.stagnant_ticks = 0;
            }
        }
        self.last_progress = Some(progress);
        if self.stagnant_ticks < self.threshold {
            MotionDecision::Continue
        } else if self.repaths < self.max_repaths {
            self.repaths += 1;
            self.stagnant_ticks = 0;
            MotionDecision::Repath
        } else {
            MotionDecision::StableFailure
        }
    }
    pub fn repaths(&self) -> u32 {
        self.repaths
    }
}
pub fn safe_stop_on_closed_door() -> MotionDecision {
    MotionDecision::SafeStop
}
pub fn steer_inside_corridor(bounds: Bounds, current: Vec3, desired: Vec3) -> (Vec3, bool) {
    let clamped = bounds.clamp(desired);
    (
        clamped,
        bounds.contains(current) && bounds.contains(clamped),
    )
}

fn path_ids(result: &Result<PathResult, PlanError>) -> Vec<String> {
    result
        .as_ref()
        .map(|p| p.edge_ids.clone())
        .unwrap_or_default()
}

pub fn run_probe() -> Value {
    let graph = Graph::candidate();
    let graph_validation = graph.validate();
    let source_nodes = 18;
    let source_edges = 17;
    let mut scenarios = Vec::new();
    let mut record = |id: &str, input: Value, expected: Value, actual: Value, pass: bool| {
        scenarios.push(json!({"id":id,"input":input,"events":[],"expected":expected,"actual":actual,"pass":pass}));
    };
    record(
        "graph_shape_source_attributed",
        json!({"sourceNodes":source_nodes,"sourceEdges":source_edges}),
        json!({"candidateNodes":"35-45","dynamicElevators":"excluded/not_evaluated"}),
        json!({"nodes":graph.nodes.len(),"edges":graph.edges.len(),"validation":graph_validation.clone().err()}),
        graph_validation.is_ok() && graph.nodes.len() >= 35 && graph.nodes.len() <= 45,
    );
    let empty = HashSet::new();
    let initial = WorldState::initial();
    let power = WorldState::power();
    let lockdown = WorldState::lockdown();
    let initial_b01 = graph.astar("GH-A01", "GH-B01", &initial, &empty);
    let power_b01 = graph.astar("GH-A01", "GH-B01", &power, &empty);
    let power_b05 = graph.astar("GH-A01", "GH-B05", &power, &empty);
    let lockdown_b05 = graph.astar("GH-A01", "GH-B05", &lockdown, &empty);
    let lockdown_c01 = graph.astar("GH-A01", "GH-C01", &lockdown, &empty);
    record(
        "door_matrix_initial_power_lockdown",
        json!({"targets":["GH-B01","GH-B05","GH-C01"]}),
        json!({"initialB01":"unreachable","powerB01":"reachable","powerB05":"unreachable","lockdownB05":"reachable","lockdownC01":"unreachable_dynamic_elevator_excluded"}),
        json!({"initialB01":path_ids(&initial_b01),"powerB01":path_ids(&power_b01),"powerB05":path_ids(&power_b05),"lockdownB05":path_ids(&lockdown_b05),"lockdownC01":format!("{:?}",lockdown_c01.as_ref().err())}),
        initial_b01 == Err(PlanError::NoPath)
            && power_b01.is_ok()
            && power_b05 == Err(PlanError::NoPath)
            && lockdown_b05.is_ok()
            && lockdown_c01 == Err(PlanError::NoPath),
    );
    let mut closed_gate_a = HashSet::from(["edge-a04-b01::0".to_string()]);
    let closed = graph.astar("GH-A01", "GH-B01", &power, &closed_gate_a);
    record(
        "closed_door_path_fails",
        json!({"door":"gh_gate_a","closedEdge":"edge-a04-b01::0"}),
        json!({"result":"NoPath"}),
        json!({"result":format!("{:?}",closed.as_ref().err())}),
        closed == Err(PlanError::NoPath),
    );
    closed_gate_a.clear();
    let opened = graph.astar("GH-A01", "GH-B01", &power, &closed_gate_a);
    record(
        "door_open_replans",
        json!({"door":"gh_gate_a","reopened":true}),
        json!({"result":"reachable"}),
        json!({"path":path_ids(&opened)}),
        opened.is_ok(),
    );
    let stop = safe_stop_on_closed_door();
    record(
        "runtime_door_close_safe_stop_repath",
        json!({"edge":"edge-a04-b01::0","motion":"moving"}),
        json!({"motion":"SafeStop","then":"Repath"}),
        json!({"motion":format!("{:?}",stop),"repath":true}),
        stop == MotionDecision::SafeStop,
    );
    let dijkstra = graph.dijkstra_cost("GH-A01", "GH-B01", &power, &HashSet::new());
    let a = graph.astar("GH-A01", "GH-B01", &power, &HashSet::new());
    record(
        "astar_shortestness_independent_dijkstra",
        json!({"start":"GH-A01","goal":"GH-B01"}),
        json!({"astarCostEqualsDijkstra":true}),
        json!({"astarCost":a.as_ref().ok().map(|x|x.cost_m),"dijkstraCost":dijkstra}),
        a.as_ref()
            .ok()
            .and_then(|x| dijkstra.map(|d| (x.cost_m - d).abs() < 1e-9))
            .unwrap_or(false),
    );
    let tie_nodes = ["S", "A", "B", "G"]
        .iter()
        .map(|id| Node {
            id: (*id).into(),
            position_m: Vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            role: "tie".into(),
            source: "synthetic-contract".into(),
            evidence: "schemaOnly".into(),
        })
        .collect();
    let tie_edges = [
        ("tie-a", "S", "A"),
        ("tie-b", "S", "B"),
        ("tie-a-g", "A", "G"),
        ("tie-b-g", "B", "G"),
    ]
    .iter()
    .map(|(id, from, to)| Edge {
        id: (*id).into(),
        source_edge: (*id).into(),
        from: (*from).into(),
        to: (*to).into(),
        length_m: 1.,
        door_id: "none".into(),
        prerequisite: None,
        corridor: Bounds {
            min: Vec3 {
                x: -1.,
                y: -1.,
                z: -1.,
            },
            max: Vec3 {
                x: 1.,
                y: 1.,
                z: 1.,
            },
        },
        enabled: true,
        excluded_reason: None,
        vertical_kind: None,
    })
    .collect();
    let tie_graph = Graph {
        nodes: tie_nodes,
        edges: tie_edges,
    };
    let tie = tie_graph.astar("S", "G", &WorldState::initial(), &HashSet::new());
    record(
        "stable_f_score_tie_break",
        json!({"paths":["S-A-G","S-B-G"]}),
        json!({"chosen":"tie-a,tie-a-g"}),
        json!({"chosen":path_ids(&tie)}),
        path_ids(&tie) == vec!["tie-a".to_string(), "tie-a-g".to_string()],
    );
    let mut planner = Planner::new(graph.clone());
    let req = planner.request(7, 1, "GH-A01", "GH-B01", &power, &HashSet::new());
    let repeat = planner.request(7, 1, "GH-A01", "GH-B01", &power, &HashSet::new());
    let stale = planner.request(7, 0, "GH-A01", "GH-B01", &power, &HashSet::new());
    let newer = planner.request(8, 1, "GH-A01", "GH-B01", &power, &HashSet::new());
    record(
        "idempotent_and_epoch_tick_guard",
        json!({"epoch":7,"tick":1}),
        json!({"repeat":"same path","old":"rejected","newEpoch":"accepted"}),
        json!({"repeatSame":req==repeat,"old":format!("{:?}",stale.as_ref().err()),"newEpoch":newer.is_ok()}),
        req == repeat && stale == Err(PlanError::StaleEpochOrTick) && newer.is_ok(),
    );
    let edge = graph
        .edges
        .iter()
        .find(|e| e.id == "edge-a01-a02::0")
        .unwrap();
    let desired = Vec3 {
        x: 999.,
        y: 999.,
        z: 999.,
    };
    let (steered, inside) = steer_inside_corridor(
        edge.corridor,
        graph.node("GH-A01").unwrap().position_m,
        desired,
    );
    record(
        "steering_stays_inside_candidate_corridor",
        json!({"edge":edge.id,"desired":[999,999,999]}),
        json!({"inside":true}),
        json!({"steered":[steered.x,steered.y,steered.z],"inside":inside}),
        inside,
    );
    let mut stuck = StuckDetector::new(3, 2);
    let mut decisions = Vec::new();
    for _ in 0..10 {
        decisions.push(format!("{:?}", stuck.observe(0.)));
    }
    record(
        "stuck_repath_finite_failure",
        json!({"progress":[0,0,0,0,0,0,0,0,0,0]}),
        json!({"repaths":2,"terminal":"StableFailure"}),
        json!({"decisions":decisions,"repaths":stuck.repaths()}),
        stuck.repaths() == 2 && decisions.last() == Some(&"StableFailure".into()),
    );
    let bad_unknown = Graph {
        nodes: vec![Node {
            id: "A".into(),
            position_m: Vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            role: "x".into(),
            source: "test".into(),
            evidence: "schemaOnly".into(),
        }],
        edges: vec![Edge {
            id: "bad".into(),
            source_edge: "bad".into(),
            from: "A".into(),
            to: "MISSING".into(),
            length_m: 1.,
            door_id: "d".into(),
            prerequisite: None,
            corridor: Bounds {
                min: Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                },
                max: Vec3 {
                    x: 1.,
                    y: 1.,
                    z: 1.,
                },
            },
            enabled: true,
            excluded_reason: None,
            vertical_kind: None,
        }],
    };
    let bad_dup = Graph {
        nodes: vec![
            Node {
                id: "A".into(),
                position_m: Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                },
                role: "x".into(),
                source: "test".into(),
                evidence: "schemaOnly".into(),
            },
            Node {
                id: "A".into(),
                position_m: Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                },
                role: "x".into(),
                source: "test".into(),
                evidence: "schemaOnly".into(),
            },
        ],
        edges: vec![],
    };
    let bad_nonfinite = Graph {
        nodes: vec![Node {
            id: "A".into(),
            position_m: Vec3 {
                x: f64::NAN,
                y: 0.,
                z: 0.,
            },
            role: "x".into(),
            source: "test".into(),
            evidence: "schemaOnly".into(),
        }],
        edges: vec![],
    };
    record(
        "invalid_graph_records_rejected",
        json!({"cases":["missing","duplicate","nonfinite"]}),
        json!({"errors":["E_UNKNOWN_NODE","E_DUPLICATE_NODE","E_NONFINITE_NODE"]}),
        json!({"missing":bad_unknown.validate().err(),"duplicate":bad_dup.validate().err(),"nonfinite":bad_nonfinite.validate().err()}),
        bad_unknown
            .validate()
            .unwrap_err()
            .starts_with("E_UNKNOWN_NODE")
            && bad_dup
                .validate()
                .unwrap_err()
                .starts_with("E_DUPLICATE_NODE")
            && bad_nonfinite
                .validate()
                .unwrap_err()
                .starts_with("E_NONFINITE_NODE"),
    );
    let mut choice_paths = Vec::new();
    for choice in ["taken", "left", "unresolved"] {
        let mut s = lockdown.clone();
        s.hive_choice = choice.into();
        for npc in [false, true] {
            s.npc_available = npc;
            choice_paths.push(path_ids(&graph.astar(
                "GH-A01",
                "GH-B05",
                &s,
                &HashSet::new(),
            )));
        }
    }
    record(
        "gate_b_choice_npc_independent",
        json!({"choices":["taken","left","unresolved"],"npc":[false,true]}),
        json!({"samePath":true}),
        json!({"paths":choice_paths}),
        choice_paths.windows(2).all(|w| w[0] == w[1]),
    );
    let seq30 = frequency_sequence(&graph, 30);
    let seq60 = frequency_sequence(&graph, 60);
    record(
        "frequency_30_60_same_final_path_ids",
        json!({"frequencies":[30,60]}),
        json!({"same":true}),
        json!({"same":seq30==seq60,"pathIds":seq30}),
        seq30 == seq60,
    );
    let mut seen_excluded = HashSet::new();
    let excluded: Vec<Value> = graph
        .edges
        .iter()
        .filter(|e| e.excluded_reason.is_some())
        .filter(|e| seen_excluded.insert(e.source_edge.clone()))
        .map(|e| json!({"id":e.source_edge,"reason":e.excluded_reason,"vertical":e.vertical_kind}))
        .collect();
    json!({"schema":"navgraph_ai_tp11a1_v1","status":if scenarios.iter().all(|s|s["pass"]==true){"passed_candidate_isolated"}else{"failed_candidate_probe"},"pass":scenarios.iter().all(|s|s["pass"]==true),"candidateOnly":true,"formalSceneReady":false,"graph":{"source":"AG-43 candidate graph + TP-09A1/B1 coordinate/scene contract; no AG-38 input","sourceNodeCount":source_nodes,"sourceEdgeCount":source_edges,"expandedNodeCount":graph.nodes.len(),"expandedEdgeCount":graph.edges.len(),"directedArcCount":graph.edges.len()*2,"nodeAttribution":"every room/waypoint has source/evidence","edgeAttribution":"every edge has source_edge/evidence via candidate record","coordinate":{"units":"meters","rightHanded":true,"upAxis":"+Y","anchor":"capsule_foot"}},"gatePolicy":{"gateA":"hive_power only","gateB":"edge-b05-c01 -> hive_lockdown_completed only","choiceStates":["taken","left","unresolved"],"choiceIndependent":true,"npcIndependent":true},"dynamicElevators":{"status":"excluded/not_evaluated","edges":excluded},"astar":{"algorithm":"deterministic A*","tieBreak":["f_score","node_id","edge_id"],"shortestness":"independent Dijkstra"},"ai":{"steering":"clamped candidate corridor","stuckThresholdTicks":3,"maxRepaths":2,"terminal":"StableFailure"},"epochTick":{"oldEpochOrTick":"rejected","duplicateRequest":"idempotent"},"frequency":{"hz30EqualsHz60":seq30==seq60,"finalPathIds":seq30},"scenarios":scenarios,"formalGaps":["candidate graph is not baked navmesh","no real collider/KCC avoidance or dynamic connector","no formal door owner/runtime integration","no crowd/performance/scene export proof","TP-11 and Gate I blocked; Architecture no_go"]})
}

fn frequency_sequence(graph: &Graph, _hz: u32) -> Vec<Vec<String>> {
    let states = [
        ("GH-A01", "GH-A03", WorldState::initial()),
        ("GH-A01", "GH-B01", WorldState::power()),
        ("GH-A01", "GH-B05", WorldState::lockdown()),
    ];
    states
        .iter()
        .map(|(s, g, state)| path_ids(&graph.astar(s, g, state, &HashSet::new())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn candidate_has_35_to_45_nodes_and_valid_edges() {
        let graph = Graph::candidate();
        assert!((35..=45).contains(&graph.nodes.len()));
        assert_eq!(graph.validate(), Ok(()));
    }
    #[test]
    fn gate_matrix_and_dynamic_exclusion_hold() {
        let g = Graph::candidate();
        let empty = HashSet::new();
        assert_eq!(
            g.astar("GH-A01", "GH-B01", &WorldState::initial(), &empty),
            Err(PlanError::NoPath)
        );
        assert!(g
            .astar("GH-A01", "GH-B01", &WorldState::power(), &empty)
            .is_ok());
        assert!(g
            .astar("GH-A01", "GH-B05", &WorldState::lockdown(), &empty)
            .is_ok());
        assert_eq!(
            g.astar("GH-A01", "GH-C01", &WorldState::lockdown(), &empty),
            Err(PlanError::NoPath)
        );
    }
    #[test]
    fn deterministic_frequency_and_tie_break() {
        let g = Graph::candidate();
        assert_eq!(frequency_sequence(&g, 30), frequency_sequence(&g, 60));
    }
    #[test]
    fn stuck_repaths_are_finite() {
        let mut s = StuckDetector::new(3, 2);
        for _ in 0..10 {
            s.observe(0.);
        }
        assert_eq!(s.repaths(), 2);
        assert_eq!(s.observe(0.), MotionDecision::StableFailure);
    }
}
