# Longitudinal.dev

The repo is in an early public state—expect rough edges while the launch is prepared.

Additional helpers live under `scripts/` (see `scripts/README.md` for details on the `build/`, `deploy/`, and `dev/` subfolders).

## Git Workflow

This project uses **Git Flow** for development:

- **`main`** - Production branch. Pushes automatically deploy to GitHub Pages & Cloudflare.
- **`dev`** - Long-running integration branch. Push here for cloud backup without deploying.
- **Feature branches** - Short-lived branches for specific changes.

### Daily Workflow

**Start new work:**
```bash
git checkout dev                    # Switch to dev
git checkout -b feature/my-change   # Create feature branch from dev
```

**Work and save:**
```bash
# Make changes, test locally with: make serve
git commit -am "Description"        # Commit to feature branch
```

**Integrate to dev:**
```bash
git checkout dev                    # Switch to dev
git merge feature/my-change         # Merge feature
git branch -d feature/my-change     # Delete feature branch
git push origin dev                 # Backup to GitHub (doesn't deploy)
```

**Deploy to production:**
```bash
git checkout main                   # Switch to main
git merge dev                       # Merge all changes from dev
git push                            # Deploy live
```

### Branch Naming Conventions

- `feature/` - New functionality (e.g., `feature/add-search`)
- `content/` - Tutorial/content updates (e.g., `content/update-lgcm`)
- `fix/` - Bug fixes (e.g., `fix/mobile-layout`)
- `experiment/` - Trying new ideas (e.g., `experiment/redesign`)

### Why This Workflow?

- **`dev` branch** allows accumulating multiple changes before deploying
- **Cloud backup** without triggering live deployment
- **`main` stays stable** - always matches production site
- **Feature branches** keep work organized and isolated

## Documentation

Lightweight notes live under `docs/`:

- `docs/TESTING.md` – how to run the preflight script (`CHECK_SSG=true ./scripts/dev/check.sh`).
