// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
import * as React from "react";
import { useThemeConfig, ErrorCauseBoundary } from "@docusaurus/theme-common";
import {
  splitNavbarItems,
  useNavbarMobileSidebar,
} from "@docusaurus/theme-common/internal";
import NavbarItem from "@theme/NavbarItem";
import ThemeToggle from "@site/src/shared/components/ThemeToggle";
// import SearchBar from "@theme/SearchBar";
import NavbarMobileSidebarToggle from "@theme/Navbar/MobileSidebar/Toggle";
import NavbarLogo from "@theme/Navbar/Logo";
import NavbarSearch from "@theme/Navbar/Search";
import GetStartedLink from "@site/src/components/GetStartedLink";

function useNavbarItems() {
  // TODO temporary casting until ThemeConfig type is improved
  return useThemeConfig().navbar.items;
}

// Safe wrapper: if the provider isn't present, don't crash
function useMobileSidebarSafe() {
  try {
    return useNavbarMobileSidebar();
  } catch {
    // Emulate a "disabled" state so we can skip rendering the toggle safely.
    return { disabled: true, toggle: () => {} };
  }
}

function NavbarItems({ items }) {
  return (
    <div className="navbar-left-items">
      {items.map((item, i) => (
        <ErrorCauseBoundary
          key={i}
          onError={(error) =>
            new Error(
              `A theme navbar item failed to render.
Please double-check the following navbar item (themeConfig.navbar.items) of your Docusaurus config:
${JSON.stringify(item, null, 2)}`,
              { cause: error },
            )
          }
        >
          <NavbarItem {...item} />
        </ErrorCauseBoundary>
      ))}
    </div>
  );
}

function NavbarContentLayout({ left, right }) {
  return (
    <div className="navbar__inner">
      <div className="navbar__items">{left}</div>
      <div className="navbar__items navbar__items--right">{right}</div>
    </div>
  );
}

export default function NavbarContent() {
  const mobileSidebar = useMobileSidebarSafe();
  const items = useNavbarItems();
  const [leftItems, rightItems] = splitNavbarItems(items);
  const searchBarItem = items.find((item) => item.type === "search");

  return (
    <NavbarContentLayout
      left={
        <>
          {!mobileSidebar.disabled && <NavbarMobileSidebarToggle />}
          <NavbarLogo />
          <NavbarItems items={leftItems} />
        </>
      }
      right={
        <div className="navbar-right-controls flex items-center flex-shrink-0">
          <NavbarItems items={rightItems} />
          <ThemeToggle />
          {!searchBarItem && (
            <NavbarSearch>
              <GetStartedLink />
            </NavbarSearch>
          )}
        </div>
      }
    />
  );
}