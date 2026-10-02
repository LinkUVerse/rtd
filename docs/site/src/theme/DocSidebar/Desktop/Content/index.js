// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import React from "react";
import Content from "@theme-original/DocSidebar/Desktop/Content";

export default function ContentWrapper(props) {
  const wrapperRef = React.useRef(null);
  const scrollRef = React.useRef(null);
  const contentRef = React.useRef(null);
  const [showShadow, setShowShadow] = React.useState(false);

  React.useEffect(() => {
    const wrapper = wrapperRef.current;
    if (!wrapper) return;

    const scrollEl = scrollRef.current;
    if (!scrollEl) return;

    const atBottom = (el, pad = 1) =>
      el.scrollTop + el.clientHeight >= el.scrollHeight - pad;

    const update = () => setShowShadow(!atBottom(scrollEl));

    // Initial check
    update();

    // Scroll + resize observers
    const onScroll = () => requestAnimationFrame(update);
    scrollEl.addEventListener("scroll", onScroll, { passive: true });

    const ro = new ResizeObserver(() => update());
    ro.observe(scrollEl);

    // Observe inner content size changes (collapsible sections expanding/collapsing)
    const contentEl = contentRef.current || scrollEl.firstElementChild;
    let contentRO;
    if (contentEl) {
      contentRO = new ResizeObserver(() => update());
      contentRO.observe(contentEl);
    }

    // Also observe content changes inside the scroller (collapsible sections)
    const mo = new MutationObserver(() => update());
    mo.observe(scrollEl, { childList: true, subtree: true, attributes: true });

    return () => {
      scrollEl.removeEventListener("scroll", onScroll);
      ro.disconnect();
      mo.disconnect();
      if (contentRO) contentRO.disconnect();
    };
  }, []);

  return (
    <div className="flex h-full min-h-0 flex-col">
      {/* Scrollable content area */}
      <div ref={wrapperRef} className="relative flex-1 min-h-0">
        <div ref={scrollRef} className="relative h-full overflow-auto">
          <div ref={contentRef}>
            <Content {...props} />
          </div>
        </div>
        {/* Top-edge gradient that appears when there is more content below (nav not at bottom) */}
        <div
          aria-hidden
          className={`${
            showShadow ? "opacity-100" : "opacity-0"
          } absolute inset-x-0 bottom-0 h-4 bg-gradient-to-t from-black/20 to-transparent pointer-events-none transition-opacity duration-200 z-20`}
        />
      </div>

    </div>
  );
}
