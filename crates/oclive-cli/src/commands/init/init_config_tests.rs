mod template_tests {
    use std::path::PathBuf;

    use super::super::*;
    use super::super::super::InitArgs;
    use crate::pipeline::PipelineArg;

    #[test]
    fn template_robot_soul_defaults() {
        let td = template_defaults(InitTemplateArg::RobotSoul);
        assert_eq!(td.preset, "minimal");
        assert!(td.monolith_default);
        assert_eq!(td.project_type, ProjectType::KernelServer);
        assert_eq!(td.role_pack, RolePackKind::RobotSoulMinimal);
    }

    #[test]
    fn template_headless_api_defaults() {
        let td = template_defaults(InitTemplateArg::HeadlessApi);
        assert_eq!(td.preset, "full");
        assert!(!td.monolith_default);
        assert_eq!(td.role_pack, RolePackKind::None);
    }

    #[test]
    fn preset_override_wins_over_template() {
        let args = InitArgs {
            output: PathBuf::from("out"),
            non_interactive: true,
            quiet: true,
            template: Some(InitTemplateArg::RobotSoul),
            preset: Some("full".into()),
            project_name: "t".into(),
            project_type: None,
            backend_memory: None,
            backend_emotion: None,
            backend_event: None,
            backend_prompt: None,
            backend_llm: None,
            backend_agent: None,
            backend_complex_emotion: None,
            monolith: false,
            monolith_preset: None,
            monolith_bench_preset: None,
            dual_core: false,
            list_templates: false,
            quick: false,
            skip_role_pack: false,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
            author: None,
            license: None,
            description: None,
            template_url: None,
            tui: false,
            smart: false,
            no_smart: true,
            pipeline: PipelineArg::Default,
            weld_modules: vec![],
            from_existing: None,
            share: false,
            json: false,
            dry_run: false,
            check: false,
            chat_storage_location: None,
        };
        let preset = args.preset.as_deref().unwrap_or("minimal");
        let mut cfg = preset_config("t", preset);
        apply_template_layer(&args, &mut cfg);
        assert_eq!(cfg.backends.llm, BackendImpl::Remote);
    }

    #[test]
    fn robot_soul_template_enables_monolith_without_flag() {
        let args = InitArgs {
            output: PathBuf::from("out"),
            non_interactive: true,
            quiet: true,
            template: Some(InitTemplateArg::RobotSoul),
            preset: None,
            project_name: "t".into(),
            project_type: None,
            backend_memory: None,
            backend_emotion: None,
            backend_event: None,
            backend_prompt: None,
            backend_llm: None,
            backend_agent: None,
            backend_complex_emotion: None,
            monolith: false,
            monolith_preset: None,
            monolith_bench_preset: None,
            dual_core: false,
            list_templates: false,
            quick: false,
            skip_role_pack: false,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
            author: None,
            license: None,
            description: None,
            template_url: None,
            tui: false,
            smart: false,
            no_smart: true,
            pipeline: PipelineArg::Default,
            weld_modules: vec![],
            from_existing: None,
            share: false,
            json: false,
            dry_run: false,
            check: false,
            chat_storage_location: None,
        };
        let mut cfg = preset_config("t", "minimal");
        apply_template_layer(&args, &mut cfg);
        resolve_monolith(&args, &mut cfg);
        assert!(cfg.monolith_enabled);
    }

    #[test]
    fn with_role_pack_overrides_template_default() {
        let args = InitArgs {
            output: PathBuf::from("out"),
            non_interactive: true,
            quiet: true,
            template: Some(InitTemplateArg::RobotSoul),
            preset: None,
            project_name: "t".into(),
            project_type: None,
            backend_memory: None,
            backend_emotion: None,
            backend_event: None,
            backend_prompt: None,
            backend_llm: None,
            backend_agent: None,
            backend_complex_emotion: None,
            monolith: false,
            monolith_preset: None,
            monolith_bench_preset: None,
            dual_core: false,
            list_templates: false,
            quick: false,
            skip_role_pack: false,
            with_role_pack: Some(RolePackKindArg::Default),
            with_example_plugin: false,
            kernel_source: None,
            author: None,
            license: None,
            description: None,
            template_url: None,
            tui: false,
            smart: false,
            no_smart: true,
            pipeline: PipelineArg::Default,
            weld_modules: vec![],
            from_existing: None,
            share: false,
            json: false,
            dry_run: false,
            check: false,
            chat_storage_location: None,
        };
        assert_eq!(resolve_role_pack_kind(&args), RolePackKind::DefaultExample);
    }

    #[test]
    fn quick_config_uses_full_without_roles() {
        let cfg = quick_project_config("q");
        assert_eq!(cfg.backends.llm, BackendImpl::Remote);
        assert!(!cfg.monolith_enabled);
        assert_eq!(cfg.role_pack_kind, RolePackKind::None);
    }

    #[test]
    fn monolith_bench_preset_enables_post_bench() {
        let args = InitArgs {
            output: PathBuf::from("out"),
            non_interactive: true,
            quiet: true,
            template: None,
            preset: Some("minimal".into()),
            project_name: "t".into(),
            project_type: Some(ProjectTypeArg::KernelServer),
            backend_memory: None,
            backend_emotion: None,
            backend_event: None,
            backend_prompt: None,
            backend_llm: None,
            backend_agent: None,
            backend_complex_emotion: None,
            monolith: false,
            monolith_preset: None,
            monolith_bench_preset: Some(MonolithPresetArg::Latency),
            dual_core: false,
            list_templates: false,
            quick: false,
            skip_role_pack: true,
            with_role_pack: None,
            with_example_plugin: false,
            kernel_source: None,
            author: None,
            license: None,
            description: None,
            template_url: None,
            tui: false,
            smart: false,
            no_smart: true,
            pipeline: PipelineArg::Default,
            weld_modules: vec![],
            from_existing: None,
            share: false,
            json: false,
            dry_run: false,
            check: false,
            chat_storage_location: None,
        };
        let mut cfg = preset_config("t", "minimal");
        apply_backend_cli_overrides(&mut cfg, &args);
        cfg.project_type = ProjectType::KernelServer;
        resolve_monolith(&args, &mut cfg);
        if args.monolith_bench_preset.is_some() {
            cfg.monolith_enabled = true;
            cfg.monolith_preset = args.monolith_bench_preset;
            cfg.run_monolith_bench_after_init = true;
        }
        assert!(cfg.run_monolith_bench_after_init);
        assert!(cfg.monolith_enabled);
    }

    #[test]
    fn robot_gateway_template_defaults() {
        let td = template_defaults(InitTemplateArg::RobotGateway);
        assert_eq!(td.preset, "mixed");
        assert!(td.monolith_default);
        assert_eq!(td.role_pack, RolePackKind::None);
    }

    #[test]
    fn dialogue_only_template_defaults() {
        let td = template_defaults(InitTemplateArg::DialogueOnly);
        assert_eq!(td.preset, "full");
        assert!(!td.monolith_default);
        assert_eq!(td.role_pack, RolePackKind::DefaultExample);
    }

    #[test]
    fn monolith_preset_latency_welds_all_slots() {
        let mut cfg = preset_config("t", "minimal");
        cfg.monolith_enabled = true;
        cfg.monolith_preset = Some(MonolithPresetArg::Latency);
        let weld = resolve_monolith_weld_modules(&cfg);
        assert_eq!(weld.len(), 7);
    }

    #[test]
    fn pipeline_memory_last_llm_before_memory_in_order_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = preset_config("pipe", "minimal");
        cfg.pipeline = PipelineArg::MemoryLast;
        cfg.monolith_enabled = false;
        crate::generator::write_project(&cfg, dir.path()).unwrap();
        let raw = std::fs::read_to_string(dir.path().join("src/oclive_pipeline_order.rs")).unwrap();
        let llm = raw.find("llm_generate").expect("llm_generate");
        let mem = raw.find("memory_rank").expect("memory_rank");
        assert!(
            llm < mem,
            "memory-last: llm before memory in OCLIVE_PIPELINE_STEPS"
        );
    }

    #[test]
    fn pipeline_emotion_first_memory_before_event() {
        let steps = PipelineArg::EmotionFirst.steps();
        let em = steps
            .iter()
            .position(|s| *s == "user_emotion_analyze")
            .unwrap();
        let ev = steps.iter().position(|s| *s == "event_estimate").unwrap();
        let mem = steps.iter().position(|s| *s == "memory_rank").unwrap();
        assert!(em < ev);
        assert!(mem < ev);
    }
}
