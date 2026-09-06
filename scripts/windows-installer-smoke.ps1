param([Parameter(Mandatory = $true)][string]$BundleRoot)

$ErrorActionPreference = 'Stop'
if (-not $env:CI -or -not $env:RUNNER_TEMP) {
    throw 'This smoke test is restricted to an isolated CI runner.'
}
$bundle = (Resolve-Path $BundleRoot).Path
$installDir = Join-Path $env:RUNNER_TEMP ('QuickLAN-smoke-' + [guid]::NewGuid().ToString('N'))
$installers = @(Get-ChildItem (Join-Path $bundle 'nsis') -Filter '*.exe' -File)
if ($installers.Count -ne 1) { throw 'Expected exactly one NSIS installer.' }

function NetworkSnapshot {
    $routes = Get-NetRoute | Sort-Object InterfaceIndex, DestinationPrefix, NextHop |
        Select-Object InterfaceIndex, DestinationPrefix, NextHop, RouteMetric
    $dns = Get-DnsClientServerAddress | Sort-Object InterfaceIndex, AddressFamily |
        Select-Object InterfaceIndex, AddressFamily, ServerAddresses
    return @{ routes = $routes; dns = $dns } | ConvertTo-Json -Depth 5 -Compress
}
function RunBounded([string]$File, [string[]]$Arguments, [int]$Seconds = 120) {
    $child = Start-Process -FilePath $File -ArgumentList $Arguments -PassThru
    if (-not $child.WaitForExit($Seconds * 1000)) {
        $child.Kill()
        throw 'Installer operation timed out.'
    }
    if ($child.ExitCode -ne 0) { throw "Installer operation failed with exit $($child.ExitCode)." }
}

$before = NetworkSnapshot
$app = $null
$installed = $false
$results = [ordered]@{
    platform = 'Windows x64 GitHub-hosted runner'
    installed = $false
    native_window_opened = $false
    graceful_exit = $false
    uninstalled = $false
    route_and_dns_unchanged = $false
    limitations = @('Engineering preview only; no system VPN', 'Hosted runner does not verify interactive UAC or ordinary-user privileges', 'Window presence does not prove every native IPC workflow', 'No cross-device connectivity evidence')
}
try {
    RunBounded $installers[0].FullName @('/S', "/D=$installDir")
    $exe = Join-Path $installDir 'quicklan.exe'
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Installed application is missing.' }
    $installed = $true
    $results.installed = $true
    $app = Start-Process -FilePath $exe -PassThru
    $deadline = (Get-Date).AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 500
        $app.Refresh()
        if ($app.HasExited) { throw 'Native application exited before opening its window.' }
    } until ($app.MainWindowHandle -ne 0 -or (Get-Date) -ge $deadline)
    if ($app.MainWindowHandle -eq 0 -or $app.MainWindowTitle -ne 'QuickLAN') {
        throw 'The expected QuickLAN native window did not open.'
    }
    $results.native_window_opened = $true
    if (-not $app.CloseMainWindow() -or -not $app.WaitForExit(15000)) {
        throw 'The application did not exit through normal window close.' }
    if ($app.ExitCode -ne 0) { throw 'The application exited with an error.' }
    $results.graceful_exit = $true
    $uninstallers = @(Get-ChildItem $installDir -Filter '*uninstall*.exe' -File)
    if ($uninstallers.Count -ne 1) { throw 'Expected exactly one local uninstaller.' }
    RunBounded $uninstallers[0].FullName @('/S')
    $deadline = (Get-Date).AddSeconds(30)
    while ((Test-Path -LiteralPath $exe) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 500 }
    if (Test-Path -LiteralPath $exe) { throw 'Uninstall left the application executable behind.' }
    $results.uninstalled = $true
    $results.route_and_dns_unchanged = $before -eq (NetworkSnapshot)
    if (-not $results.route_and_dns_unchanged) { throw 'Route or DNS state changed during the installer smoke test.' }
} finally {
    if ($app -and -not $app.HasExited) { $app.Kill(); $app.WaitForExit() }
    $results | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 (Join-Path $bundle 'windows-smoke.json')
    # Only clean this run's unique directory. Never remove user or shared driver data.
    if (-not $installed -and (Test-Path -LiteralPath $installDir)) {
        Remove-Item -LiteralPath $installDir -Recurse -Force
    }
}
Write-Output 'Windows install/window/exit/uninstall and route/DNS smoke checks passed.'
