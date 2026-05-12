$content = Get-Content 'C:\Users\Administrator\.qclaw\workspace\harper-work\quick123-666\harper\harper-core\src\spell\mod.rs' -Raw -Encoding UTF8
$old = 'let mut score = sug.edit_distance as i32 * 10;'
if ($content -match [regex]::Escape($old)) {
    Write-Host 'FOUND'
} else {
    Write-Host 'NOT FOUND'
}