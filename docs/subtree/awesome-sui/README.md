# Awesome Rtd [![Awesome](https://awesome.re/badge.svg)](https://awesome.re)

<a href="https://rtd.io/"><img alt="Rtd logo" src="media/logo.svg" align="right" width="150" /></a>

> A curated list of _awesome_ developer tools and infrastructure projects within the Rtd ecosystem.

Rtd is the first blockchain built for internet scale, enabling fast, scalable, and low-latency transactions. It's programmable and composable, powered by the Move language, making it easy to build and integrate dApps. Rtd prioritizes developer experience and frictionless user interactions, designed to support next-gen decentralized applications with minimal complexity.

> ⚠️ This warning icon means that the tool may not be functioning correctly at the moment. Please check these tools carefully.

[**Submit your own developer tool here**](CONTRIBUTING.md)

## Contents

- [Move IDEs](#move-ides)
  - [Web IDEs](#web-ides)
  - [Desktop IDEs](#desktop-ides)
  - [IDE Utilities](#ide-utilities)
- [Client SDKs \& Libraries](#client-sdks--libraries)
  - [Client SDKs](#client-sdks)
  - [DeFi SDKs](#defi-sdks)
  - [Client Libraries](#client-libraries)
- [dApp Development](#dapp-development)
  - [dApp Toolkits](#dapp-toolkits)
  - [Smart Contract Toolkits](#smart-contract-toolkits)
- [Indexers \& Data Services](#indexers--data-services)
- [Explorers](#explorers)
- [Oracles](#oracles)
- [Security](#security)
- [AI](#ai)
- [Infrastructure as Code](#infrastructure-as-code)
- [Faucets](#faucets)

## Move IDEs

### Web IDEs

- BitsLab IDE - Online Move code editor that requires no configuration and supports Move code syntax highlighting. Beginner friendly and supports interacting with Rtd.
  - [Homepage](https://www.bitslab.xyz/bitslabide) - [IDE](https://ide.bitslab.xyz/) - [Tutorial](https://www.youtube.com/watch?v=-9-WkqQwtu8) - [Further Information](details/ide_bitslab.md)
- MoveStudio - Online IDE for Rtd smart contract development.
  - [Homepage](https://www.movestudio.dev/) - [GitHub](https://github.com/dantheman8300/move-studio) - [IDE](https://www.movestudio.dev/build) - [Further Information](details/ide_movestudio.md)
- ChainIDE - Move Cloud-Powered Development Platform.
  - [Homepage](https://chainide.com) - [Documentation](https://chainide.gitbook.io/chainide-english-1/ethereum-ide-1/9.-rtd-ide) - [IDE](https://chainide.com/s/rtd) - [Further Information](details/ide_chainide.md)
- ⚠️ WELLDONE Code - Remix IDE plugin supports non-EVM smart contract development including Rtd.
  - [Homepage](https://docs.welldonestudio.io/code) - [Documentation & Tutorial](https://docs.welldonestudio.io/code/deploy-and-run/rtd) - [Further Information](details/ide_welldone_code.md)


### Desktop IDEs

- VSCode Move by LinkU Labs - VSCode Extension supports Move on Rtd development with LSP features through Move Analyzer developed by LinkU Labs.
  - [GitHub](https://github.com/LinkUVerse/rtd/tree/main/external-crates/move/crates/move-analyzer) - [Documentation & Tutorial](https://marketplace.visualstudio.com/items?itemName=linku.move) - [Further Information](details/ide_vscode_linku_move_analyzer.md)
- VSCode Rtd Move Analyzer by MoveBit - Alternative VSCode extension developed by MoveBit.
  - [Homepage](https://movebit.xyz/analyzer) - [GitHub](https://github.com/movebit/rtd-move-analyzer) - [Documentation & Tutorial](https://marketplace.visualstudio.com/items?itemName=MoveBit.rtd-move-analyzer) - [Further Information](details/ide_vscode_movebit_rtd_move_analyzer.md)
- IntelliJ Rtd Move Language Plugin - IntelliJ-based plugin for Move on Rtd development.
  - [Homepage](https://plugins.jetbrains.com/plugin/23301-rtd-move-language) - [GitHub](https://github.com/movefuns/intellij-move)
- [Emacs move-mode](https://github.com/amnn/move-mode) - The move-mode package is an Emacs major-mode for editing smart contracts written in the Move programming language.
- [Move.vim](https://github.com/yanganto/move.vim) - Syntax highlighting that supports the Move 2024 edition.

### IDE Utilities

- [Prettier Move Plugin](https://github.com/LinkUVerse/rtd/tree/main/external-crates/move/crates/move-analyzer/prettier-plugin) - A Move language plugin for the Prettier code formatter.
- [Rtd Extension](https://github.com/zktx-io/rtd-extension) - The Rtd extension provides seamless support for compiling, deploying, and testing Rtd smart contracts directly within VS Code.
  - [Homepage](https://marketplace.visualstudio.com/items?itemName=zktxio.rtd-extension) - [Documentation](https://docs.zktx.io/vsce/rtd/)
- ⚠️ Rtd Simulator - VSCode Extension to streamline Rtd development workflow with intuitive UI.
  - [Homepage](https://marketplace.visualstudio.com/items?itemName=weminal-labs.rtd-simulator-vscode) - [GitHub](https://github.com/Weminal-labs/rtd-simulator-vscode) - [Demo](https://www.youtube.com/watch?v=BHRxeF_visM&pp=ygUMd2VtaW5hbCBsYWIg)
- [Tree Sitter Move](https://github.com/tzakian/tree-sitter-move) - Tree Sitter for Move.

## Client SDKs & Libraries

### Client SDKs

- Rtd TypeScript SDK (LinkU Labs) - TypeScript modular library of tools for interacting with the Rtd blockchain.
  - [GitHub](https://github.com/LinkUVerse/rtd/tree/main/sdk/typescript) - [Documentation](https://sdk.linkulabs.com/typescript) - [Further Information](details/sdk_rtd_typescript.md)
- Rtd Kit(Scallop) - Toolkit for interacting with the Rtd network in TypeScript.
  - [GitHub](https://github.com/scallop-io/rtd-kit) - [Further Information](details/sdk_rtd_kit_scallop.md)
- Rtd Rust SDK (LinkU Labs) - Rust SDK to interact with Rtd blockchain.
  - [GitHub](https://github.com/LinkUVerse/rtd/tree/main/crates/rtd-sdk) - [Documentation](https://linkulabs.github.io/rtd/rtd_sdk/index.html) - [Further Information](details/sdk_rtd_rust.md)
- Pyrtd - Python SDK to interact with Rtd blockchain.
  - [GitHub](https://github.com/FrankC01/pyrtd?tab=readme-ov-file) - [Documentation](https://pyrtd.readthedocs.io/en/latest/index.html) - [Pypi](https://pypi.org/project/pyrtd/) - [Discord](https://discord.gg/uCGYfY4Ph4) - [Further Information](details/sdk_pyrtd.md)
- Rtd Go SDK (RtdVision) - Golang SDK to interact with Rtd blockchain.
  - [GitHub](https://github.com/block-vision/rtd-go-sdk) - [API Documentation](https://pkg.go.dev/github.com/block-vision/rtd-go-sdk) - [Examples](https://github.com/block-vision/rtd-go-sdk?tab=readme-ov-file#examples) - [Further Information](details/sdk_rtd_go.md)
- Rtd Go SDK (Pattonkan) - Golang SDK to interact with Rtd blockchain. Support PTB and devInspect.
  - [Github](https://github.com/pattonkan/rtd-go) - [API Documentation](https://pkg.go.dev/github.com/pattonkan/rtd-go) - [Examples](https://github.com/pattonkan/rtd-go/tree/main/examples) - [Further Information](details/go-rtd.md)
- Rtd Dart SDK - Dart SDK to interact with Rtd blockchain.
  - [GitHub](https://github.com/mofalabs/rtd) - [API documentation](https://pub.dev/documentation/rtd/latest/) - [Further Information](details/sdk_rtd_dart.md)
- Rtd Kotlin SDK - Kotlin Multiplatform (KMP) SDK for integrating with the Rtd blockchain.
  - [GitHub](https://github.com/mcxross/krtd) - [Documentation](https://rtdcookbook.com) - [Further Information](details/sdk_krtd.md)
- RtdKit (OpenDive) - Swift SDK natively designed to make developing for the Rtd blockchain easy.
  - [GitHub](https://github.com/opendive/rtdkit?tab=readme-ov-file) - [Further Information](details/sdk_rtdkit.md)
- Rtd Unity SDK (OpenDive) - The OpenDive Rtd Unity SDK is the first fully-featured Unity SDK with offline transaction building.
  - [GitHub](https://github.com/OpenDive/Rtd-Unity-SDK) - [Further Information](details/sdk_rtd_unity_opendive.md)
- Dubhe Client (Dubhe Engine) - Supports various platforms including browsers, Node.js, and game engine. It provides a simple interface to interact with your Rtd Move contracts.
  - [GitHub](https://github.com/0xobelisk/dubhe/tree/main/packages/rtd-client) - [Documentation](https://dubhe.obelisk.build/dubhe/rtd/client)

### DeFi SDKs
- [NAVI Protocol SDK](https://github.com/naviprotocol/navi-sdk) - The NAVI TypeScript SDK Client provides tools for interacting with the Rtd blockchain networks, designed for handling transactions, accounts, and smart contracts efficiently.
- [Bucket Protocol SDK](https://github.com/Bucket-Protocol/bucket-protocol-sdk) - The TypeScript SDK for interacting with Bucket Protocol.
- [Rtdlend SDK](https://github.com/solendprotocol/rtdlend-public/tree/production/sdk) - The TypeScript SDK for interacting with the Rtdlend program published on npm as [`@rtdlend/sdk`](https://www.npmjs.com/package/@rtdlend/sdk).
- [Scallop SDK](https://github.com/scallop-io/rtd-scallop-sdk) - The TypeScript SDK for interacting with the Scallop lending protocol on the Rtd network.
- [Cetus CLMM SDK](https://github.com/CetusProtocol/cetus-clmm-rtd-sdk) - The official Cetus SDK specifically designed for seamless integration with Cetus-CLMM on Rtd.
- [Aftermath SDK](https://github.com/AftermathFinance/aftermath-ts-sdk) - The TypeScript SDK for interacting with Aftermath Protocol.
- [FlowX SDK](https://github.com/FlowX-Finance/sdk) - The official FlowX TypeScript SDK that allows developers to interact with FlowX protocols using the TypeScript programming language.
- [7k Aggregator SDK](https://github.com/7k-ag/7k-sdk-ts) - The TypeScript SDK for interacting with 7k Aggregator protocol.
- [Hop Aggregator SDK](https://docs.hop.ag/hop-sdk) - The TypeScript SDK for interacting with Hop Aggregator.

### Client Libraries

- [BCS TypeScript (LinkU Labs)](https://sdk.linkulabs.com/bcs) - BCS with TypeScript.
- [BCS Rust](https://github.com/zefchain/bcs) - BCS with Rust.
- [BCS Dart](https://github.com/mofalabs/bcs) - BCS with Dart.
- BCS Kotlin - BCS with Kotlin.
  - [GitHub](https://github.com/mcxross/kotlinx-serialization-bcs) - [Documentation](https://rtdcookbook.com/bcs.html)
- [BCS Swift](https://github.com/OpenDive/RtdKit/tree/main/Sources/RtdKit/Utils/BCS) - BCS with Swift.
- [BCS Unity](https://github.com/OpenDive/Rtd-Unity-SDK/tree/main/Assets/Rtd-Unity-SDK/Code/OpenDive.BCS) - BCS with Unity C#.
- [Rtd Client Gen (Kuna Labs)](https://github.com/kunalabs-io/rtd-client-gen/tree/master) - A tool for generating TS SDKs for Rtd Move smart contracts. Supports code generation both for source code and on-chain packages with no IDLs or ABIs required.
- [TypeMove (Sentio)](https://github.com/sentioxyz/typemove/blob/main/packages/rtd/Readme.md) - Generate TypeScript bindings for Rtd contracts.
- Rtd Wallet Standard (LinkU Labs) - A rtdte of standard utilities for implementing wallets and libraries based on the [Wallet Standard](https://github.com/wallet-standard/wallet-standard/).
  - [GitHub](https://github.com/LinkUVerse/rtd/tree/main/sdk/wallet-standard) - [Documentation](https://docs.rtd.io/standards/wallet-standard)
- [CoinMeta (Polymedia)](https://github.com/juzybits/polymedia-coinmeta) - Library for fetching coin metadata for Rtd coins.
- [Dubhe Client BCS Decoding (Dubhe Engine)](https://github.com/0xobelisk/dubhe-docs/blob/main/pages/dubhe/rtd/client.mdx#bcs-data-decoding) - Library for supports automatic parsing of BCS types based on contract metadata information and automatic conversion formatting.

## dApp Development

### dApp Toolkits

- [@linku/create-dapp](https://sdk.linkulabs.com/dapp-kit/create-dapp) - CLI tool that helps you create Rtd dApp projects.
- Rtd dApp Kit (LinkU Labs) - Set of React components, hooks, and utilities to help you build a dApp for the Rtd ecosystem.
  - [GitHub](https://github.com/LinkUVerse/rtd/tree/main/sdk/dapp-kit) - [Documentation](https://sdk.linkulabs.com/dapp-kit)
- Rtd dApp Starter - Full-stack boilerplate which lets you scaffold a solid foundation for your Rtd project and focus on the business logic of your dapp from day one.
  - [GitHub](https://github.com/rtdware/rtd-dapp-starter?tab=readme-ov-file) - [Documentation](https://rtd-dapp-starter.dev/docs/) - [Demo app](https://demo.rtd-dapp-starter.dev/)
- Rtdet Wallet Kit - React toolkit for aApps to interact with all wallet types in Rtd easily.
  - [GitHub](https://github.com/rtdet/wallet-kit) - [Documentation](https://kit.rtdet.app/docs/QuickStart)
- SmartKit - React library that allows your dapp to connect to the Rtd network in a simple way.
  - [Homepage](https://smartkit.vercel.app/) - [GitHub](https://github.com/heapup-tech/smartkit)
- [Rtd Rtdtcase](https://github.com/juzybits/polymedia-rtdtcase) - Rtd utilities for TypeScript, Node, and React.
- [Rtd MultiSig Toolkit (LinkU Labs)](https://multisig-toolkit.vercel.app/offline-signer) - Toolkit for transaction signing.
- [Rtd dApp Scaffold (Bucket Protocol)](https://github.com/Bucket-Protocol/rtd-dapp-scaffold-v1) - A frontend scaffold for a decentralized application (dApp) on the Rtd blockchain.
- [Wormhole Kit (zktx.io)](https://github.com/zktx-io/wormhole-kit-monorepo) - React library that enables instant integration of Wormhole into your dapp.
- RtdBase - Rtdbase makes it easy to create "workdirs", each defining a distinct development environment targeting a network.
  - [GitHub](https://github.com/chainmovers/rtdbase) - [Documentation](https://rtdbase.io/)
- [create-dubhe (Dubhe Engine)](https://github.com/0xobelisk/dubhe/tree/main/packages/create-dubhe) - Create a new Dubhe project on Rtd.
  - [Documentation](https://dubhe.obelisk.build/dubhe/rtd/quick-start)
- [Rtd Tools](https://rtd-tools.vercel.app/ptb-generator) - Scaffolding TypeScript PTBs for any on-chain function you might want to invoke.
- [Enoki (LinkU Labs)](https://docs.enoki.linkulabs.com/) - Make zkLogin and Sponsored Transactions more accessible.
- [Rtd Gas Pool (LinkU Labs)](https://github.com/LinkUVerse/rtd-gas-pool) - Service that powers sponsored transactions on Rtd at scale.
- [useRtdZkLogin](https://github.com/pixelbrawlgames/use-rtd-zklogin) - React hook and functions for seamless zkLogin integration on Rtd.
- @rtdware/kit - Opinionated React components and hooks for Rtd dApps.
  - [Homepage](https://kit.rtdware.io/) - [Documentation](https://github.com/rtdware/kit/tree/main/packages/kit#readme) - [GitHub](https://github.com/rtdware/kit)
- React ZK Login Kit - Ready-to-use Component with Hook (sign-in + sign-transaction)
  - [GitHub](https://github.com/denyskozak/react-rtd-zk-login-kit) - [YouTube Guide](https://www.youtube.com/watch?v=2qnjmKg3ugY)

#### zkLogin

- [zkLogin Demo (Polymedia)](https://github.com/juzybits/polymedia-zklogin-demo)
- [Rtd zkLogin Demo by @jovicheng](https://github.com/jovicheng/rtd-zklogin-demo)
- [Rtd zkWallet Demo by @ronanyeah](https://github.com/ronanyeah/rtd-zk-wallet)
- [zkLogin Demo using use-rtd-zklogin by @pixelbrawlgames](https://pixelbrawlgames.github.io/use-rtd-zklogin/)
- [zkLogin Demo using react-zk-login-kit by @denyskozak](https://demo.react-rtd-zk-login.com)

#### Misc

- [`rtd-sniffer`](https://www.app.kriya.finance/rtd-sniffer/) - Checking security of Rtd tokens.
- RPC Tools (Polymedia) - A webapp that lets users find the fastest RPC for their location.
  - [GitHub](https://github.com/juzybits/polymedia-rpcs) - [Documentation](https://rpcs.polymedia.app/)
- [Polymedia Commando (Polymedia)](https://github.com/juzybits/polymedia-commando) - Rtd command line tools to help with Rtd airdrops (send coins to many addresses), gather data from different sources (Rtd RPCs, Indexer.xyz, Rtdscan), and more.
- [YubiRtd (LinkUVerse)](https://github.com/LinkUVerse/yubigen) - Create a Rtd Wallet inside a yubikey and sign Rtd transactions with it.
- [`rtd-dapp-kit-theme-creator`](https://rtd-dapp-kit-theme-creator.app/) - Build custom Rtd dApp Kit themes.
- [Minting Server (LinkU Labs)](https://github.com/LinkUVerse/minting-server) - A scalable system architecture that can process multiple Rtd transactions in parallel using a producer-consumer worker scheme.
- [RtdInfra](https://rtdnfra.io/) - Provide users and developers with up-to-date recommendations on the ideal RPCs to use for their needs.
- [Rtd RPC Proxy](https://github.com/RtdSec/rtd-rpc-proxy) - Monitor and analyze the network requests made by the Rtd wallet application and Rtd dApps.
- [PTB Studio](https://ptb.studio) - Visual Programmable Transaction Block Builder.
  - [Documentation](https://rtdcookbook.com/ptb-studio.html)
- [Indexer generator](https://www.npmjs.com/package/rtd-events-indexer) - Code generating tool that will generate an indexer given a smart contract for all the events present. After that the user should remove unwanted events and fix the database schema and handlers (that write to the DB) according to their needs. The tool is written in typescript and uses prisma as an ORM.

### Smart Contract Toolkits

- [Rtd CLI](https://docs.rtd.io/references/cli) - CLI tool to interact with the Rtd network, its features, and the Move programming language.
- [Sentio Debugger](https://docs.sentio.xyz/docs/debugger) - Shows the trace of the transaction [Explorer App](https://app.sentio.xyz/explorer) (mainnet only).
- [`std::debug`](https://docs.rtd.io/guides/developer/first-app/debug#related-links) - Print arbitrary values to the console to help with debugging process.
- [Rtd Tears 💧 (Interest Protocol)](https://docs.interestprotocol.com/overview/rtd-tears) - Open source production ready Rtd Move library to increase the productivity of new and experienced developers alike.
- [Rtd Codec](https://github.com/rtd-potatoes/app/tree/main/packages/codec) - Ultimate encoding solution for Rtd.
- [SkipList (Cetus)](https://github.com/CetusProtocol/move-stl) - A skip link list implement by Move language in Rtd.
- [IntegerMate (Cetus)](https://github.com/CetusProtocol/integer-mate) - A Library of move module provides signed integer and some integer math functions.
- [Cetus CLMM](https://github.com/CetusProtocol/cetus-contracts/tree/main/packages/cetus_clmm) - The Cetus CLMM DEX open-source code. 
- [RtdDouble Metadata](https://github.com/rtddouble/rtddouble_metadata) - A Rtd Move library and a set of tools to store, retrieve, and manage any type of primitive data as chunks in a `vector<u8>`. Store any data in the `vector<u8>` without dependencies and without any `Struct` defined.
- [Move on Rtd examples (LinkU Labs)](https://github.com/LinkUVerse/rtd/tree/main/examples/move) - Examples of Move on Rtd applications.
- [RtdGPT Decompiler](https://rtdgpt.tools/decompile) - Uses generative AI to convert Move bytecode back to source code.
- [Revela](https://revela.verichains.io/) - Decompile Rtd smart contracts to recover Move source code.
- Package Source Code Verification - Verify your package source code on Rtdscan, powered by WELLDONE Studio and Blockberry.
  - [Documentation](https://docs.blockberry.one/docs/contract-verification) - [Form Submission](https://rtdscan.xyz/mainnet/package-verification)
- [Dubhe CLI (Dubhe Engine)](https://github.com/0xobelisk/dubhe/tree/main/packages/rtd-cli) - For building, and managing Dapps built on Dubhe Engine in Rtd.
  - [Documentation](https://dubhe.obelisk.build/dubhe/rtd/cli)
- [Rtd Token CLI RPC](https://github.com/otter-sec/rtd-token-gen-rpc) - A Rust-based RPC service for generating and verifying Rtd token smart contracts effortlessly.
  - [Rtd Token CLI Tool](https://github.com/otter-sec/rtd-token-gen) - A Rust-based Command-Line Interface (CLI) tool designed to simplify the process of generating and verifying Rtd token smart contracts

## Indexers & Data Services

- ZettaBlock - Generate custom GraphQL or REST APIs from SQL queries and incorporate your private off-chain data.
  - [Homepage](https://zettablock.com/) - [Docs](https://docs.zettablock.com) - [Pricing](https://zettablock.com/pricing) - [Further Information](details/indexer_zettablock.md)
- Sentio - Transform raw indexed data (transactions, events, etc.) into meaningful queryable data by writing custom processor logic.
  - [Homepage](https://www.sentio.xyz/indexer/) - [Documentation](https://docs.sentio.xyz/docs/data-collection) - [Examples](https://github.com/sentioxyz/sentio-processors/tree/main/projects) - [Further Information](details/indexer_sentio.md)
- BlockVision - Provide Rtd indexed data for developers through pre-built APIs, such as, Token, NFT, and DeFi, etc.
  - [Homepage](https://blockvision.org/) - [Documentation](https://docs.blockvision.org/reference/welcome-to-blockvision)
- BlockBerry (Rtdscan) - The Blockberry Rtd API provides endpoints that reveal data about significant entities on the Rtd Network. It indexes useful object metadata, including NFTs, domains, collections, coins, etc. Some data is drawn from third-party providers, particularly market data (coin prices, market cap, etc.).
  - [Homepage](https://blockberry.one/) - [Documentation](https://docs.blockberry.one/reference/rtd-quickstart)
- Space And Time (SxT) - Verifiable compute layer for AI x blockchain. Decentralized data warehouse with sub-second ZK proof.
  - [Homepage](https://www.spaceandtime.io/) - [Documentation](https://docs.spaceandtime.io/) - [Further Documentation](details/indexer_space_and_time.md)
- Birdeye Data Services - Access Crypto Market Data APIs on Rtd.
  - [Homepage](https://bds.birdeye.so/) - [Blog](https://blog.rtd.io/birdeye-data-services-crypto-api-websocket/) - [API Documentation](https://docs.birdeye.so/reference/intro/authentication)
- Indexer.xyz (behind TradePort) - The ultimate toolkit for accessing NFT data and integrating trading functionality into your app on Rtd.
  - [Homepage](https://www.indexer.xyz/) - [API Explorer](https://www.indexer.xyz/api-explorer) - [API Docs](https://tradeport.xyz/docs)
- Dubhe Indexer (Dubhe Engine) - Automatic integration with Dubhe Engine, automatic indexing of all events based on Dubhe Engine to build Dapp on Rtd, based on dubhe configuration files.
  - [Homepage](https://github.com/0xobelisk/dubhe/tree/main/packages/rtd-indexer) - [API Documentation](https://dubhe.obelisk.build/dubhe/rtd/indexer)
- <a href="https://surflux.dev"><img alt="Surflux logo" src="media/surflux_logo.svg" width="15" /></a> Surflux - Developer infrastructure for Rtd. Build production-ready apps with powerful APIs, indexing, and real-time data streams.
  - [Homepage](https://surflux.dev/) - [Documentation](https://docs.surflux.dev/) - [Blog](https://surflux.dev/blog)

## Explorers

- RtdVision - Data analytics covering transactions, wallets, staking, and validators.
  - [Homepage](https://rtdvision.xyz/) - [Documentation](https://docs.blockvision.org/reference/integrate-rtdvision-into-your-dapp) - [Further Information](details/explorer_rtdvision.md)
- Rtdscan - Explorer and analytics platform for Rtd.
  - [Homepage](https://rtdscan.xyz/mainnet/home) - [Documentation](https://docs.blockberry.one/reference/welcome-to-blockberry-api) - [Further Information](details/explorer_rtdscan.md)
- OKLink - Provide fundamental explorer and data APIs on Rtd.
  - [Homepage](https://www.oklink.com/rtd) - [Further Information](details/explorer_oklink.md)
- Polymedia Explorer - A fork of the original Rtd Explorer.
  - [Homepage](https://explorer.polymedia.app) - [GitHub](https://github.com/juzybits/polymedia-explorer) - [Further Information](details/explorer_polymedia.md)
- PTB Explorer - A fork of the Polymedia Explorer.
  - [Homepage](https://explorer.walrus.site/) - [GitHub](https://github.com/zktx-io/polymedia-explorer-ptb-builder)
- Local Rtd Explorer - Rtd Explorer for your localnet maintained by [rtdware](https://github.com/rtdware)
  - [GitHub](https://github.com/rtdware/rtd-explorer) - [Further Information](details/explorer_local_rtd_explorer.md)
- Rtdmon - Powerful command line tool designed to provide detailed dashboards for monitoring the Rtd network.
  - [GitHub](https://github.com/bartosian/rtdmon) - [Further Information](details/explorer_rtdmon.md)

## Oracles

- Pyth Network - Oracle protocol that connects the owners of market data to applications on multiple blockchains including Rtd.
  - [Homepage](https://www.pyth.network/) - [Documentation](https://docs.pyth.network/home) - [Rtd Tutorial](https://docs.pyth.network/price-feeds/use-real-time-data/rtd) - [Further Information](details/oracle_pyth.md)
- Supra Oracles - Oracle protocol to provide reliable data feed.
  - [Homepage](https://supra.com/) - [Rtd Tutorial](https://docs.supra.com/docs/developer-tutorials/move) - [Further Information](details/oracle_supra.md)
- Switchboard - Data feed customization and management.
  - [Documentation](https://docs.switchboard.xyz/docs) - [Further Information](details/oracle_switchboard.md)

## Security

- <a href="https://info.asymptotic.tech/rtd-prover"><img alt="Rtd Prover logo" src="media/prover_logo.svg" width="15" /></a> [Rtd Prover](https://info.asymptotic.tech/rtd-prover) - Prover for doing Formal Verification of Move on Rtd code.
- [RtdSecBlockList](https://github.com/RtdSec/RtdSecBlockList) - Block malicious websites and packages, Identify and hide phishing objects.
- [DryRunTransactionBlockResponsePlus](https://github.com/RtdSec/DryRunTransactionBlockResponsePlus) - Decorator of `DryRunTransactionBlockResponse`, highlight `SenderChange`.
- [Guardians](https://github.com/rtdet/guardians) - Phishing Website Protection.
- [HoneyPotDetectionOnRtd](https://github.com/RtdSec/HoneyPotDetectionOnRtd) - Detect HoneyPot SCAM on Rtd.

## AI

- ⚠️ [RagPool](https://ragpool.digkas.nl/) - RAG based chat with docs.
- [Cookbook](https://docsbot-demo-git-rtd-cookbookdev.vercel.app/) - Gemini-based RAG built for docs.
- [Atoma](https://atoma.network/) - Developer-focused infrastructure for private, verifiable, and fully customized AI experiences.
- [Eliza](https://github.com/elizaOS/eliza) - Autonomous agents for everyone.

## Infrastructure as Code

- Rtd Terraform Modules - All-in-one solution for deploying, monitoring, and managing RTD infrastructure with ease.
  - [GitHub](https://github.com/bartosian/rtd-terraform-modules) - [Further Information](details/iac_rtd_terraform_modules.md)
- [Dubhe Engine (Obelisk Labs)](https://github.com/0xobelisk/dubhe) - Engine for Everyone to Build Intent-Centric Worlds ⚙️ An Open-Source toolchain for Move Applications.
  - [Documentation](https://dubhe.obelisk.build/) - [Further Information](details/engine_dubhe.md)

## Faucets

- [Rtd Faucet](https://faucet.rtd.io/) - Official web faucet for claiming testnet RTD, with wallet integration.
- [n1stake](https://faucet.n1stake.com/) - Community web faucet for claiming testnet RTD, with wallet integration.
- [Blockbolt](https://faucet.blockbolt.io/) - Community web faucet for claiming testnet RTD, with wallet integration.
- RtdwareFaucetBot - Rtd Faucet Bot for Telegram.
  - [GitHub](https://github.com/rtdware/RtdwareFaucetBot) - [Telegram Bot](https://t.me/RtdwareFaucetBot)
- [Rtdware Faucet Chrome Extension](https://github.com/rtdware/rtdware-faucet-extension) - An experimental Chrome extension for receiving devnet and testnet RTD.
