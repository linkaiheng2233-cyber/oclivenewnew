# Check internal relative links in creator-docs/**/*.md (repository root).
# Exit 1 if any relative target file is missing. HTTP(S) links are reported only (unverified).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

$docs = Join-Path $root "creator-docs"
$linkRe = [regex]'\[[^\]]*\]\(([^)]+)\)'
$broken = [System.Collections.Generic.List[object]]::new()
$httpCount = 0

Get-ChildItem -Path $docs -Filter *.md -Recurse -File | ForEach-Object {
    $md = $_
    $relMd = $md.FullName.Substring($root.Length + 1).Replace("\", "/")
    $lineNo = 0
    Get-Content -LiteralPath $md.FullName -Encoding UTF8 | ForEach-Object {
        $lineNo++
        foreach ($m in $linkRe.Matches($_)) {
            $raw = $m.Groups[1].Value.Trim()
            if (-not $raw -or $raw.StartsWith("#")) { continue }
            if ($raw -match '^(https?://)') {
                $script:httpCount++
                continue
            }
            if ($raw.StartsWith("mailto:")) { continue }
            $pathPart = ($raw -split '\s')[0]
            if ($pathPart -match '#') { $pathPart = $pathPart.Split('#')[0] }
            if (-not $pathPart) { continue }
            $target = [System.IO.Path]::GetFullPath((Join-Path $md.DirectoryName $pathPart))
            if (-not $target.StartsWith($root, [StringComparison]::OrdinalIgnoreCase)) {
                $broken.Add([pscustomobject]@{ File = $relMd; Line = $lineNo; Link = $raw; Reason = "escapes repo root" })
                continue
            }
            if (-not (Test-Path -LiteralPath $target)) {
                $relTarget = $target.Substring($root.Length + 1)
                $broken.Add([pscustomobject]@{ File = $relMd; Line = $lineNo; Link = $raw; Reason = "missing: $relTarget" })
            }
        }
    }
}

Write-Host "HTTP(S) links seen: $httpCount (not verified for reachability)"
if ($broken.Count -gt 0) {
    Write-Host "Broken relative links:" -ForegroundColor Red
    $broken | ForEach-Object { Write-Host "  $($_.File):$($_.Line): ($($_.Link)) $($_.Reason)" }
    exit 1
}
Write-Host "OK: creator-docs link check passed"
exit 0
