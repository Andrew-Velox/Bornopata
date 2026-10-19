# Security Policy

## Supported versions

Bornopata has no release yet. Once there is one, only the latest release receives fixes.

## Reporting a vulnerability

**Do not put vulnerability details in public issues.** Use GitHub's private reporting: the repository's Security tab, then "Report a vulnerability".

An input method sees every key you type, so these matter most:

- Leaking what you type: logs, diagnostics or cloud requests that contain input; composing or sending requests inside a password field.
- Secrets: API keys from `.env` or the configuration written to logs, diagnostics or other files.
- Crashes or out-of-bounds access from malformed dictionary files (TSV, `.bnd`), configuration files or cloud responses.

Out of scope: problems that need local administrator access or physical access, and issues on the side of an AI provider you configured yourself.
