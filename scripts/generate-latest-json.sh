#!/bin/bash

# Generate latest.json for Tauri updater
VERSION=${1:-$(git describe --tags --abbrev=0 | sed 's/^v//')}
RELEASE_URL="https://github.com/Zbuter/image-filter/releases/download/v${VERSION}"

cat > latest.json << ENDJSON
{
  "version": "v${VERSION}",
  "notes": $(awk "/^## \[${VERSION}\]/{flag=1;next}/^## \[/{flag=0}flag" CHANGELOG.md | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))"),
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "darwin-universal": {
      "signature": "",
      "url": "${RELEASE_URL}/Image.Filter_${VERSION}_universal.dmg"
    },
    "windows-x86_64": {
      "signature": "",
      "url": "${RELEASE_URL}/Image.Filter_${VERSION}_x64-setup.exe"
    }
  }
}
ENDJSON

echo "Generated latest.json for v${VERSION}"
cat latest.json
