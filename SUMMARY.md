## Änderungen
- Devcontainer inklusive VSCode/Neovim Defaults, Post-Create Setup und reproduzierbarer Bootstrap.
- Erweiterter `Taskfile.yml` mit Lint/Test/Bench-Flows plus Helper-Skripte (`run_task.sh`, Benchmarks).
- `.githooks` Pre-Commit, `.editorconfig`, `.env.example`, Issue/PR-Templates und Onboarding-Skript.
- Spezifikation, Plan und TODO-Backlog um neue Tooling-Ziele ergänzt; CR-0002 mit Workflow-Doku angelegt.

## Kommandos
- Setup: `task setup`
- Lint: `task lint`
- Tests: `task test`
- Benchmarks: `task bench`

## Nächste Schritte
- CI/Container-Abbilder auf neuen Devcontainer-Stack ausrichten (Follow-up in P7).
- Inhalte von CR-0002 in `docs/DOCUMENTATION.md` konsolidieren, sobald alle Pakete gemerged sind.

## Annahmen
- Contributors verfügen über Bun, Rust und Node oder verwenden den bereitgestellten Devcontainer.
- `bunx` verfügbar für Fallback, falls `task` nicht global installiert ist.
