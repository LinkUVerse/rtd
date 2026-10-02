// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { formatAddress } from "@linku/rtd/utils";
import { CheckIcon, CopyIcon } from "@radix-ui/react-icons";
import { Button, Flex, Link, Tooltip } from "@radix-ui/themes";
import { useNetworkVariable } from "config";
import { ReactElement, useState } from "react";
import toast from "react-hot-toast";

type Props = {
    id: string;
    size?: "1" | "2" | "3" | "4";
    display?: "none" | "flex" | "inline-flex";
};

/**
 * Renders an Object ID.
 *
 * The ID is represented in a contracted form and links to an explorer when
 * one is configured. It also has a tooltip to show its
 * full value and a button to copy that value to the clipboard.
 *
 * The optional `size` parameter controls how big the ID is, in
 * Radix's size units.
 */
export function IDLink({ id, size, display }: Props): ReactElement {
    const explorer = useNetworkVariable("explorer");
    const explorerLink = explorer(id);
    size = size ?? "1";

    const [copied, setCopied] = useState(false);
    const onClick = async () => {
        await navigator.clipboard.writeText(id);
        setCopied(true);
        setTimeout(() => setCopied(false), 1000);
        toast.success("Copied ID to clipboard!");
    };

    return (
        <Flex align="center" display={display}>
            <Tooltip content={id}>
                {explorerLink ? (
                    <Link href={explorerLink} target="_blank" rel="noreferrer" size={size}>
                        {formatAddress(id)}
                    </Link>
                ) : (
                    <span>{formatAddress(id)}</span>
                )}
            </Tooltip>
            <Tooltip content="Copy Object ID">
                <Button size={size} ml="2" onClick={onClick} variant={copied ? "outline" : "soft"}>
                    {copied ? <CheckIcon /> : <CopyIcon />}
                </Button>
            </Tooltip>
        </Flex>
    );
}
