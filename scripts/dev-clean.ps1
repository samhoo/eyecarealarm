# Kills every process related to the dev session: app exes and vite/CDP port owners.
Get-Process eye-care-alarm -ErrorAction SilentlyContinue | ForEach-Object {
  Write-Output ("kill exe " + $_.Id)
  Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
}
foreach ($port in 1420, 9222) {
  $conns = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue
  foreach ($c in $conns) {
    $proc = Get-Process -Id $c.OwningProcess -ErrorAction SilentlyContinue
    if ($proc -and $proc.ProcessName -ne 'System') {
      Write-Output ("kill :" + $port + " owner " + $proc.ProcessName + " " + $proc.Id)
      Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
  }
}
Start-Sleep -Seconds 1
$left = Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue
Write-Output ("port1420-listening=" + [bool]$left)
$exe = Get-Process eye-care-alarm -ErrorAction SilentlyContinue
Write-Output ("exe-running=" + [bool]$exe)
