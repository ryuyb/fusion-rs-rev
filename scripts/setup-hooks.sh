#!/bin/bash
# Git Hooks 配置脚本
# 用于在新设备上快速配置 git hooks

set -e

echo "🔧 配置 Git Hooks..."

# 配置 git 使用项目中的 .githooks 目录
git config core.hooksPath .githooks

echo "✅ Git Hooks 配置完成!"
echo ""
echo "已启用的 hooks:"
echo "  - pre-commit: 自动运行 cargo fmt"
echo ""
echo "如需禁用,运行: git config --unset core.hooksPath"
