# `bundle.resources`（roles / plugins）

配置：[`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json) 中 `resources`。

**PowerShell（仓库根）**

```powershell
$r = (Get-ChildItem roles -Recurse -File | Measure-Object -Property Length -Sum).Sum
$p = if (Test-Path plugins) { (Get-ChildItem plugins -Recurse -File | Measure-Object -Property Length -Sum).Sum } else { 0 }
"roles=$r plugins=$p total=$(( $r + $p ))"
```

瘦包 / 外置 / 分包属产品决策，默认不改配置。
