// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import React from "react";
import Layout from "@theme/Layout";

import useDocusaurusContext from "@docusaurus/useDocusaurusContext";

export default function JsonRpc() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout title={`Rtd API Reference (Legacy) | ${siteConfig.title}`}>
      <div style={{ maxWidth: "960px", margin: "0 auto", padding: "1rem" }}>
        <div className="legacy-api-banner" role="alert">
          <p className="legacy-api-banner__title">
            Legacy API scheduled for removal
          </p>
          <p>
            RTD has not launched Mainnet. Its future Mainnet will not expose
            JSON-RPC, following the upstream Sui direction. The upstream
            July&nbsp;2026 dates do not describe an RTD event. Use this page
            to identify methods in local deployments that still expose the
            legacy service. Call{" "}
            <a href="/develop/accessing-data/grpc">gRPC</a> or{" "}
            <a href="/develop/accessing-data/graphql/graphql-rpc">
              GraphQL RPC
            </a>{" "}
            instead, and see the{" "}
            <a href="/develop/accessing-data/json-rpc-migration">
              JSON-RPC Migration Guide
            </a>{" "}
            for the method-by-method mapping.
          </p>
        </div>
        <p>
          A public RTD API catalog and network-specific OpenRPC snapshots have
          not been released. Inspect the source specification and query your
          own RTD node for the API enabled in your deployment.
        </p>
      </div>
    </Layout>
  );
}
