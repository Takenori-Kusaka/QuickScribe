<#
.SYNOPSIS
  QuickScribe のアイドル時 CPU 使用率を Windows 実機／Windows ランナーで計測する（#664 Phase 2）。

.DESCRIPTION
  perf.yml の Linux ジョブ（#664 Phase 1）と同じ定義でアイドル CPU を算出する:

      使用率(%) = (観測窓での消費CPU秒 / 観測窓の経過実時間秒) * 100    ※1コア基準(100% = 1コア占有)

  計測対象はアプリ実体（既定 quickscribe）と、その配下の WebView2 子プロセス
  （msedgewebview2）。Linux 側が quickscribe + WebKit(WebProcess|NetworkProcess) を
  合算しているのと同じ考え方（WebView の常駐コストを取りこぼすと指標として意味を成さない）。

  Linux の perf CI では Windows 限定コード（src-tauri/src/taskbar_widget.rs のタスクバー
  ウィジェット）が動かないため、アイドル CPU の主要因が原理的に現れない。本スクリプトはその穴を
  埋めるためのもの。

.PARAMETER ProcessName
  計測対象のプロセス名（拡張子なし）。既定 "quickscribe"。

.PARAMETER SettleSeconds
  観測を始める前に待つ秒数。起動直後の処理を「アイドル」に混ぜないため。既定 3。

.PARAMETER WindowSeconds
  観測窓の長さ（秒）。長いほどノイズが減る。既定 30。

.PARAMETER Label
  レポートに載せる条件名（例 "widget-on" / "widget-off"）。既定 "default"。

.PARAMETER JsonPath
  指定すると計測結果を JSON でこのパスへ書き出す（CI での突合・差分計算用）。

.EXAMPLE
  # 既に起動しているアプリをそのまま 30 秒観測する
  pwsh -File scripts/perf/measure_idle_cpu.ps1 -Label widget-on

.EXAMPLE
  # 設定でタスクバーウィジェットを OFF にしてから、同じ長さで観測して差分を見る
  pwsh -File scripts/perf/measure_idle_cpu.ps1 -Label widget-off -JsonPath idle-cpu-widget-off.json

.NOTES
  終了コード: 0=計測成功 / 1=対象プロセスが見つからない・観測中に消えた。
#>
[CmdletBinding()]
param(
  [string]$ProcessName = 'quickscribe',
  [int]$SettleSeconds = 3,
  [int]$WindowSeconds = 30,
  [string]$Label = 'default',
  [string]$JsonPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# 対象プロセス群（アプリ実体 + その子孫の WebView2）の PID を集める。
# Get-Process だけでは親子関係が取れないため Win32_Process の ParentProcessId をたどる。
function Get-TargetProcessIds {
  param([string]$Name)

  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name
  $roots = @($all | Where-Object { $_.Name -ieq "$Name.exe" } | Select-Object -ExpandProperty ProcessId)
  if ($roots.Count -eq 0) { return @() }

  # 子孫を幅優先でたどる（WebView2 は quickscribe.exe -> msedgewebview2.exe -> 子群 と枝分かれする）。
  $ids = New-Object 'System.Collections.Generic.HashSet[int]'
  foreach ($r in $roots) { [void]$ids.Add([int]$r) }
  $frontier = @($roots)
  while ($frontier.Count -gt 0) {
    $next = @()
    foreach ($p in $frontier) {
      foreach ($c in ($all | Where-Object { $_.ParentProcessId -eq $p })) {
        if ($ids.Add([int]$c.ProcessId)) { $next += [int]$c.ProcessId }
      }
    }
    $frontier = $next
  }
  return @($ids)
}

# 対象 PID 群の累積 CPU 時間（秒）を **PID ごとに** 返す。
# 合計ではなく PID ごとに持つのは、観測窓の途中で消えた／増えたプロセスを差分計算から
# 除外するため（合計だけだと消滅分が引き算に混ざり使用率が負や過小になる）。
function Get-CpuSecondsByPid {
  param([int[]]$Ids)

  $map = @{}
  foreach ($id in $Ids) {
    try {
      $p = Get-Process -Id $id -ErrorAction Stop
      $map[[int]$id] = $p.TotalProcessorTime.TotalSeconds
    } catch {
      # 既に終了したプロセス。マップに入れない＝両端に揃わないので差分から自動的に外れる。
    }
  }
  return $map
}

$targets = Get-TargetProcessIds -Name $ProcessName
if ($targets.Count -eq 0) {
  Write-Error "計測対象のプロセスが見つかりません: $ProcessName.exe （アプリを起動してから実行してください）"
  exit 1
}

Write-Host "対象プロセス: $($targets.Count) 件 (PID: $($targets -join ', '))"
Write-Host "settle ${SettleSeconds}s -> 観測 ${WindowSeconds}s ..."

Start-Sleep -Seconds $SettleSeconds

$t0 = Get-CpuSecondsByPid -Ids $targets
$sw = [System.Diagnostics.Stopwatch]::StartNew()
Start-Sleep -Seconds $WindowSeconds
$sw.Stop()
$t1 = Get-CpuSecondsByPid -Ids $targets

# 観測窓の両端に存在した PID だけで差分を取る（途中で消えたプロセスを混ぜない）。
$stable = @($t0.Keys | Where-Object { $t1.ContainsKey($_) } | Sort-Object)
if ($stable.Count -eq 0) {
  Write-Error "観測窓の間に対象プロセスが全て消えました。計測できません。"
  exit 1
}
if ($stable.Count -ne $targets.Count) {
  Write-Warning "観測窓の前後でプロセス集合が変化しました（$($targets.Count) -> $($stable.Count)）。両端に存在した $($stable.Count) 件のみで算出します。"
}

$elapsed = $sw.Elapsed.TotalSeconds
$cpuDelta = 0.0
foreach ($id in $stable) { $cpuDelta += ($t1[$id] - $t0[$id]) }
if ($cpuDelta -lt 0) { $cpuDelta = 0 }
$pct = if ($elapsed -gt 0) { ($cpuDelta / $elapsed) * 100.0 } else { 0 }

# プロセス別の内訳。タスクバーウィジェットのタイマーはアプリ実体（quickscribe.exe）で
# 回るため、実体単独の消費を見れば WebView2 と切り分けられる（#662 の切り分けに必要）。
$breakdown = foreach ($id in $stable) {
  $name = try { (Get-Process -Id $id -ErrorAction Stop).ProcessName } catch { 'exited' }
  [pscustomobject]@{
    pid         = $id
    name        = $name
    cpu_seconds = [math]::Round($t1[$id] - $t0[$id], 4)
    cpu_percent = if ($sw.Elapsed.TotalSeconds -gt 0) { [math]::Round((($t1[$id] - $t0[$id]) / $sw.Elapsed.TotalSeconds) * 100.0, 3) } else { 0 }
  }
}

$result = [pscustomobject]@{
  label            = $Label
  process          = $ProcessName
  pids             = $stable
  process_count    = $stable.Count
  per_process      = @($breakdown)
  window_seconds   = [math]::Round($elapsed, 3)
  cpu_seconds      = [math]::Round($cpuDelta, 4)
  idle_cpu_percent = [math]::Round($pct, 3)
  logical_cores    = [Environment]::ProcessorCount
}

Write-Host ""
Write-Host "| 条件 | プロセス数 | 観測窓 | 消費CPU | アイドルCPU (1コア基準) |"
Write-Host "|---|---|---|---|---|"
Write-Host ("| {0} | {1} | {2:N1} s | {3:N3} s | **{4:N3} %** |" -f $result.label, $result.process_count, $result.window_seconds, $result.cpu_seconds, $result.idle_cpu_percent)
Write-Host ""
Write-Host "プロセス別内訳:"
foreach ($b in $breakdown) { Write-Host ("  PID {0,-7} {1,-20} {2,8:N4} s  {3,7:N3} %" -f $b.pid, $b.name, $b.cpu_seconds, $b.cpu_percent) }

if ($JsonPath) {
  $result | ConvertTo-Json -Depth 4 | Out-File -FilePath $JsonPath -Encoding utf8
  Write-Host "JSON: $JsonPath"
}

exit 0
