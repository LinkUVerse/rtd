---
name: Move IDE Issue
about: Create a new report for issues encountered running Move IDE
title: Move IDE issue report
labels: move-ide
---

## IDE Setup

Describe your setup:
* OS: <specify OS version>
* IDE/editor: <specify IDE/editor version>
* Move analyzer version: <specify `move-analyzer` version>

When using a Move VS Code extension, provide its publisher, version, and the **Move Client** output. To access the output, select **View** -> **Output** from the main menu and open the appropriate tab.

Paste the actual output from your installation, with any secrets removed.

When using a different editor (e.g., Vim, Emacs), provide its version and version of `move-analyzer` binary your editor is using. You can get `move-analyzer` version by running the following command:

``` shell
move-analyzer --version
```

## Steps to Reproduce Issue

Provide the concrete steps needed to reproduce the issue. The more detail you provide, the better chance the problem can be addressed. If the issue is not reproducible, skip this step and proceed to the following ones. When providing code in the reproduction steps, use the smallest buildable example that demonstrates the issue, removing any extraneous details.

e.g.
1. Clone repository <repository>
1. Load file <file>.
2. Hover over language construct <construct> on line <line> in column <column>


## Expected Result

Specify what outcome you expected should have resulted, but didn't.

e.g.
Expected some on-hover information to appear when hovering over <construct> on line <line> in column <column>

## Actual Result

Specify what the actual unexpected outcome was.

e.g.
No on-hover information was displayed when hovering over <construct> on line <line> in column <column>

## Editor Logs

Upon encountering an issue, capture `move-analyzer` logs that may help diagnose it.

When using the Move VS Code extension, provide the content of the **Move** tab. To access this data, select **View** -> **Output** from the main menu and open the appropriate tab from the drop down menu. Paste the actual log from your environment, with any secrets removed:



When using a different editor, capture error output of the `move-analyzer` binary. Consult your editor's documentation to discover how to access this data.
