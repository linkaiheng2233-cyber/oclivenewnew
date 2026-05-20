//! 目录插件 manifest `plugin_dependencies` 字段校验。

use serde_json::Value;
use std::collections::HashSet;

/// 从 manifest JSON 解析 `plugin_dependencies`（缺省 `[]`）。
pub fn parse_plugin_dependencies(manifest_json: &str) -> Result<Vec<String>, String> {
    let v: Value = serde_json::from_str(manifest_json)
        .map_err(|e| format!("manifest JSON 错误: {e}"))?;
    let Some(obj) = v.as_object() else {
        return Err("manifest 根须为对象".into());
    };
    match obj.get("plugin_dependencies") {
        None => Ok(vec![]),
        Some(Value::Array(arr)) => {
            let mut out = Vec::with_capacity(arr.len());
            for (i, item) in arr.iter().enumerate() {
                let Some(s) = item.as_str() else {
                    return Err(format!("plugin_dependencies[{i}] 须为字符串"));
                };
                if s.trim().is_empty() {
                    return Err(format!("plugin_dependencies[{i}] 为空"));
                }
                out.push(s.trim().to_string());
            }
            Ok(out)
        }
        Some(_) => Err("plugin_dependencies 须为字符串数组".into()),
    }
}

/// 拓扑排序安装顺序；`available` 为已安装/可解析的 id 集合。
pub fn resolve_install_order(
    root_id: &str,
    load_deps: impl Fn(&str) -> Result<Vec<String>, String>,
) -> Result<Vec<String>, String> {
    let mut order = Vec::new();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();

    fn visit(
        id: &str,
        load_deps: &impl Fn(&str) -> Result<Vec<String>, String>,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if visited.contains(id) {
            return Ok(());
        }
        if !visiting.insert(id.to_string()) {
            return Err(format!("插件依赖存在循环: 涉及 {id}"));
        }
        for dep in load_deps(id)? {
            visit(&dep, load_deps, visiting, visited, order)?;
        }
        visiting.remove(id);
        visited.insert(id.to_string());
        order.push(id.to_string());
        Ok(())
    }

    visit(root_id, &load_deps, &mut visiting, &mut visited, &mut order)?;
    Ok(order)
}

/// 哪些已安装插件声明依赖 `target_id`。
pub fn dependents_of(
    installed: &[(String, String)],
    target_id: &str,
) -> Vec<String> {
    installed
        .iter()
        .filter_map(|(id, manifest)| {
            parse_plugin_dependencies(manifest)
                .ok()
                .filter(|deps| deps.iter().any(|d| d == target_id))
                .map(|_| id.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn detects_cycle() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("a".into(), vec!["b".into()]);
        graph.insert("b".into(), vec!["a".into()]);
        let load = |id: &str| -> Result<Vec<String>, String> {
            graph
                .get(id)
                .cloned()
                .ok_or_else(|| format!("missing {id}"))
        };
        assert!(resolve_install_order("a", load).is_err());
    }
}
