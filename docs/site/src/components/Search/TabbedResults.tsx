// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import React from "react";

export default function TabbedResults({
  activeTab,
  onChange,
  tabs,
  showTooltips = true,
}) {
  const rtdtooltip = "Search results from the official Rtd Docs";
  const rtdnstooltip = "Search results from Rtd Name Service";
  const movetooltip = "Search results from The Move Book";
  const dapptooltip = "Search results from the Rtd ecosystem SDKs";
  const walrustooltip =
    "Search results from the Walrus decentralized storage platform";
  return (
    <div className="mb-4 flex justify-start border-2 border-solid border-white rounded-t-lg dark:bg-black dark:border-rtd-black border-b-rtd-gray-50 dark:border-b-rtd-gray-80">
      {tabs.map(({ label, indexName, count }) => (
        <div className="relative group inline-block" key={indexName}>
          <button
            className={`mr-4 flex items-center font-semibold text-sm lg:text-md xl:text-lg bg-white dark:bg-rtd-black cursor-pointer dark:text-rtd-gray-45 ${activeTab === indexName ? "text-rtd-disabled/100 font-bold border-2 border-solid border-transparent border-b-rtd-blue-dark dark:border-b-rtd-blue" : "border-transparent text-rtd-disabled/70"}`}
            onClick={() => onChange(indexName)}
          >
            {label}{" "}
            <span
              className={`dark:text-rtd-gray-90 text-xs rounded-full ml-1 py-1 px-2 border border-solid ${activeTab === indexName ? "dark:!text-rtd-gray-45 bg-transparent border-rtd-gray-3s dark:border-rtd-gray-50" : "bg-rtd-gray-45 border-transparent"}`}
            >
              {count}
            </span>
          </button>
          {showTooltips && (
            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-max max-w-xs px-2 py-1 text-sm text-white bg-gray-800 rounded tooltip-delay">
              {label === "Rtd"
                ? rtdtooltip
                : label === "RtdNS"
                  ? rtdnstooltip
                  : label === "The Move Book"
                    ? movetooltip
                    : label === "SDKs"
                      ? dapptooltip
                      : walrustooltip}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
