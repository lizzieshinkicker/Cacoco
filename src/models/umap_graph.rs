use crate::models::umapinfo::{MapEntry, UmapField, UmapInfoFile};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EdgeType {
    #[default]
    Normal,
    Secret,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Map { levelname: String },
    Episode { name: String, patch: String },
    InterText { is_secret: bool },
    Terminal { end_type: String },
}

#[derive(Debug, Clone)]
pub struct UmapNode {
    pub id: String,
    pub node_type: NodeType,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct UmapEdge {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Default)]
pub struct UmapGraph {
    pub nodes: Vec<UmapNode>,
    pub edges: Vec<UmapEdge>,
}

/// Helper struct to cleanly hold extracted properties for a single map
struct ParsedMapData {
    levelname: String,
    next: Option<String>,
    next_secret: Option<String>,
    has_text_normal: bool,
    has_text_secret: bool,
    terminal_type: Option<String>,
    episode_fields: Vec<(String, String)>,
}

impl ParsedMapData {
    fn extract(map: &MapEntry) -> Self {
        let mut data = Self {
            levelname: map.mapname.to_uppercase(),
            next: None,
            next_secret: None,
            has_text_normal: false,
            has_text_secret: false,
            terminal_type: None,
            episode_fields: Vec::new(),
        };

        for field in &map.fields {
            match field {
                UmapField::LevelName(name) => data.levelname = name.clone(),
                UmapField::Next(n) => data.next = Some(n.to_uppercase()),
                UmapField::NextSecret(n) => data.next_secret = Some(n.to_uppercase()),
                UmapField::InterText(lines) => {
                    if lines.len() != 1 || lines[0].to_lowercase() != "clear" {
                        data.has_text_normal = true;
                    }
                }
                UmapField::InterTextSecret(lines) => {
                    if lines.len() != 1 || lines[0].to_lowercase() != "clear" {
                        data.has_text_secret = true;
                    }
                }
                UmapField::EndGame(v) if *v => data.terminal_type = Some("End Game".to_string()),
                UmapField::EndBunny(v) if *v => data.terminal_type = Some("End Bunny".to_string()),
                UmapField::EndCast(v) if *v => data.terminal_type = Some("Cast Roll".to_string()),
                UmapField::EndPic(p) => data.terminal_type = Some(format!("End Pic: {}", p)),
                UmapField::Episode { patch, name, .. } => {
                    if patch.to_lowercase() != "clear" {
                        data.episode_fields.push((patch.clone(), name.clone()));
                    }
                }
                _ => {}
            }
        }
        data
    }
}

impl UmapGraph {
    /// Parses a standard UMAPINFO file and projects it into a visual node graph.
    pub fn build(file: &UmapInfoFile) -> Self {
        let spine_counts = Self::count_spine_blockers(file);
        let map_coords = Self::calculate_topological_grid(file, &spine_counts);
        Self::construct_graph(file, &map_coords)
    }

    /// Counts how many extra vertical slots each map requires for text/terminals
    fn count_spine_blockers(file: &UmapInfoFile) -> HashMap<String, usize> {
        let mut spine_counts = HashMap::new();
        for map in &file.data.maps {
            let mut count = 0;
            let mut has_text = false;
            let mut has_term = false;
            for field in &map.fields {
                match field {
                    UmapField::InterText(lines) => {
                        if lines.len() != 1 || lines[0].to_lowercase() != "clear" {
                            has_text = true;
                        }
                    }
                    UmapField::EndGame(v) | UmapField::EndBunny(v) | UmapField::EndCast(v)
                        if *v =>
                    {
                        has_term = true;
                    }
                    UmapField::EndPic(_) => has_term = true,
                    _ => {}
                }
            }
            if has_text {
                count += 1;
            }
            if has_term {
                count += 1;
            }
            spine_counts.insert(map.mapname.to_uppercase(), count);
        }
        spine_counts
    }

    /// Assigns theoretical coordinates based on structure (which "track" the
    /// flow is on, normal or secret depth).
    fn calculate_topological_grid(
        file: &UmapInfoFile,
        spine_counts: &HashMap<String, usize>,
    ) -> HashMap<String, (f32, f32)> {
        let mut map_coords: HashMap<String, (f32, f32)> = HashMap::new();
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        let episode_maps: std::collections::HashSet<String> = file
            .data
            .maps
            .iter()
            .filter_map(|m| {
                let has_episode = m.fields.iter().any(|f| {
                    if let UmapField::Episode { patch, .. } = f {
                        patch.to_lowercase() != "clear"
                    } else {
                        false
                    }
                });
                if has_episode {
                    Some(m.mapname.to_uppercase())
                } else {
                    None
                }
            })
            .collect();

        for map in &file.data.maps {
            let name = map.mapname.to_uppercase();
            if !visited.contains(&name) {
                let start_t = 0.0;
                let max_r = map_coords
                    .values()
                    .map(|(_, r)| *r)
                    .fold(0.0_f32, |a, b| a.max(b));
                let actual_start_r = if map_coords.is_empty() {
                    0.0
                } else {
                    max_r + 2.0
                };

                queue.push_back((name, start_t, actual_start_r));

                while let Some((curr_name, curr_t, curr_r)) = queue.pop_front() {
                    if visited.contains(&curr_name) {
                        continue;
                    }
                    visited.insert(curr_name.clone());
                    map_coords.insert(curr_name.clone(), (curr_t, curr_r));

                    if let Some(map_entry) = file
                        .data
                        .maps
                        .iter()
                        .find(|m| m.mapname.to_uppercase() == curr_name)
                    {
                        for field in &map_entry.fields {
                            match field {
                                UmapField::Next(target) => {
                                    let t_name = target.to_uppercase();
                                    if !visited.contains(&t_name) {
                                        let spine_blocker =
                                            spine_counts.get(&curr_name).cloned().unwrap_or(0)
                                                as f32;
                                        let target_has_episode =
                                            episode_maps.contains(&t_name) as i32 as f32;
                                        let episode_spacing = target_has_episode * 1.0;
                                        let total_offset = 1.0 + spine_blocker + episode_spacing;
                                        queue.push_back((t_name, curr_t, curr_r + total_offset));
                                    }
                                }
                                UmapField::NextSecret(target) => {
                                    let t_name = target.to_uppercase();
                                    if !visited.contains(&t_name) {
                                        queue.push_back((t_name, curr_t + 2.0, curr_r));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        map_coords
    }

    /// Instantiates the actual nodes and edges using calculated positions.
    fn construct_graph(file: &UmapInfoFile, map_coords: &HashMap<String, (f32, f32)>) -> Self {
        let mut graph = UmapGraph::default();

        for map in &file.data.maps {
            let map_id = map.mapname.to_uppercase();
            let data = ParsedMapData::extract(map);

            let (t, r) = map_coords.get(&map_id).copied().unwrap_or((0.0, 0.0));
            let x_base = 60.0 + (t * 160.0);
            let y_base = 60.0 + (r * 80.0);

            let (map_vx, map_vy) = Self::get_pos(&file.metadata, &map_id, x_base, y_base);

            for (ep_idx, (patch, name)) in data.episode_fields.into_iter().enumerate() {
                let ep_id = format!("{}::EPISODE::{}", map_id, ep_idx);
                let (vx, vy) = Self::get_pos(&file.metadata, &ep_id, map_vx, map_vy - 80.0);
                graph.nodes.push(UmapNode {
                    id: ep_id.clone(),
                    node_type: NodeType::Episode { name, patch },
                    x: vx,
                    y: vy,
                });
                graph.edges.push(UmapEdge {
                    source: ep_id,
                    target: map_id.clone(),
                    edge_type: EdgeType::Normal,
                });
            }

            graph.nodes.push(UmapNode {
                id: map_id.clone(),
                node_type: NodeType::Map {
                    levelname: data.levelname,
                },
                x: map_vx,
                y: map_vy,
            });

            let mut current_v_stack_y = map_vy + 80.0;
            let mut last_node_id = map_id.clone();

            if data.has_text_normal {
                let text_id = format!("{}::TEXT_NORMAL", map_id);
                let (vx, vy) = Self::get_pos(&file.metadata, &text_id, map_vx, current_v_stack_y);
                graph.nodes.push(UmapNode {
                    id: text_id.clone(),
                    node_type: NodeType::InterText { is_secret: false },
                    x: vx,
                    y: vy,
                });
                graph.edges.push(UmapEdge {
                    source: last_node_id,
                    target: text_id.clone(),
                    edge_type: EdgeType::Normal,
                });
                last_node_id = text_id;
                current_v_stack_y += 80.0;
            }

            if let Some(term) = data.terminal_type {
                let term_id = format!("{}::TERMINAL", map_id);
                let (vx, vy) = Self::get_pos(&file.metadata, &term_id, map_vx, current_v_stack_y);
                graph.nodes.push(UmapNode {
                    id: term_id.clone(),
                    node_type: NodeType::Terminal { end_type: term },
                    x: vx,
                    y: vy,
                });
                graph.edges.push(UmapEdge {
                    source: last_node_id,
                    target: term_id,
                    edge_type: EdgeType::Normal,
                });
            } else if let Some(dest_id) = data.next {
                graph.edges.push(UmapEdge {
                    source: last_node_id,
                    target: dest_id,
                    edge_type: EdgeType::Normal,
                });
            }

            if let Some(sec_dest) = data.next_secret {
                if data.has_text_secret {
                    let text_id = format!("{}::TEXT_SECRET", map_id);
                    let (vx, vy) = Self::get_pos(&file.metadata, &text_id, map_vx + 160.0, map_vy);
                    graph.nodes.push(UmapNode {
                        id: text_id.clone(),
                        node_type: NodeType::InterText { is_secret: true },
                        x: vx,
                        y: vy,
                    });
                    graph.edges.push(UmapEdge {
                        source: map_id.clone(),
                        target: text_id.clone(),
                        edge_type: EdgeType::Secret,
                    });
                    graph.edges.push(UmapEdge {
                        source: text_id,
                        target: sec_dest,
                        edge_type: EdgeType::Secret,
                    });
                } else {
                    graph.edges.push(UmapEdge {
                        source: map_id.clone(),
                        target: sec_dest,
                        edge_type: EdgeType::Secret,
                    });
                }
            }
        }
        graph
    }

    /// Fetches node positions from metadata, falling back to a default if missing.
    pub fn get_pos(metadata: &Value, id: &str, def_x: f32, def_y: f32) -> (f32, f32) {
        if let Some(pos) = metadata
            .get("node_positions")
            .and_then(|p| p.get(id))
            .and_then(|arr| arr.as_array())
        {
            if pos.len() == 2 {
                if let (Some(x), Some(y)) = (pos[0].as_f64(), pos[1].as_f64()) {
                    return (x as f32, y as f32);
                }
            }
        }
        (def_x, def_y)
    }
}
