use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    path_rules::{
        normalize_repo_path, reasoned_selections, selector_kind_name, selector_matches,
        sorted_unique, tier_name,
    },
    CiPlan, CiPlanError, FallbackDecision, PlanRequest, PlannedValidator, Planner,
    SkippedValidator, PLAN_SCHEMA_VERSION,
};

impl Planner {
    /// Build a deterministic validation plan for the supplied change set.
    ///
    /// Any uncertainty in module metadata, changed-path ownership, or configured risk
    /// overrides produces a policy-scoped full fallback. The planner only selects trusted
    /// catalog coordinates; it never executes validation commands.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested validation policy is not present in the loaded
    /// catalog.
    pub fn plan(&self, request: PlanRequest) -> Result<CiPlan, CiPlanError> {
        let policy = self
            .catalog
            .policies
            .iter()
            .find(|policy| policy.id == request.policy)
            .ok_or_else(|| CiPlanError::UnknownPolicy(request.policy.clone()))?;
        let included_tiers = policy
            .included_tiers
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();

        let mut changed_files = BTreeSet::new();
        let mut fallback_reasons = BTreeSet::new();
        for changed_file in request.changed_files {
            match normalize_repo_path(&changed_file) {
                Ok(path) => {
                    changed_files.insert(path);
                }
                Err(message) => {
                    fallback_reasons
                        .insert(format!("invalid_changed_path:{changed_file}:{message}"));
                }
            }
        }
        if changed_files.is_empty() {
            fallback_reasons.insert("no_changed_files".to_owned());
        }

        for (module_id, loaded) in &self.descriptors {
            for issue in &loaded.issues {
                fallback_reasons.insert(format!("module_metadata:{module_id}:{issue}"));
            }
        }

        let mut direct = BTreeMap::<String, BTreeSet<String>>::new();
        for changed_file in &changed_files {
            let mut matched = false;
            for binding in &self.impact_map.module_bindings {
                for selector in &binding.selectors {
                    if selector_matches(selector, changed_file) {
                        matched = true;
                        direct
                            .entry(binding.module_id.clone())
                            .or_default()
                            .insert(format!(
                                "changed_path:{changed_file}:{}:{}",
                                selector_kind_name(selector.kind),
                                selector.value
                            ));
                    }
                }
            }
            if !matched {
                fallback_reasons.insert(format!("unmapped_changed_path:{changed_file}"));
            }
        }

        let mut forced_profiles = BTreeMap::<String, BTreeSet<String>>::new();
        for risk in &self.impact_map.risk_overrides {
            let matches = changed_files.iter().any(|path| {
                risk.selectors
                    .iter()
                    .any(|selector| selector_matches(selector, path))
            });
            if !matches {
                continue;
            }
            if risk.full {
                fallback_reasons.insert(format!("risk_override:{}:{}", risk.id, risk.reason));
            }
            for profile in &risk.force_profiles {
                forced_profiles
                    .entry(profile.clone())
                    .or_default()
                    .insert(format!("risk_override:{}:{}", risk.id, risk.reason));
            }
        }

        let mut affected = direct
            .keys()
            .map(|id| (id.clone(), BTreeSet::from(["direct_change".to_owned()])))
            .collect::<BTreeMap<_, _>>();
        let mut queue = direct.keys().cloned().collect::<VecDeque<_>>();
        let mut visited = BTreeSet::new();
        while let Some(source) = queue.pop_front() {
            if !visited.insert(source.clone()) {
                continue;
            }
            let mut targets = BTreeMap::<String, BTreeSet<String>>::new();
            if let Some(policy_targets) = self.impact_map.policy_affects.get(&source) {
                for target in policy_targets {
                    targets
                        .entry(target.clone())
                        .or_default()
                        .insert(format!("policy_affects:{source}"));
                }
            }
            if let Some(Some(descriptor)) = self
                .descriptors
                .get(&source)
                .map(|loaded| loaded.descriptor.as_ref())
            {
                for target in &descriptor.declared_affects {
                    targets
                        .entry(target.clone())
                        .or_default()
                        .insert(format!("declared_affects:{source}"));
                }
            }
            for (target, reasons) in targets {
                let was_new = !affected.contains_key(&target);
                affected.entry(target.clone()).or_default().extend(reasons);
                if was_new {
                    queue.push_back(target);
                }
            }
        }

        let full = !fallback_reasons.is_empty();
        if full {
            for module_id in self.descriptors.keys() {
                affected
                    .entry(module_id.clone())
                    .or_default()
                    .insert("full_fallback".to_owned());
            }
        }

        let mut selected_profiles = forced_profiles;
        if full {
            for profile in &self.catalog.profiles {
                selected_profiles
                    .entry(profile.id.clone())
                    .or_default()
                    .insert("full_fallback".to_owned());
            }
        } else {
            for module_id in affected.keys() {
                if let Some(Some(descriptor)) = self
                    .descriptors
                    .get(module_id)
                    .map(|loaded| loaded.descriptor.as_ref())
                {
                    for profile in &descriptor.validation_profiles {
                        selected_profiles
                            .entry(profile.clone())
                            .or_default()
                            .insert(format!("affected_module:{module_id}"));
                    }
                }
            }
        }

        let profiles_by_id = self
            .catalog
            .profiles
            .iter()
            .map(|profile| (profile.id.as_str(), profile))
            .collect::<BTreeMap<_, _>>();
        let mut validator_reasons = BTreeMap::<String, BTreeSet<String>>::new();
        for profile_id in selected_profiles.keys() {
            if let Some(profile) = profiles_by_id.get(profile_id.as_str()) {
                for validator_id in &profile.validators {
                    validator_reasons
                        .entry(validator_id.clone())
                        .or_default()
                        .insert(format!("validation_profile:{profile_id}"));
                }
            }
        }
        if full {
            for validator in &self.catalog.validators {
                if included_tiers.contains(&validator.tier) {
                    validator_reasons
                        .entry(validator.id.clone())
                        .or_default()
                        .insert("full_fallback".to_owned());
                }
            }
        }

        let mut selected_validators = Vec::new();
        let mut skipped_validators = Vec::new();
        for validator in &self.catalog.validators {
            if !included_tiers.contains(&validator.tier) {
                skipped_validators.push(SkippedValidator {
                    id: validator.id.clone(),
                    reason: format!("tier_not_in_policy:{}", tier_name(validator.tier)),
                });
                continue;
            }
            if let Some(reasons) = validator_reasons.get(&validator.id) {
                selected_validators.push(PlannedValidator {
                    id: validator.id.clone(),
                    tier: validator.tier,
                    gate: validator.gate,
                    platforms: sorted_unique(validator.platforms.clone()),
                    trust: validator.trust,
                    command_id: validator.command_id.clone(),
                    workflow_jobs: sorted_unique(validator.workflow_jobs.clone()),
                    reasons: reasons.iter().cloned().collect(),
                });
            } else {
                skipped_validators.push(SkippedValidator {
                    id: validator.id.clone(),
                    reason: "profile_not_selected".to_owned(),
                });
            }
        }
        selected_validators.sort_by(|left, right| left.id.cmp(&right.id));
        skipped_validators.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(CiPlan {
            schema_version: PLAN_SCHEMA_VERSION,
            base_sha: request.base_sha,
            head_sha: request.head_sha,
            policy: request.policy,
            shadow: request.shadow,
            changed_files: changed_files.into_iter().collect(),
            direct_modules: reasoned_selections(direct),
            affected_modules: reasoned_selections(affected),
            selected_profiles: reasoned_selections(selected_profiles),
            selected_validators,
            skipped_validators,
            fallback: FallbackDecision {
                full,
                reasons: fallback_reasons.into_iter().collect(),
            },
            warnings: self.warnings.clone(),
            impact_map_sha256: self.impact_map_sha256.clone(),
            validation_catalog_sha256: self.validation_catalog_sha256.clone(),
        })
    }
}
