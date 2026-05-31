---
name: Bug report
about: Create a report to help us improve
title: ''
labels: bug
assignees: ''

---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. ...
2. ...
3. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Environment**
- Distribution: [e.g. Fedora 44, Ubuntu 24.04]
- Driver version: [e.g. 1.2.0]
- Laptop model: [e.g. ASUS ZenBook UX433FA]

**Logs**
Set `log_level` to `"debug"` in `config.json`, reproduce the issue, then paste the output of:
```bash
journalctl -u asus-touchpad-numpad --since "5 minutes ago"
```
