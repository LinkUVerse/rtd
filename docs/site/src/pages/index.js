// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import React from "react";

import Layout from "@theme/Layout";
import Link from "@docusaurus/Link";
import styles from "./index.module.css";

export default function Home() {
  const developerResources = [
    {
      title: "Getting Started",
      description:
        "Install the Rtd toolchain, set up a wallet, and publish your first Move package.",
      to: "/getting-started",
    },
    {
      title: "Rtd Agent Skills",
      description: "Check the status of RTD-specific agent skills before use.",
      to: "/skills",
    },
    {
      title: "Develop",
      description:
        "Write and upgrade Move packages, work with objects, and query onchain data.",
      to: "/develop",
    },
    {
      title: "Onchain Finance",
      description:
        "Explore Move asset patterns and the status of inherited finance integrations.",
      to: "/onchain-finance",
    },
    {
      title: "Rtd Stack",
      description:
        "Review source capabilities and the deployment status of external services.",
      to: "/rtd-stack",
    },
    {
      title: "References",
      description:
        "Look up the CLI, SDKs, Move framework, and network API references.",
      to: "/references",
    },
  ];

  const useCases = [
    {
      title: "DeepBook",
      description:
        "Review the inherited DeepBook design; no RTD market is deployed.",
      to: "/onchain-finance/deepbook",
    },
    {
      title: "Walrus",
      description:
        "Review upstream storage concepts; RTD integration has not been verified.",
      to: "/rtd-stack/walrus",
    },
    {
      title: "zkLogin",
      description:
        "Review identity features and provider requirements before integration.",
      to: "/rtd-stack/zklogin-integration/zklogin",
    },
    {
      title: "Digital Assets",
      description:
        "Build NFTs and composable digital collectibles with the object model.",
      to: "/onchain-finance/types-of-assets",
    },
  ];

  const nodeOperators = [
    {
      title: "Run a Rtd Full Node",
      description: "Run a full node to sync the network and serve onchain data.",
      to: "/operators/full-node/rtd-full-node",
    },
    {
      title: "Validators",
      description: "Set up and operate a validator to help secure the network.",
      to: "/operators/validator",
    },
    {
      title: "Data Management",
      description:
        "Set up archival storage and indexing services for onchain data.",
      to: "/operators/data-management",
    },
  ];

  const ResourceCard = ({ title, description, to }) => (
    <Link to={to} className={styles.resourceCard}>
      <h3 className={styles.resourceCardTitle}>{title}</h3>
      <p className={styles.resourceCardDesc}>{description}</p>
      <span className={styles.resourceCardArrow} aria-hidden="true">
        →
      </span>
    </Link>
  );

  const Section = ({ heading, items }) => (
    <section className={styles.homeSection}>
      <h2 className={styles.homeSectionHeading}>{heading}</h2>
      <div className="flex flex-row flex-wrap justify-center gap-2">
        {items.map((item) => (
          <ResourceCard key={item.title} {...item} />
        ))}
      </div>
    </section>
  );

  return (
    <>
      <Layout>
        <div
          className="overflow-hidden min-h-screen flex flex-col bg-cover bg-center bg-no-repeat"
          style={{ backgroundColor: "#000000" }}
        >
          <div className="w-full mt-8 mb-4 mx-auto">
            <div className={styles.heroText}>
              <h1 className="h1 center-text text-white">Rtd Documentation</h1>
              <p
                className="center-text"
                style={{
                  color: "#89919F",
                  maxWidth: "720px",
                  margin: "0 auto",
                  fontSize: "1.1rem",
                  lineHeight: "1.6",
                }}
              >
                RTD is a fork of the Sui source code built around Move. These
                guides cover source capabilities and locally operated networks.
                RTD has not launched a public Mainnet; check each integration’s
                deployment status before using its examples.
              </p>
            </div>
          </div>

          <Section heading="Developer Resources" items={developerResources} />
          <Section heading="Use Cases" items={useCases} />
          <Section heading="Node Operators" items={nodeOperators} />
        </div>
      </Layout>
    </>
  );
}
