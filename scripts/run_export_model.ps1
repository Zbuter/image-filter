# PowerShell wrapper for Windows: clears OPENSSL_CONF before running export script
$env:OPENSSL_CONF = ''

# Try to find the right Python (with open_clip installed)
$pythonCandidates = @(
    'python',
    'python3',
    "$env:LOCALAPPDATA\Programs\Python\Python*\python.exe"
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$scriptPath = Join-Path $scriptDir 'export_clip_model.py'

foreach ($candidate in $pythonCandidates) {
    $resolved = Resolve-Path $candidate -ErrorAction SilentlyContinue
    if ($resolved) {
        foreach ($p in $resolved) {
            $testResult = & $p.Path -c "import open_clip; print('ok')" 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Using Python: $($p.Path)"
                & $p.Path $scriptPath @args
                exit $LASTEXITCODE
            }
        }
    }
}

Write-Host "ERROR: Could not find Python with open_clip installed."
Write-Host "Install dependencies: pip install open-clip-torch onnx torch torchvision numpy"
exit 1
