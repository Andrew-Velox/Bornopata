# Adds the Bornopata keyboard to the language list of the user who ran the installer, so nobody has to find
# Settings > Time & language > Language & region > Add a keyboard. Bangla (Bangladesh) is added too when it is missing.
# The installer runs this as that user (the list is per user) and never as the administrator. A failure only means the
# keyboard has to be added by hand; it is written to the log and the installation goes on.
$ErrorActionPreference = 'Stop'
$tag = 'bn-BD'
# 0845 is Bangla (Bangladesh); then the text service and its profile, the same ids as apps\windows\tsf\src\com\mod.rs.
$tip = '0845:{1363C4BB-A7D0-4054-AD76-D5A3C0B681BA}{349D7B39-B963-47DE-B270-DE1E2548FA61}'
$log = Join-Path $env:LOCALAPPDATA 'Bornopata\logs\add-keyboard.log'

function Write-Log($text) {
    New-Item -ItemType Directory -Force -Path (Split-Path $log) | Out-Null
    Add-Content -Path $log -Value ('{0:u} {1}' -f (Get-Date), $text)
}

try {
    $list = Get-WinUserLanguageList
    $language = $list | Where-Object { $_.LanguageTag -eq $tag } | Select-Object -First 1
    if (-not $language) {
        $language = (New-WinUserLanguageList $tag)[0]
        $list.Add($language)
    }
    if ($language.InputMethodTips -notcontains $tip) {
        $language.InputMethodTips.Add($tip)
    }
    Set-WinUserLanguageList $list -Force
    Write-Log 'the Bornopata keyboard is in the language list'
} catch {
    Write-Log ('could not add the keyboard: ' + $_)
}
