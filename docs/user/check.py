#!/usr/bin/env python3
"""Checks the user docs: every page has frontmatter, every group has an index, every relative link and image exists."""
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
problems = []
for page in sorted(root.rglob('*.md')):
    if page.name == 'README.md':
        continue
    text = page.read_text(encoding='utf-8')
    match = re.match(r'---\ntitle: (.+)\norder: (\d+)\ndescription: (.+)\n---\n', text)
    where = page.relative_to(root)
    if not match:
        problems.append(f'{where}: needs frontmatter with title, order and description')
    elif len(match.group(3)) > 200:
        problems.append(f'{where}: the description is longer than 200 characters')
    for target in re.findall(r'!?\[[^\]]*\]\(([^)#\s]+)(?:#[^)]*)?\)', text):
        if not target.startswith(('http://', 'https://', 'mailto:')) and not (page.parent / target).exists():
            problems.append(f'{where}: broken link {target}')
for folder in sorted(p for p in root.rglob('*') if p.is_dir() and p.name != 'images'):
    if not (folder / 'index.md').is_file():
        problems.append(f'{folder.relative_to(root)}: a group needs an index.md')
print('\n'.join(problems) if problems else 'user docs ok')
sys.exit(1 if problems else 0)
