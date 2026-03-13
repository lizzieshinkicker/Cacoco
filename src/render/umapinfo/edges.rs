use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::models::umap_graph::{EdgeType, UmapGraph};
use crate::ui::properties::editor::ViewportContext;

#[derive(Clone, Default)]
pub struct EdgePath {
    pub edge_type: EdgeType,
    pub virtual_points: Vec<eframe::egui::Pos2>,
    pub is_fallback: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct AStarState {
    cost: i32,
    x: i32,
    y: i32,
    dir: i32,
}

impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn make_seg(p1: (i32, i32), p2: (i32, i32)) -> ((i32, i32), (i32, i32)) {
    if p1 < p2 { (p1, p2) } else { (p2, p1) }
}

/// Pre-calculates orthogonal paths between nodes.
/// Tries a few passes to optimize the layout to look nice.
pub fn calculate_all_edge_paths(graph: &UmapGraph) -> Vec<EdgePath> {
    let mut blocked = HashSet::new();

    let mut min_x = 0;
    let mut max_x = 0;
    let mut min_y = 0;
    let mut max_y = 0;

    if let Some(first) = graph.nodes.first() {
        min_x = (first.x / 20.0).round() as i32;
        max_x = min_x;
        min_y = (first.y / 20.0).round() as i32;
        max_y = min_y;
    }

    for node in &graph.nodes {
        let gx = (node.x / 20.0).round() as i32;
        let gy = (node.y / 20.0).round() as i32;
        min_x = min_x.min(gx);
        max_x = max_x.max(gx + 6);
        min_y = min_y.min(gy);
        max_y = max_y.max(gy + 2);

        for x in gx..=(gx + 6) {
            for y in gy..=(gy + 2) {
                blocked.insert((x, y));
            }
        }
    }

    min_x -= 20;
    max_x += 20;
    min_y -= 20;
    max_y += 20;

    let mut edge_priorities: HashMap<usize, i32> = HashMap::new();
    let mut best_paths = Vec::new();
    let mut lowest_total_penalty = i32::MAX;

    for _iteration in 0..4 {
        let mut point_tracks: HashMap<(i32, i32), (String, bool)> = HashMap::new();
        let mut segment_tracks: HashMap<((i32, i32), (i32, i32)), (String, bool)> = HashMap::new();
        let mut corners: HashMap<(i32, i32), (String, bool)> = HashMap::new();

        let mut current_paths = Vec::new();
        let mut current_total_penalty = 0;

        let mut ordered_indices: Vec<usize> = (0..graph.edges.len()).collect();
        ordered_indices.sort_by_key(|&idx| {
            let prev_penalty = edge_priorities.get(&idx).copied().unwrap_or(0);
            let e = &graph.edges[idx];
            let src = graph.nodes.iter().find(|n| n.id == e.source).unwrap();
            let dst = graph.nodes.iter().find(|n| n.id == e.target).unwrap();
            let dist = ((src.x - dst.x).abs() + (src.y - dst.y).abs()) as i32;
            (-prev_penalty, dist)
        });

        for &edge_idx in &ordered_indices {
            let edge = &graph.edges[edge_idx];
            let src_node = graph.nodes.iter().find(|n| n.id == edge.source).unwrap();
            let dst_node = graph.nodes.iter().find(|n| n.id == edge.target).unwrap();

            let is_secret = edge.edge_type == EdgeType::Secret;
            let sgx = (src_node.x / 20.0).round() as i32;
            let sgy = (src_node.y / 20.0).round() as i32;
            let dgx = (dst_node.x / 20.0).round() as i32;
            let dgy = (dst_node.y / 20.0).round() as i32;

            let (start_pt, pre_start, start_dir, end_pt, pre_end) = if is_secret {
                (
                    (sgx + 6, sgy + 1),
                    (sgx + 7, sgy + 1),
                    2,
                    (dgx, dgy + 1),
                    (dgx - 1, dgy + 1),
                )
            } else {
                (
                    (sgx + 3, sgy + 2),
                    (sgx + 3, sgy + 3),
                    3,
                    (dgx + 3, dgy),
                    (dgx + 3, dgy - 1),
                )
            };

            let mut heap = BinaryHeap::new();
            let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
            let mut cost_so_far: HashMap<(i32, i32), i32> = HashMap::new();

            heap.push(AStarState {
                cost: 0,
                x: pre_start.0,
                y: pre_start.1,
                dir: start_dir,
            });
            cost_so_far.insert(pre_start, 0);

            let mut found = false;
            while let Some(AStarState { cost: _, x, y, dir }) = heap.pop() {
                if (x, y) == pre_end {
                    found = true;
                    break;
                }

                for (nx, ny, ndir) in [(x, y - 1, 1), (x + 1, y, 2), (x, y + 1, 3), (x - 1, y, 4)] {
                    if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                        continue;
                    }
                    if blocked.contains(&(nx, ny)) && (nx, ny) != pre_end && (nx, ny) != pre_start {
                        continue;
                    }

                    let seg = make_seg((x, y), (nx, ny));
                    let mut step_cost = 10;

                    if let Some((owner, owner_is_secret)) = segment_tracks.get(&seg) {
                        if owner == &edge.target && *owner_is_secret == is_secret {
                            step_cost = 2;
                        } else {
                            step_cost = 10000;
                        }
                    }

                    let mut new_cost = cost_so_far[&(x, y)] + step_cost;
                    let is_turning = dir != 0 && dir != ndir;
                    if is_turning {
                        new_cost += 20;
                    }

                    if let Some((owner, owner_is_secret)) = point_tracks.get(&(nx, ny)) {
                        if owner != &edge.target || *owner_is_secret != is_secret {
                            if is_turning {
                                new_cost += 10000;
                            } else {
                                new_cost += 30;
                            }
                        }
                    }

                    if let Some((owner, owner_is_secret)) = corners.get(&(nx, ny)) {
                        if owner != &edge.target || *owner_is_secret != is_secret {
                            new_cost += 10000;
                        }
                    }

                    if !cost_so_far.contains_key(&(nx, ny)) || new_cost < cost_so_far[&(nx, ny)] {
                        cost_so_far.insert((nx, ny), new_cost);
                        let h = ((nx - pre_end.0).abs() + (ny - pre_end.1).abs()) * 10;
                        heap.push(AStarState {
                            cost: new_cost + h,
                            x: nx,
                            y: ny,
                            dir: ndir,
                        });
                        came_from.insert((nx, ny), (x, y));
                    }
                }
            }

            if !found {
                current_total_penalty += 50000;
                edge_priorities.insert(edge_idx, 50000);

                let src_rect = eframe::egui::Rect::from_min_size(
                    eframe::egui::pos2(src_node.x, src_node.y),
                    eframe::egui::vec2(120.0, 40.0),
                );
                let dst_rect = eframe::egui::Rect::from_min_size(
                    eframe::egui::pos2(dst_node.x, dst_node.y),
                    eframe::egui::vec2(120.0, 40.0),
                );
                let s = if is_secret {
                    src_rect.right_center()
                } else {
                    src_rect.center_bottom()
                };
                let mut d = if is_secret {
                    dst_rect.left_center()
                } else {
                    dst_rect.center_top()
                };
                if is_secret {
                    d.x += 4.0;
                } else {
                    d.y += 4.0;
                }

                let (cp1, cp2) = if is_secret {
                    let dist = (d.x - s.x).abs().max(40.0) * 0.5;
                    (
                        s + eframe::egui::vec2(dist, 0.0),
                        d - eframe::egui::vec2(dist, 0.0),
                    )
                } else {
                    let dist = (d.y - s.y).abs().max(40.0) * 0.5;
                    (
                        s + eframe::egui::vec2(0.0, dist),
                        d - eframe::egui::vec2(0.0, dist),
                    )
                };
                current_paths.push(EdgePath {
                    edge_type: edge.edge_type.clone(),
                    virtual_points: vec![s, cp1, cp2, d],
                    is_fallback: true,
                });
                continue;
            }

            let mut raw_path = Vec::new();
            let mut curr = pre_end;
            while curr != pre_start {
                raw_path.push(curr);
                curr = came_from[&curr];
            }
            raw_path.push(pre_start);
            raw_path.reverse();

            let base_cost = raw_path.len() as i32 * 10;
            let actual_cost = cost_so_far[&pre_end];
            let penalty = (actual_cost - base_cost).max(0);

            edge_priorities.insert(edge_idx, penalty);
            current_total_penalty += penalty;

            let target_sig = (edge.target.clone(), is_secret);
            corners.insert(pre_start, target_sig.clone());
            corners.insert(pre_end, target_sig.clone());

            for i in 0..raw_path.len() {
                point_tracks.insert(raw_path[i], target_sig.clone());
                if i > 0 {
                    segment_tracks
                        .insert(make_seg(raw_path[i - 1], raw_path[i]), target_sig.clone());
                }
            }
            segment_tracks.insert(make_seg(start_pt, pre_start), target_sig.clone());
            segment_tracks.insert(make_seg(pre_end, end_pt), target_sig.clone());

            let mut full_path = vec![start_pt];
            full_path.extend(raw_path);
            full_path.push(end_pt);

            let mut final_pts = Vec::new();
            for i in 0..full_path.len() {
                let p = full_path[i];
                let is_corner = if i == 0 || i == full_path.len() - 1 {
                    true
                } else {
                    let (pr, c, n) = (full_path[i - 1], full_path[i], full_path[i + 1]);
                    !((c.0 - pr.0 == n.0 - c.0) && (c.1 - pr.1 == n.1 - c.1))
                };

                if is_corner {
                    corners.insert(p, target_sig.clone());
                    final_pts.push(eframe::egui::pos2(p.0 as f32 * 20.0, p.1 as f32 * 20.0));
                }
            }
            current_paths.push(EdgePath {
                edge_type: edge.edge_type.clone(),
                virtual_points: final_pts,
                is_fallback: false,
            });
        }

        if current_total_penalty < lowest_total_penalty {
            lowest_total_penalty = current_total_penalty;
            best_paths = current_paths.clone();
        }

        if lowest_total_penalty < 1000 {
            break;
        }
    }

    best_paths
}

fn get_edge_color(edge_type: &EdgeType) -> eframe::egui::Color32 {
    if *edge_type == EdgeType::Secret {
        eframe::egui::Color32::from_rgb(200, 100, 200)
    } else {
        eframe::egui::Color32::from_rgb(100, 200, 100)
    }
}

pub fn draw_edge_lines(painter: &eframe::egui::Painter, paths: &[EdgePath], ctx: &ViewportContext) {
    let scale = ctx.proj.final_scale_x;
    let tip_inset = 4.0 * scale;

    let render_pass = |is_fallback_pass: bool| {
        for path in paths.iter().filter(|p| p.is_fallback == is_fallback_pass) {
            let base_color = get_edge_color(&path.edge_type);
            let mut sp: Vec<_> = path
                .virtual_points
                .iter()
                .map(|&p| ctx.proj.to_screen(p))
                .collect();

            if path.is_fallback {
                painter.add(eframe::egui::epaint::CubicBezierShape::from_points_stroke(
                    [sp[0], sp[1], sp[2], sp[3]],
                    false,
                    eframe::egui::Color32::TRANSPARENT,
                    eframe::egui::Stroke::new(2.5 * scale, base_color.gamma_multiply(0.2)),
                ));
            } else {
                if sp.len() >= 2 {
                    let last = sp.len() - 1;
                    let dir = (sp[last - 1] - sp[last]).normalized();
                    if dir.is_finite() {
                        sp[last] = sp[last] + dir * tip_inset;
                    }
                }
                painter.add(eframe::egui::Shape::line(
                    sp,
                    eframe::egui::Stroke::new(2.5 * scale, base_color),
                ));
            }
        }
    };

    render_pass(true);
    render_pass(false);
}

pub fn draw_edge_arrows(
    painter: &eframe::egui::Painter,
    paths: &[EdgePath],
    ctx: &ViewportContext,
) {
    let scale = ctx.proj.final_scale_x;
    let (aw, ah) = (5.0 * scale, 8.0 * scale);

    for path in paths {
        let is_sec = path.edge_type == EdgeType::Secret;
        let mut color = get_edge_color(&path.edge_type);

        if path.is_fallback {
            color = color.gamma_multiply(0.2);
        }

        if let Some(&v_dst) = path.virtual_points.last() {
            let dp = ctx.proj.to_screen(v_dst);
            let pts = if is_sec {
                vec![
                    dp,
                    dp + eframe::egui::vec2(-ah, -aw),
                    dp + eframe::egui::vec2(-ah, aw),
                ]
            } else {
                vec![
                    dp,
                    dp + eframe::egui::vec2(-aw, -ah),
                    dp + eframe::egui::vec2(aw, -ah),
                ]
            };
            painter.add(eframe::egui::Shape::convex_polygon(
                pts,
                color,
                eframe::egui::Stroke::NONE,
            ));
        }
    }
}
