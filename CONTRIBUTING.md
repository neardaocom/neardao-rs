# Contributing to neardao-rs

:+1::tada: First off, thanks for taking the time to contribute! :tada::+1:

The following is a set of guidelines for contributing to blockchain part of NearDAO platform. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

### Table of contents

[Code of Conduct](#code-of-conduct)

[What should I know before I get started?](#what-should-i-know-before-i-get-started)

- [Architecture](#architecture)
- [Development Tools](#development-tools)

[How Can I Contribute?](#how-can-i-contribute)

- [Reporting Bugs](#reporting-bugs)
- [Any other kinds of contribution](#any-other-kinds-of-contribution)

[Styleguides](#styleguides)

- [Git Commit Messages](#git-commit-messages)
- [Rust Styleguide](#rust-styleguide)

[Contact us](#contact-us)

## Code of Conduct

This project and everyone participating in it is governed by the [NearDAO Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [dev@gneardao.com](mailto:dev@gneardao.com).

## What should I know before I get started?

### Architecture

We recommend to read at our [NEARDAO documentation](https://info-128.gitbook.io/neardao/) to get familiar with the project.

### Development tools

We recommend to use VSCode with rust-analyser extension but any other IDE is ok as it supports this extension (eg. CLion by JetBrains).
Docker is necessary to be able to build the code as we use dockerized contract-builder to build it.
In general any x86-64 machine with decent computing power should be ok. The new Apple Silicon machines might now work with integration tests. Workspaces-rs doc [here](https://github.com/near/workspaces-rs#m1-macos) mentions this issue.

## How can I contribute

### Reporting bugs

If you find a bug, don't hesitate to open an issue. Please try to provide as much information as possible to help us identify the source of the issue. PR's with fix obviously even the best solution :)

**Please use following template:**
| Section | Description |
|---|---|
| Title | Briefly describe the bug. |  
| Resulting and expected behaviour | Specify what happened and what you expected to happen. |
| Transaction link | Provide testnet's or mainnet's transaction link. |

### Any other kinds of contribution

At the time of the writing, the best way is to [contact us](#contact-us) directly.

## Styleguides

### Git commit messages

We adhere (since v1.0.0) to [Convetional commits](https://www.conventionalcommits.org/en/v1.0.0/#summary).

### Rust styleguide

Please use rustfmt before each commit to keep the code formatted.

## Contact us

Haven`t found answer to your question or something is here missing/not complete? Contact us at [Discord channel](https://discord.gg/ED7Gj3tG) or send us email to [dev@gneardao.com](mailto:dev@gneardao.com).
