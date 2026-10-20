# User documentation

The Markdown in this directory is written for **users**. The website (a separate repository) pulls the whole directory at build time and renders it by folder structure. Everything else under `docs/` is for developers and is not rendered. This README is a maintenance note and is not rendered either.

## Structure

```text
docs/user/
├── index.md              # documentation home
├── getting-started/      # install, first words, the keys
├── input/                # typing Bangla, the candidate list, phrases, date and time, emoji, English mode
├── learning/             # what it learns, next-word suggestions, English glosses
├── settings/             # the config file, themes, the dictionary
└── help/                 # files and privacy, troubleshooting, uninstall, feedback
```

- A folder is a group. Its `index.md` gives the group title and order; the other `.md` files are its pages.
- Every page starts with frontmatter:

  ```yaml
  ---
  title: Install            # page title and sidebar name
  order: 1                  # order inside the group, small first
  description: One sentence  # search result and share card, about 150 characters
  ---
  ```
- Images live next to the page that uses them, in an `images/` folder, and are referenced relatively.

## Rules

- Write about what a user sees and does, not how the code works; that belongs in `docs/design` and `docs/notes`.
- Every claim must be true of the code as it is. When behavior, keys, settings or files change, change the page in the same commit (see `docs/contributing.md`).
- Examples of typing are checked against the engine: `cargo run -p borno-cli -- --no-dict <roman>` shows the direct conversion.
- Link between pages with relative `.md` paths.
