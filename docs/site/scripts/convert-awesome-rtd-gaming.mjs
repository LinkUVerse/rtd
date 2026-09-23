// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

// Get __dirname equivalent in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Paths (adjusted for new location)
const readmePath = path.join(
  __dirname,
  "../../subtree/awesome-rtd-gaming/README.md",
);
const readmeTargetPath = path.join(
  __dirname,
  "../../content/references/awesome-rtd-gaming.mdx",
);

// Process the content for the awesome-rtd-gaming README structure:
// - Remove everything up to and including the # Contents section
// - Keep all # section headings and their tables intact
function processContent(content) {
  // Remove everything from the start up to and including the Contents section + its trailing ---
  // The Contents section ends before the first # I. heading
  // Strip everything before the first # [Roman numeral] heading
  const firstSectionMatch = content.search(/^# [IVX]+\./m);
  let processedContent;
  if (firstSectionMatch !== -1) {
    processedContent = content.slice(firstSectionMatch);
  } else {
    console.log("Warning: Could not find first section heading, using full content");
    processedContent = content;
  }

  // Convert # headings (h1) to ## headings (h2) for proper doc hierarchy
  processedContent = processedContent.replace(/^# /gm, "## ");

  return processedContent.trim();
}

// Convert README.md
console.log("Reading README file:", readmePath);
const readmeContent = fs.readFileSync(readmePath, "utf8");
const processedReadmeContent = processContent(readmeContent);

const readmeMdxContent = `---
title: Awesome RTD Gaming status
description: Status of the inherited gaming catalog in this RTD fork.
keywords:
  - gaming
  - integrations
  - upstream
questions:
  - Are inherited gaming integrations available on RTD?
answer: >-
  The inherited gaming catalog has not been validated for RTD.
---

:::info

The checked-in source snapshot under docs/subtree/awesome-rtd-gaming is historical material. Its entries are not verified RTD integrations.

:::

${processedReadmeContent}`;

// Ensure target directories exist
const readmeTargetDir = path.dirname(readmeTargetPath);
if (!fs.existsSync(readmeTargetDir)) {
  fs.mkdirSync(readmeTargetDir, { recursive: true });
}

// Write the main README MDX file
console.log("Writing README target file:", readmeTargetPath);
fs.writeFileSync(readmeTargetPath, readmeMdxContent, "utf8");

console.log("✅ Successfully converted README.md");
