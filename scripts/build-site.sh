#!/bin/bash
# Assembles the site Cloudflare serves into dist/site (not committed): the website (apps/wfhub-web, with its images under /public) and the API data
# (apps/wfhub-api/data, at /data). Cloudflare takes one assets folder, hence this copy.
set -euo pipefail
cd "$(dirname "$0")/.."

rm -rf dist/site
mkdir -p dist/site
cp -R apps/wfhub-web/. dist/site/
mkdir dist/site/data
cp -R apps/wfhub-api/data/. dist/site/data/
find dist/site -name .DS_Store -delete
echo "dist/site: $(find dist/site -type f | wc -l | tr -d ' ') files"
