use crate::option::SeriesOption;

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
    pub fn analyze(spec_series: &[usize], option: &crate::option::ChartOption) -> Vec<GroupPlan> {
        let mut plans: Vec<GroupPlan> = Vec::new();
        let mut visited = vec![false; spec_series.len()];

        for (local_idx, &global_idx) in spec_series.iter().enumerate() {
            if visited[local_idx] {
                continue;
            }

            let series = &option.series[global_idx];

            match series {
                SeriesOption::Bar(bar) => {
                    let grid_idx = bar.grid_index.unwrap_or(0);

                    if let Some(stack_name) = &bar.stack {
                        let mut group_indices = Vec::new();
                        for (j, &other_idx) in spec_series.iter().enumerate() {
                            if visited[j] {
                                continue;
                            }
                            if let SeriesOption::Bar(other_bar) = &option.series[other_idx] {
                                if other_bar.grid_index.unwrap_or(0) == grid_idx
                                    && other_bar.stack.as_ref() == Some(stack_name)
                                {
                                    visited[j] = true;
                                    group_indices.push(other_idx);
                                }
                            }
                        }
                        plans.push(GroupPlan {
                            series_indices: group_indices,
                            group_type: GroupType::Stacked,
                        });
                    } else {
                        let bar_group = bar.group_index.unwrap_or(0);
                        let mut group_indices = Vec::new();
                        for (j, &other_idx) in spec_series.iter().enumerate() {
                            if visited[j] {
                                continue;
                            }
                            if let SeriesOption::Bar(other_bar) = &option.series[other_idx] {
                                if other_bar.grid_index.unwrap_or(0) == grid_idx
                                    && other_bar.group_index.unwrap_or(0) == bar_group
                                    && other_bar.stack.is_none()
                                {
                                    visited[j] = true;
                                    group_indices.push(other_idx);
                                }
                            }
                        }
                        plans.push(GroupPlan {
                            series_indices: group_indices,
                            group_type: GroupType::SideBySide,
                        });
                    }
                }
                _ => {
                    visited[local_idx] = true;
                    plans.push(GroupPlan {
                        series_indices: vec![global_idx],
                        group_type: GroupType::Single,
                    });
                }
            }
        }

        plans
    }
}
