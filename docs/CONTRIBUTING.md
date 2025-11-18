# Contributing to Longitudinal.dev

Thank you for your interest in contributing! This guide covers how to contribute content, report issues, and work with the codebase.

## Ways to Contribute

### 1. Suggest Improvements to Tutorials

The easiest way to contribute is directly from the website:

1. Visit any tutorial page on [longitudinal.dev](https://longitudinal.dev)
2. Click the **"Suggest changes"** button in the right sidebar
3. Fill out the form with your suggestions
4. Submit! Your feedback will create a GitHub issue for review

### 2. Report Issues or Request Features

Found a bug or have an idea? [Open an issue](https://github.com/beginDevSci/longitudinal-dev/issues) on GitHub:

- **Bug reports**: Describe what happened vs. what you expected
- **Feature requests**: Explain the use case and benefit
- **Content gaps**: Suggest topics or tutorials you'd like to see

### 3. Join the Discussion

Connect with the community:

- **GitHub Discussions**: [github.com/beginDevSci/longitudinal-dev/discussions](https://github.com/beginDevSci/longitudinal-dev/discussions)
- **Discord**: [discord.gg/WXMv25rf](https://discord.gg/WXMv25rf)

Share ideas, ask questions, or help others!

### 4. Submit Code or Content

Ready to contribute directly? Here's how to work with the repository.

---

## Development Workflow

### Prerequisites

- **Rust** (latest stable)
- **Node.js** 18+
- **Git**
- **wasm-pack** (for building WebAssembly)

### Getting Started

1. **Fork and clone:**
   ```bash
   git clone https://github.com/YOUR-USERNAME/longitudinal-dev.git
   cd longitudinal-dev
   ```

2. **Set up Git Flow branches:**
   ```bash
   git checkout dev  # Work from dev branch, not main
   ```

3. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Making Changes

#### Adding or Editing Tutorials

Tutorial content lives in `content/tutorials/` as Markdown files. Each tutorial follows a structured format:

- **Overview**: Introduction and goals
- **Data Access**: Where to get the data
- **Data Preparation**: Setup and cleaning
- **Statistical Analysis**: Core analysis code
- **Discussion**: Results interpretation
- **Additional Resources**: References and links

See existing tutorials for examples.

#### Building Locally

```bash
# Build the site
make ssg

# Serve locally on http://localhost:8000
cd dist && python3 -m http.server 8000
```

### Git Flow Workflow

This project uses **Git Flow** for development:

- **`main`** - Production branch. Merges to main trigger deployment to GitHub Pages.
- **`dev`** - Integration branch. Use this for development work.
- **Feature branches** - Created from `dev` for specific changes.

#### Daily Workflow

**1. Start new work:**
```bash
git checkout dev
git pull origin dev
git checkout -b feature/my-change
```

**2. Make changes and commit:**
```bash
# Make your edits
git add .
git commit -m "Brief description of changes"
```

**3. Push your feature branch:**
```bash
git push origin feature/my-change
```

**4. Create a Pull Request:**
- Go to GitHub and create a PR from your feature branch to `dev`
- Describe your changes and why they're useful
- Wait for review and feedback

**5. After PR is merged:**
```bash
git checkout dev
git pull origin dev
git branch -d feature/my-change  # Delete local feature branch
```

### Branch Naming Conventions

- `feature/` - New functionality (e.g., `feature/add-search`)
- `content/` - Tutorial/content updates (e.g., `content/update-lgcm-tutorial`)
- `fix/` - Bug fixes (e.g., `fix/mobile-layout`)
- `docs/` - Documentation changes (e.g., `docs/update-contributing`)

---

## Code Guidelines

### Rust Code

- Follow standard Rust formatting (`cargo fmt`)
- Run clippy before committing (`cargo clippy`)
- Write clear comments for complex logic

### Markdown Content

- Use clear, concise language
- Include code examples with syntax highlighting
- Test all code snippets before submitting
- Add references where appropriate

### Commit Messages

Write clear, descriptive commit messages:

```
Good: "Add missing data preparation step to GLMM tutorial"
Good: "Fix broken link in resources section"
Bad: "updates"
Bad: "fix stuff"
```

---

## Testing

Before submitting a PR:

1. **Build successfully**: `make ssg` should complete without errors
2. **Check links**: Verify all internal and external links work
3. **Test locally**: Browse the generated site at `http://localhost:8000`
4. **Verify examples**: Run any code examples to ensure they work

---

## Questions?

- Check existing [issues](https://github.com/beginDevSci/longitudinal-dev/issues)
- Ask in [Discussions](https://github.com/beginDevSci/longitudinal-dev/discussions)
- Join our [Discord](https://discord.gg/WXMv25rf)
- Email: support@longitudinal.dev

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

Thank you for helping make Longitudinal.dev better! 🎉
