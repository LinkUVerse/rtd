// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
import styles from "./index.module.css";

export default function FooterLayout({ logo, copyright }) {
  return (
    <footer className={styles.footer}>
      <div className={styles.footerWrap}>
        <div className={styles.footerLogo}>{logo}</div>
        <div className={styles.footerContent}>
          <div className={styles.footerCopy}>{copyright}</div>
        </div>
      </div>
    </footer>
  );
}
