use crate::pipeline::types::{ChartType, SeriesSpec};

#[derive(Debug, Clone, PartialEq)]
pub enum GroupType {
    Single,
    SideBySide,
    Stacked,
}

#[derive(Debug, Clone)]
pub struct GroupPlan {
    pub series_indices: Vec<usize>,
    pub group_type: GroupType,
}

pub struct GroupAnalyzer;

impl GroupAnalyzer {
    pub fn analyze(spec_series: &[usize], series: &[SeriesSpec]) -> Vec<GroupPlan> {
        let mut plans: Vec<GroupPlan> = Vec::new();
        let mut visited = vec![false; spec_series.len()];

        for (local_idx, &global_idx) in spec_series.iter().enumerate() {
            if visited[local_idx] {
                continue;
            }

            let s = &series[global_idx];

            if s.chart_type == ChartType::Bar {
                if let Some(stack_name) = &s.stack {
                    let mut group_indices = Vec::new();
                    for (j, &other_idx) in spec_series.iter().enumerate() {
                        if visited[j] {
                            continue;
                        }
                        let other = &series[other_idx];
                        if other.chart_type == ChartType::Bar
                            && other.grid_index == s.grid_index
                            && other.stack.as_ref() == Some(stack_name)
                        {
                            visited[j] = true;
                            group_indices.push(other_idx);
                        }
                    }
                    plans.push(GroupPlan {
                        series_indices: group_indices,
                        group_type: GroupType::Stacked,
                    });
                } else {
                    let bar_group = s.group_index;
                    let mut group_indices = Vec::new();
                    for (j, &other_idx) in spec_series.iter().enumerate() {
                        if visited[j] {
                            continue;
                        }
                        let other = &series[other_idx];
                        if other.chart_type == ChartType::Bar
                            && other.grid_index == s.grid_index
                            && other.group_index == bar_group
                            && other.stack.is_none()
                        {
                            visited[j] = true;
                            group_indices.push(other_idx);
                        }
                    }
                    plans.push(GroupPlan {
                        series_indices: group_indices,
                        group_type: GroupType::SideBySide,
                    });
                }
            } else {
                visited[local_idx] = true;
                plans.push(GroupPlan {
                    series_indices: vec![global_idx],
                    group_type: GroupType::Single,
                });
            }
        }

        plans
    }
}
