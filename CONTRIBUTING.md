# Contributing to Hypr-XDisplay

Thank you for your interest in contributing to Hypr-XDisplay! This project aims to provide a modern, modular display and network streaming manager for Hyprland (Wayland) compositors, supporting both Arch Linux (and derivatives like Athena OS) and Debian/Ubuntu systems.

---

## How to Contribute

### 1. **Reporting Issues**
- Search [existing issues](https://github.com/Kyle6012/hypr-xdisplay/issues) before opening a new one.
- Include your OS (Arch, Athena OS, Debian, Ubuntu, etc.), Hyprland version, and relevant logs or screenshots.
- For installation issues, include the output of `uname -a`, `cat /etc/os-release`, and any error messages.

### 2. **Requesting Features**
- Open a [feature request issue](https://github.com/Kyle6012/hypr-xdisplay/issues/new?template=feature_request.md).
- Describe your use case and why it would benefit Hypr-XDisplay users.

### 3. **Submitting Pull Requests**
- Fork the repository and create a new branch for your feature or fix.
- Follow the code style and structure of the existing codebase (Rust 2021, modular, clear comments).
- Test your changes on at least one supported OS (Arch/Athena OS or Debian/Ubuntu) with Hyprland.
- If your change affects the UI, test on both X11 fallback and Wayland/Hyprland if possible.
- Run `cargo fmt` and `cargo clippy` before submitting.
- Reference related issues in your PR description.

### 4. **Code Style**
- Use Rust 2021 idioms and formatting (`cargo fmt`).
- Prefer modular, readable code. Add comments for complex logic.
- UI code should use GTK4 + Libadwaita best practices.
- Keep platform-specific code (e.g., Arch vs. Ubuntu) clearly separated and documented.

### 5. **Testing**
- Manual testing is required for UI and protocol features.
- Add or update unit tests for backend logic where possible.
- If you add a new protocol or integration, document how to test it in the PR.

### 6. **Communication**
- Be respectful and constructive in issues, PRs, and discussions.
- If you’re unsure about a design or feature, open a discussion or draft PR first.

---

## Project Scope
- **Supported OS:** Arch Linux, Athena OS, Debian, Ubuntu (with Hyprland)
- **Supported Compositor:** Hyprland (Wayland, wlroots-based)
- **Not supported:** X11-only, GNOME, KDE, or non-wlroots compositors

---

## Getting Help
- For quick questions, open a GitHub Discussion or use the issue tracker.
- For installation or packaging help, include your OS, package manager, and error output.

---

Thank you for helping make Hypr-XDisplay better for the Wayland/Hyprland community! 