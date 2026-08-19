# Prints the PE subsystem of an exe: 2 = Windows GUI (no console), 3 = Console.
param([string]$Path)
$fs = [System.IO.File]::OpenRead($Path)
$br = New-Object System.IO.BinaryReader($fs)
$fs.Seek(0x3C, 'Begin') | Out-Null
$peOff = $br.ReadInt32()
$fs.Seek($peOff + 0x5C, 'Begin') | Out-Null
$sub = $br.ReadUInt16()
$fs.Close()
Write-Output ("subsystem=" + $sub + ($(if ($sub -eq 2) { " (GUI)" } elseif ($sub -eq 3) { " (Console)" } else { " (?)" })))
