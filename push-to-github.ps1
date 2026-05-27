# SVL Stardew Mod Manager - GitHub 推送脚本
# 仓库: https://github.com/fantuan9234/svl-stardew-mod-manager

$repoUrl = "https://github.com/fantuan9234/svl-stardew-mod-manager.git"
$projectPath = "d:\stardew mod mannager\stardew-mod-manager"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  SVL GitHub 推送脚本" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 检查 Git 是否安装
$gitPath = Get-Command git -ErrorAction SilentlyContinue
if (-not $gitPath) {
    Write-Host "Git 未安装，正在下载 MinGit 便携版..." -ForegroundColor Yellow
    
    $minGitUrl = "https://github.com/git-for-windows/git/releases/download/v2.49.0.windows.1/MinGit-2.49.0-64-bit.zip"
    $minGitZip = "$env:TEMP\MinGit.zip"
    $minGitPath = "$env:LOCALAPPDATA\MinGit"
    
    try {
        Invoke-WebRequest -Uri $minGitUrl -OutFile $minGitZip -TimeoutSec 300
        Expand-Archive -Path $minGitZip -DestinationPath $minGitPath -Force
        $env:PATH = "$minGitPath\cmd;$env:PATH"
        Write-Host "MinGit 安装完成!" -ForegroundColor Green
    } catch {
        Write-Host "下载失败，请手动安装 Git: https://git-scm.com/download/win" -ForegroundColor Red
        Read-Host "按 Enter 退出"
        exit 1
    }
} else {
    Write-Host "Git 已安装: $(git --version)" -ForegroundColor Green
}

Set-Location $projectPath

# 检查是否已有 Git 仓库
if (-not (Test-Path ".git")) {
    Write-Host "初始化 Git 仓库..." -ForegroundColor Yellow
    git init
    git remote add origin $repoUrl
} else {
    Write-Host "Git 仓库已存在" -ForegroundColor Green
    # 检查远程仓库
    $remotes = git remote -v 2>$null
    if (-not $remotes) {
        git remote add origin $repoUrl
    }
}

# 配置 Git 用户信息（如果未配置）
$userName = git config user.name 2>$null
$userEmail = git config user.email 2>$null

if (-not $userName) {
    git config user.name "SVL Developer"
}
if (-not $userEmail) {
    git config user.email "dev@svl.app"
}

Write-Host ""
Write-Host "当前 Git 状态:" -ForegroundColor Cyan
Write-Host "----------------------------------------"
git status
Write-Host "----------------------------------------"

Write-Host ""
Write-Host "准备添加文件并提交..." -ForegroundColor Yellow

# 添加所有文件
git add .

# 提交
git commit -m "Initial commit: SVL Stardew Mod Manager

- Tauri + React + Rust desktop application
- PHP backend with admin panel
- Official website with Stardew Valley theme
- Version management, visitor stats, feedback system
- API endpoints for client integration"

Write-Host ""
Write-Host "推送到 GitHub..." -ForegroundColor Yellow

# 推送到 main 分支
git branch -M main
git push -u origin main

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "  推送成功!" -ForegroundColor Green
    Write-Host "  仓库地址: $repoUrl" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "  推送失败，可能需要输入 GitHub 凭据" -ForegroundColor Red
    Write-Host "  请尝试使用 GitHub Token 进行认证" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
}

Write-Host ""
Read-Host "按 Enter 退出"
