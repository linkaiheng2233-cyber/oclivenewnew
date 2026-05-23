Get-ChildItem -Path src-tauri/src,src,crates -Recurse -Include *.rs,*.ts,*.vue -ErrorAction SilentlyContinue |
  Select-Object @{N='KB';E={[math]::Round($_.Length/1024,1)}}, FullName |
  Sort-Object KB -Descending |
  Select-Object -First 30 |
  Format-Table -AutoSize