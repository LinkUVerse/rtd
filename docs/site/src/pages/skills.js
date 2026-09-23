// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import React from "react";
import Layout from "@theme/Layout";
import Link from "@docusaurus/Link";

export default function Skills() {
  return (
    <Layout
      title="RTD Agent Skills status"
      description="Availability of RTD-specific agent skills."
    >
      <main style={{ maxWidth: "760px", margin: "3rem auto", padding: "0 1rem" }}>
        <h1>RTD Agent Skills</h1>
        <p>
          A public RTD skills repository and installable skill collection have
          not been verified for this fork. Do not install inherited Sui skill
          instructions as though they target an RTD network.
        </p>
        <p>
          Inspect the checked-in <Link to="/references/rtd-sdks">RTD SDKs</Link>
          {" "}and the source revision for the chain you operate before creating
          project-specific instructions.
        </p>
      </main>
    </Layout>
  );
}
