# Experimental pipeline method registry

See [METHOD_REGISTRY.md](./METHOD_REGISTRY.md) for the canonical table. Action format:

```text
slot.<registry_key>.<method>
```

CLI:

```bash
cargo run -p oclive-cli -- explain DUAL_CORE
cargo run -p oclive-cli -- explain slot.emotion.analyze
```
