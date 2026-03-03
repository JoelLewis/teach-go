#!/usr/bin/env bash
# Download classical Go problem collections (public-domain SGF files).
# Output: src-tauri/src/problems/*.sgf
# Run once, then commit the SGF files to the repo.

set -euo pipefail

PROBLEMS_DIR="$(dirname "$0")/../src-tauri/src/problems"
mkdir -p "$PROBLEMS_DIR"
cd "$PROBLEMS_DIR"

echo "=== Fetching classical Go problem collections ==="

# Gokyo Shumyo (Qi Jing Zhong Miao) — ~85 problems with solutions
echo "[1/5] Gokyo Shumyo..."
curl -fsSL "https://dl.u-go.net/problems/qjzm-a.sgf.gz" | gunzip > gokyo-shumyo.sgf
echo "  -> $(grep -c '(;' gokyo-shumyo.sgf || echo '?') game trees"

# Xuanxuan Qijing — 347 problems
echo "[2/5] Xuanxuan Qijing..."
curl -fsSL "https://dl.u-go.net/problems/xxqj.sgf.gz" | gunzip > xuanxuan-qijing.sgf
echo "  -> $(grep -c '(;' xuanxuan-qijing.sgf || echo '?') game trees"

# Guanzipu — ~1500 problems in 3 parts
echo "[3/5] Guanzipu part 1..."
curl -fsSL "https://dl.u-go.net/problems/gzp1.sgf.gz" | gunzip > guanzipu-1.sgf
echo "  -> $(grep -c '(;' guanzipu-1.sgf || echo '?') game trees"

echo "[4/5] Guanzipu part 2..."
curl -fsSL "https://dl.u-go.net/problems/gzp2.sgf.gz" | gunzip > guanzipu-2.sgf
echo "  -> $(grep -c '(;' guanzipu-2.sgf || echo '?') game trees"

echo "[5/5] Guanzipu part 3..."
curl -fsSL "https://dl.u-go.net/problems/gzp3.sgf.gz" | gunzip > guanzipu-3.sgf
echo "  -> $(grep -c '(;' guanzipu-3.sgf || echo '?') game trees"

echo ""
echo "=== Done ==="
echo "Files saved to: $PROBLEMS_DIR"
ls -lh *.sgf
